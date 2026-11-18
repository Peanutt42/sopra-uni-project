use crate::depth_texture::DepthTexture;
use wgpu::{
	BindGroupLayout, CompareFunction, DepthBiasState, DepthStencilState, Device, Face,
	FragmentState, MultisampleState, PipelineCompilationOptions, PipelineLayoutDescriptor,
	PrimitiveState, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource,
	StencilState, TextureFormat, VertexBufferLayout, VertexState,
};

// suppress cargo unused, since hot-reload on wasm is not possible
#[cfg(all(feature = "hot-reload", target_arch = "wasm32"))]
use sphynx_wesl as _;

#[cfg(not(all(feature = "hot-reload", not(target_arch = "wasm32"))))]
pub mod static_inline;
#[cfg(not(all(feature = "hot-reload", not(target_arch = "wasm32"))))]
pub use static_inline::Shader;

#[cfg(all(feature = "hot-reload", not(target_arch = "wasm32")))]
pub mod hotreload;
#[cfg(all(feature = "hot-reload", not(target_arch = "wasm32")))]
pub use hotreload::Shader;

pub const VERTEX_BUFFER_BIND_SLOT: u32 = 0;
pub const INSTANCE_BUFFER_BIND_SLOT: u32 = 1;
pub const CAMERA_UNIFORM_BIND_GROUP: u32 = 0;
pub const LIGHT_UNIFORM_BIND_GROUP: u32 = 1;

/// creates a shader pipeline based on shader source code.
/// this is used by both static_inline and hotreload
pub(crate) fn create_pipeline(
	source: ShaderSource,
	vertex_layout: VertexBufferLayout<'static>,
	instance_layout: Option<VertexBufferLayout<'static>>,
	label: Option<&str>,
	bindings: &[Option<&BindGroupLayout>],
	device: &Device,
	swapchain_texture_view_format: TextureFormat,
) -> RenderPipeline {
	let shader_module = device.create_shader_module(ShaderModuleDescriptor { label, source });

	let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
		label: None,
		bind_group_layouts: bindings,
		immediate_size: 0,
	});

	let mut vertex_buffers = vec![Some(vertex_layout)];
	if let Some(instance_layout) = instance_layout {
		vertex_buffers.push(Some(instance_layout));
	}

	device.create_render_pipeline(&RenderPipelineDescriptor {
		label,
		layout: Some(&pipeline_layout),
		vertex: VertexState {
			module: &shader_module,
			entry_point: Some("vs_main"),
			buffers: &vertex_buffers,
			compilation_options: PipelineCompilationOptions::default(),
		},
		fragment: Some(FragmentState {
			module: &shader_module,
			entry_point: Some("fs_main"),
			targets: &[Some(swapchain_texture_view_format.into())],
			compilation_options: PipelineCompilationOptions::default(),
		}),
		primitive: PrimitiveState {
			cull_mode: Some(Face::Back),
			..Default::default()
		},
		depth_stencil: Some(DepthStencilState {
			format: DepthTexture::FORMAT,
			depth_write_enabled: Some(true),
			depth_compare: Some(CompareFunction::Less),
			stencil: StencilState::default(),
			bias: DepthBiasState::default(),
		}),
		multisample: MultisampleState::default(),
		multiview_mask: None,
		cache: None,
	})
}
