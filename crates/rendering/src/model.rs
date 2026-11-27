use crate::{Mesh, vertex::PCN_Vertex};
use cgmath::Vector3;
use std::io::BufRead;
use tobj::{LoadError, LoadOptions};
use wgpu::Device;

/// Local-space bounds of a model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelBounds {
	pub min: Vector3<f32>,
	pub max: Vector3<f32>,
}
impl ModelBounds {
	pub fn size(&self) -> Vector3<f32> {
		self.max - self.min
	}

	pub fn center(&self) -> Vector3<f32> {
		(self.min + self.max) * 0.5
	}
}

/// includes the model file while compiling (currently only obj file format)
#[macro_export]
macro_rules! include_model {
	($filepath:expr, $device:expr) => {{ load_obj_model(include_bytes!($filepath) as &[u8], $device) }};
}

/// includes a model with its X/Z bounds centered on the origin (like `include_model!`)
#[macro_export]
macro_rules! include_centered_model {
	($filepath:expr, $device:expr) => {{ load_centered_obj_model(include_bytes!($filepath) as &[u8], $device) }};
}

fn parse_obj_vertices<B: BufRead>(
	mut buf_reader: B,
) -> Result<(Vec<PCN_Vertex>, ModelBounds), LoadError> {
	let (models, _materials) = tobj::load_obj_buf(
		&mut buf_reader,
		&LoadOptions {
			triangulate: true,
			single_index: true, // (pos/normal/uv unified)
			..Default::default()
		},
		|_mtl_path| Ok(Default::default()), // TODO: handle the mtl files
	)?;

	let mut vertices: Vec<PCN_Vertex> = Vec::new();
	let mut min = Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
	let mut max = Vector3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);

	for model in &models {
		let mesh = &model.mesh;
		let has_normals = !mesh.normals.is_empty();

		for &idx in &mesh.indices {
			let i = idx as usize;
			let position = [
				mesh.positions[i * 3],
				mesh.positions[i * 3 + 1],
				mesh.positions[i * 3 + 2],
			];
			for (axis, &component) in position.iter().enumerate() {
				min[axis] = min[axis].min(component);
				max[axis] = max[axis].max(component);
			}

			let normal = if has_normals {
				[
					mesh.normals[i * 3],
					mesh.normals[i * 3 + 1],
					mesh.normals[i * 3 + 2],
				]
			} else {
				[0.0, 0.0, 0.0]
			};

			vertices.push(PCN_Vertex {
				position,
				color: [1.0, 1.0, 1.0],
				normal,
			});
		}
	}

	if vertices.is_empty() {
		min = Vector3::new(0.0, 0.0, 0.0);
		max = Vector3::new(0.0, 0.0, 0.0);
	}

	Ok((vertices, ModelBounds { min, max }))
}

pub fn load_obj_model<B>(buf_reader: B, device: &Device) -> Result<Mesh, LoadError>
where
	B: BufRead,
{
	let (vertices, _) = parse_obj_vertices(buf_reader)?;
	let indices: Vec<u32> = (0..vertices.len() as u32).collect();

	Ok(Mesh::with_vertices(&vertices, Some(&indices), device))
}

/// Centers X/Z without touching Y.
fn center_xz(vertices: &mut [PCN_Vertex], bounds: &mut ModelBounds) {
	let center = bounds.center();

	for vertex in vertices {
		vertex.position[0] -= center.x;
		vertex.position[2] -= center.z;
	}
	bounds.min.x -= center.x;
	bounds.max.x -= center.x;
	bounds.min.z -= center.z;
	bounds.max.z -= center.z;
}

/// Loads an X/Z-centered model and returns its bounds.
pub fn load_centered_obj_model<B>(
	buf_reader: B,
	device: &Device,
) -> Result<(Mesh, ModelBounds), LoadError>
where
	B: BufRead,
{
	let (mut vertices, mut bounds) = parse_obj_vertices(buf_reader)?;
	center_xz(&mut vertices, &mut bounds);

	let indices: Vec<u32> = (0..vertices.len() as u32).collect();
	let mesh = Mesh::with_vertices(&vertices, Some(&indices), device);

	Ok((mesh, bounds))
}
