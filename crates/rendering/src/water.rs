use crate::{
	Mesh, Shader, include_shader, shader::WATER_TIME_BIND_GROUP, uniform_buffer::UniformBuffer,
	vertex::PCN_Vertex,
};
use wgpu::{BindGroupLayout, Device, Queue, RenderPass, TextureFormat};

const WATER_SIZE: f32 = 100.0;
const WATER_LEVEL: f32 = -1.0;
const WATER_SUBDIVISIONS: u32 = 32;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct TimeUniform {
	/// elapsed time in seconds.
	elapsed: f32,
	_padding: [f32; 3],
}

impl TimeUniform {
	fn new(elapsed: f32) -> Self {
		Self {
			elapsed,
			_padding: [0.0, 0.0, 0.0],
		}
	}
}

/// low-poly plane. vertices are offset in the water vertex shader pipeline
pub struct Water {
	mesh: Mesh,
	shader: Shader,
	time_uniform: UniformBuffer<TimeUniform>,
	elapsed: f32,
}

impl Water {
	pub fn new(
		device: &Device,
		swapchain_format: TextureFormat,
		camera_binding: &BindGroupLayout,
		light_binding: &BindGroupLayout,
	) -> Self {
		let mesh = create_water_mesh(device);
		let time_uniform = UniformBuffer::for_vertex(
			TimeUniform::new(0.0),
			Some("Water Time Uniform Buffer"),
			device,
		);

		let shader = include_shader!(
			"../../assets/shaders",
			"water",
			&[
				Some(camera_binding),
				Some(light_binding),
				Some(time_uniform.get_binding())
			],
			&device,
			swapchain_format
		);

		Self {
			mesh,
			shader,
			time_uniform,
			elapsed: 0.0,
		}
	}

	pub fn update(&mut self, delta_time: f32, queue: &Queue) {
		self.elapsed += delta_time;
		self.time_uniform
			.update(TimeUniform::new(self.elapsed), queue);
	}

	pub fn draw<'a>(&'a self, renderpass: &mut RenderPass<'a>) {
		self.time_uniform.bind(WATER_TIME_BIND_GROUP, renderpass);
		self.shader.bind(renderpass);
		self.mesh.draw(renderpass);
	}
}

fn create_water_mesh(device: &Device) -> Mesh {
	let step = WATER_SIZE / WATER_SUBDIVISIONS as f32;
	let half_size = WATER_SIZE * 0.5;

	let vertex = |x: f32, z: f32| PCN_Vertex {
		position: [x, WATER_LEVEL, z],
		color: [0.04, 0.25, 0.4],
		normal: [0.0, 1.0, 0.0],
	};

	let mut vertices = Vec::with_capacity((WATER_SUBDIVISIONS * WATER_SUBDIVISIONS * 6) as usize);

	for row in 0..WATER_SUBDIVISIONS {
		for column in 0..WATER_SUBDIVISIONS {
			let x0 = -half_size + column as f32 * step;
			let x1 = x0 + step;
			let z0 = -half_size + row as f32 * step;
			let z1 = z0 + step;

			// Two triangles per cell, wound counter-clockwise when seen from above.
			vertices.push(vertex(x0, z0));
			vertices.push(vertex(x0, z1));
			vertices.push(vertex(x1, z0));

			vertices.push(vertex(x1, z0));
			vertices.push(vertex(x0, z1));
			vertices.push(vertex(x1, z1));
		}
	}

	Mesh::with_vertices(&vertices, None, device)
}
