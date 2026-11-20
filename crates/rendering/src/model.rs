use crate::{Mesh, vertex::PCN_Vertex};
use std::io::BufRead;
use tobj::{LoadError, LoadOptions};
use wgpu::Device;

/// includes the model file while compiling (currently only obj file format)
#[macro_export]
macro_rules! include_model {
	($filepath:expr, $device:expr) => {{ load_obj_model(include_bytes!($filepath) as &[u8], $device) }};
}

pub fn load_obj_model<B>(mut buf_reader: B, device: &Device) -> Result<Mesh, LoadError>
where
	B: BufRead,
{
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

	for model in &models {
		let mesh = &model.mesh;
		let has_normals = !mesh.normals.is_empty();

		for &idx in &mesh.indices {
			let i = idx as usize;
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
				position: [
					mesh.positions[i * 3],
					mesh.positions[i * 3 + 1],
					mesh.positions[i * 3 + 2],
				],
				color: [1.0, 1.0, 1.0],
				normal,
			});
		}
	}

	let indices: Vec<u32> = (0..vertices.len() as u32).collect();

	Ok(Mesh::with_vertices(&vertices, Some(&indices), device))
}
