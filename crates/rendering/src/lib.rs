mod camera;
mod camera_uniform;
mod depth_texture;
mod instance_buffer;
mod instance_data;
mod lighting;
mod mesh;
mod model;
mod renderer;
mod shader;
mod transform;
mod uniform_buffer;
mod vertex;
mod water;

pub use camera::{Camera, Projection};
pub use camera_uniform::CameraUniform;
pub use lighting::LightUniform;
pub use mesh::Mesh;
pub use model::{ModelBounds, load_centered_obj_model, load_obj_model};
pub use renderer::{CreateRendererError, CreateRendererResult, Renderer};
pub use shader::Shader;
pub use transform::Transform;
pub use water::Water;

#[cfg(all(feature = "hot-reload", target_arch = "wasm32"))]
compile_error!("The `hot-reload` feature is not supported on web (WASM).");
