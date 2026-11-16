use crate::instance_data::InstanceData;
use crate::shader::create_pipeline;
use crate::vertex::Vertex;
use std::borrow::Cow;
use wgpu::{
	BindGroupLayout, Device, RenderPass, RenderPipeline, ShaderSource, TextureFormat,
	VertexBufferLayout,
};

#[cfg(not(all(feature = "hot-reload", not(target_arch = "wasm32"))))]
#[macro_export]
macro_rules! include_shader {
	($shader_root_dir:literal, $shader_module_name:literal, $bindings:expr, $device:expr, $swapchain_format:expr) => {{
		$crate::Shader::from_str_instanced::<
			$crate::vertex::PCN_Vertex,
			$crate::instance_data::Model_InstanceData,
		>(
			::sphynx_wesl_include::include_wesl!($shader_root_dir, $shader_module_name),
			Some($shader_module_name),
			$bindings,
			$device,
			$swapchain_format,
		)
	}};
}

#[cfg(not(all(feature = "hot-reload", not(target_arch = "wasm32"))))]
#[macro_export]
macro_rules! include_custom_shader {
	($vertex:expr, $instance:expr, $shader_root_dir:expr, $shader_module_name:expr, $bindings:expr, $device:expr, $swapchain_format:expr) => {{
		$crate::Shader::from_str_instanced::<$vertex, $instance>(
			::sphynx_wesl_include::include_wesl!($shader_root_dir, $shader_module_name),
			Some($shader_module_name),
			$bindings,
			$device,
			$swapchain_format,
		)
	}};
}

pub struct Shader {
	pipeline: RenderPipeline,
}

impl Shader {
	pub fn from_str<V: Vertex>(
		shader_code: &str,
		instance_layout: Option<VertexBufferLayout<'static>>,
		label: Option<&str>,
		bindings: &[Option<&BindGroupLayout>],
		device: &Device,
		swapchain_texture_view_format: TextureFormat,
	) -> Self {
		Self::new(
			ShaderSource::Wgsl(Cow::Borrowed(shader_code)),
			V::desc(),
			instance_layout,
			label,
			bindings,
			device,
			swapchain_texture_view_format,
		)
	}

	pub fn from_str_instanced<V: Vertex, I: InstanceData>(
		shader_code: &str,
		label: Option<&str>,
		bindings: &[Option<&BindGroupLayout>],
		device: &Device,
		swapchain_texture_view_format: TextureFormat,
	) -> Self {
		Self::from_str::<V>(
			shader_code,
			Some(I::desc()),
			label,
			bindings,
			device,
			swapchain_texture_view_format,
		)
	}

	pub fn new(
		source: ShaderSource,
		vertex_layout: VertexBufferLayout<'static>,
		instance_layout: Option<VertexBufferLayout<'static>>,
		label: Option<&str>,
		bindings: &[Option<&BindGroupLayout>],
		device: &Device,
		swapchain_texture_view_format: TextureFormat,
	) -> Self {
		assert!(
			swapchain_texture_view_format.is_srgb(),
			"the texture view of the swapchain texture should always be srgb, we dont apply gamma correction ourselfs"
		);

		let pipeline = create_pipeline(
			source,
			vertex_layout,
			instance_layout,
			label,
			bindings,
			device,
			swapchain_texture_view_format,
		);

		Self { pipeline }
	}

	pub fn bind<'a>(&'a self, renderpass: &'_ mut RenderPass<'a>) {
		renderpass.set_pipeline(&self.pipeline);
	}
}
