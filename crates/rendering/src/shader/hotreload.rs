use crate::instance_data::InstanceData;
use crate::shader::create_pipeline;
use crate::vertex::Vertex;
use notify_debouncer_full::{
	DebounceEventResult, Debouncer, RecommendedCache,
	notify::{self, RecommendedWatcher},
};
use sphynx_logging::{error, info};
use sphynx_wesl_include as _;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, TryRecvError};
use std::time::Duration;
use wgpu::{
	BindGroupLayout, Device, RenderPass, RenderPipeline, ShaderSource, TextureFormat,
	VertexBufferLayout,
};

#[macro_export]
macro_rules! include_shader {
	($shader_root_dir:literal, $shader_module_name:literal, $bindings:expr, $device:expr, $swapchain_format:expr) => {{
		$crate::shader::hotreload::Shader::from_wesl_instanced::<
			$crate::vertex::PCN_Vertex,
			$crate::instance_data::Model_InstanceData,
		>(
			&::std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
				.join($shader_root_dir)
				.to_string_lossy(),
			$shader_module_name,
			$bindings,
			$device,
			$swapchain_format,
		)
	}};
}

#[macro_export]
macro_rules! include_custom_shader {
	($vertex:expr, $instance:expr, $shader_root_dir:expr, $shader_module_name:expr, $bindings:expr, $device:expr, $swapchain_format:expr) => {{
		$crate::Shader::from_wesl_instanced::<$vertex, $instance>(
			&::std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
				.join($shader_root_dir)
				.to_string_lossy(),
			$shader_module_name,
			$bindings,
			$device,
			$swapchain_format,
		)
	}};
}

pub struct Shader {
	shader_dir: PathBuf,
	module: String,
	package_dirs: HashSet<PathBuf>,
	vertex_layout: VertexBufferLayout<'static>,
	instance_layout: Option<VertexBufferLayout<'static>>,
	bindings: Vec<Option<BindGroupLayout>>,
	label: Option<String>,
	swapchain_texture_view_format: TextureFormat,
	device: Device,
	render_pipeline: RefCell<RenderPipeline>,
	hot_reload_receiver: mpsc::Receiver<DebounceEventResult>,
	/// RefCell so that if the shader imports a new package, the file watcher can watch that too
	hot_reload_file_watcher: RefCell<Debouncer<RecommendedWatcher, RecommendedCache>>,
}
impl Shader {
	pub fn from_wesl<V: Vertex>(
		shader_root_dir: &str,
		shader_module_name: &str,
		instance_layout: Option<VertexBufferLayout<'static>>,
		bindings: &[Option<&BindGroupLayout>],
		device: &Device,
		swapchain_texture_view_format: TextureFormat,
	) -> Self {
		assert!(
			swapchain_texture_view_format.is_srgb(),
			"the texture view of the swapchain texture should always be srgb, we dont apply gamma correction ourselfs"
		);

		let shader_dir = Path::new(shader_root_dir);
		let compiled =
			sphynx_wesl::compile_module(shader_dir, shader_module_name).unwrap_or_else(|e| {
				panic!(
					"failed to compile WESL shader `{shader_module_name}` from `{shader_root_dir}`: {e}"
				)
			});

		let label = Some(shader_module_name);
		let pipeline = create_pipeline(
			ShaderSource::Wgsl(Cow::Borrowed(&compiled.wgsl_code)),
			V::desc(),
			instance_layout.clone(),
			label,
			bindings,
			device,
			swapchain_texture_view_format,
		);

		let (tx, rx) = mpsc::channel::<DebounceEventResult>();
		let mut watcher =
			notify_debouncer_full::new_debouncer(Duration::from_millis(100), None, tx)
				.expect("failed to create a shader hotreload file watcher");
		for package_dir in compiled.package_dirs.iter() {
			watcher
				.watch(package_dir, notify::RecursiveMode::Recursive)
				.expect("failed to watch files for shader hotreloading");
		}

		Self {
			shader_dir: shader_dir.to_path_buf(),
			module: shader_module_name.to_string(),
			vertex_layout: V::desc(),
			instance_layout,
			package_dirs: HashSet::from_iter(compiled.package_dirs),
			label: label.map(str::to_string),
			bindings: bindings.iter().map(|binding| binding.cloned()).collect(),
			swapchain_texture_view_format,
			device: device.clone(),
			render_pipeline: RefCell::new(pipeline),
			hot_reload_receiver: rx,
			hot_reload_file_watcher: RefCell::new(watcher),
		}
	}

	pub fn from_wesl_instanced<V: Vertex, I: InstanceData>(
		shader_root_dir: &str,
		shader_module_name: &str,
		bindings: &[Option<&BindGroupLayout>],
		device: &Device,
		swapchain_texture_view_format: TextureFormat,
	) -> Self {
		Self::from_wesl::<V>(
			shader_root_dir,
			shader_module_name,
			Some(I::desc()),
			bindings,
			device,
			swapchain_texture_view_format,
		)
	}

	pub fn bind<'a>(&'a self, renderpass: &'_ mut RenderPass<'a>) {
		self.try_recompile_shader_pipeline();

		// the .clone() is a shallow clone of the renderpipeline handle
		renderpass.set_pipeline(&self.render_pipeline.borrow().clone());
	}

	fn try_recompile_shader_pipeline(&self) {
		loop {
			match self.hot_reload_receiver.try_recv() {
				Ok(events) => match events {
					Ok(events) => {
						let should_reload = events.into_iter().any(|e| {
							e.kind.is_create()
								|| e.kind.is_modify()
								|| e.kind.is_remove()
								|| e.kind.is_other()
						});
						if should_reload {
							break;
						}
					}
					Err(errors) => error!(
						Rendering,
						"error(s) with shader hotreloading notification event handler:\n{}",
						errors
							.into_iter()
							.map(|e| e.to_string())
							.collect::<Vec<_>>()
							.join("\n")
					),
				},
				Err(TryRecvError::Disconnected) => {
					error!(
						Rendering,
						"the shader hot-reload file watcher for `{}` must have shutdown too soon or crashed",
						self.module
					);
					return;
				}
				Err(TryRecvError::Empty) => return,
			}
		}

		// drain out all other events, since we already decided to hot-reload
		while self.hot_reload_receiver.try_recv().is_ok() {}

		match sphynx_wesl::compile_module(&self.shader_dir, &self.module) {
			Ok(compiled) => {
				for package_dir in compiled.package_dirs {
					if !self.package_dirs.contains(&package_dir) {
						self.hot_reload_file_watcher
							.borrow_mut()
							.watch(&package_dir, notify::RecursiveMode::Recursive)
							.expect("failed to watch new referenced dir");
					}
				}

				let bindings = self.bindings.iter().map(|b| b.as_ref()).collect::<Vec<_>>();

				let pipeline = create_pipeline(
					ShaderSource::Wgsl(Cow::Borrowed(&compiled.wgsl_code)),
					self.vertex_layout.clone(),
					self.instance_layout.clone(),
					self.label.as_deref(),
					&bindings,
					&self.device,
					self.swapchain_texture_view_format,
				);

				info!(Rendering, "hot-reloaded shader `{}`", self.module);
				*self.render_pipeline.borrow_mut() = pipeline;
			}
			Err(e) => {
				error!(
					Rendering,
					"failed to hot-reload shader `{}`, keeping the previous version: {e}",
					self.module
				);
			}
		}
	}
}
