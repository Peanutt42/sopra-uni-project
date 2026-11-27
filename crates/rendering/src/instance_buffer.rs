use crate::instance_data::InstanceData;
use crate::shader::INSTANCE_BUFFER_BIND_SLOT;
use wgpu::{
	Buffer, BufferDescriptor, BufferUsages, Device, Queue, RenderPass,
	util::{BufferInitDescriptor, DeviceExt},
};

/// stores both the cpu memory instance data as well as the handle to the gpu buffer
pub struct InstanceBuffer<I: InstanceData> {
	pub instances: Vec<I>,
	buffer: Buffer,
}

impl<I: InstanceData> InstanceBuffer<I> {
	pub fn new(instances: Vec<I>, device: &Device) -> Self {
		let buffer = device.create_buffer_init(&BufferInitDescriptor {
			label: Some("Instance Buffer"),
			contents: bytemuck::cast_slice(&instances),
			usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
		});

		Self { instances, buffer }
	}

	/// Uploads the current instances, growing the GPU buffer when needed.
	pub fn update(&mut self, queue: &Queue, device: &Device) {
		let required_bytes = (self.instances.len() * std::mem::size_of::<I>()) as u64;

		if required_bytes > self.buffer.size() {
			let capacity = required_bytes
				.max(self.buffer.size().saturating_mul(2))
				.max(std::mem::size_of::<I>() as u64);
			self.buffer = device.create_buffer(&BufferDescriptor {
				label: Some("Instance Buffer"),
				size: capacity,
				usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
				mapped_at_creation: false,
			});
		}

		if required_bytes > 0 {
			queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.instances));
		}
	}

	/// binds the instance buffer to a shader
	pub fn bind<'a>(&'a self, renderpass: &'_ mut RenderPass<'a>) {
		renderpass.set_vertex_buffer(INSTANCE_BUFFER_BIND_SLOT, self.buffer.slice(..));
	}
}
