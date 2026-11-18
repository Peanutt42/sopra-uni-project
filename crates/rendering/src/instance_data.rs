use cgmath::{Matrix3, Matrix4};
use sphynx_derive_vertex_attrib::InstanceData;
use wgpu::{VertexAttribute, VertexBufferLayout, vertex_attr_array};

/// trait for all InstanceData types: must be zero-copyable to gpu memory and return their
/// [wgpu::VertexBufferLayout]
pub trait InstanceData: bytemuck::Pod + bytemuck::Zeroable {
	/// returns the [wgpu::VertexBufferLayout]
	fn desc() -> VertexBufferLayout<'static>;
}

/// GPU memory buffer layout of a single instances data: both model matrix as well as rotation matrix
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, InstanceData)]
#[allow(non_camel_case_types)]
pub struct Model_InstanceData {
	pub model: [[f32; 4]; 4],
	pub rotation: [[f32; 3]; 3],
}

impl Model_InstanceData {
	pub fn new(model: Matrix4<f32>, rotation: Matrix3<f32>) -> Self {
		Self {
			model: model.into(),
			rotation: rotation.into(),
		}
	}

	const ATTRIBS: [VertexAttribute; 7] = vertex_attr_array![
		5  => Float32x4,
		6  => Float32x4,
		7  => Float32x4,
		8  => Float32x4,
		9  => Float32x3,
		10 => Float32x3,
		11 => Float32x3
	];
}
