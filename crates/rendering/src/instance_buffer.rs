use crate::instance_data::InstanceData;
use crate::shader::INSTANCE_BUFFER_BIND_SLOT;
use wgpu::{
	Buffer, BufferUsages, Device, Queue, RenderPass,
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

	/// only actually updates when the queue is submitted with [wgpu::Queue::submit]
	pub fn update(&self, queue: &Queue) {
		queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.instances));
	}

	/// binds the instance buffer to a shader
	pub fn bind<'a>(&'a self, renderpass: &'_ mut RenderPass<'a>) {
		renderpass.set_vertex_buffer(INSTANCE_BUFFER_BIND_SLOT, self.buffer.slice(..));
	}
}
