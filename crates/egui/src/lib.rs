//! Mostly copied from <https://github.com/kaphula/winit-egui-wgpu-template/blob/5b5ffc9eaf561e688f7864a9f79c8c33da7487f5/src/egui_tools.rs>
//! Since this "template" is not really up to date with the newest egui version,
//! the TexturesDelta uploading/painting was copied from
//! <https://github.com/emilk/egui/blob/86099aca1276fe786921992c8d5a68914ee79bec/crates/egui-wgpu/src/winit.rs#L568-L578>

/// reexport
pub use egui;

mod renderer;
pub use renderer::EguiRenderer;

#[cfg(any(debug_assertions, feature = "debug"))]
mod debug_window;
#[cfg(any(debug_assertions, feature = "debug"))]
pub use debug_window::debug_window;

#[cfg(any(debug_assertions, feature = "debug"))]
mod entity_inspector;
#[cfg(any(debug_assertions, feature = "debug"))]
pub use entity_inspector::{InspectorState, entity_inspector};
