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
mod transfrom;
mod uniform_buffer;
mod vertex;

pub use camera::Camera;
pub use mesh::Mesh;
pub use model::load_obj_model;
pub use renderer::{CreateRendererError, CreateRendererResult, Renderer};
pub use shader::Shader;
pub use transfrom::Transform;

#[cfg(all(feature = "hot-reload", target_arch = "wasm32"))]
compile_error!("The `hot-reload` feature is not supported on web (WASM).");
