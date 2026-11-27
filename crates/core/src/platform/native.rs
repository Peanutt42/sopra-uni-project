use crate::{Engine, EngineError};
use sphynx_rendering::{CreateRendererError, CreateRendererResult, Renderer};
use std::sync::Arc;
use winit::{
	event_loop::EventLoop,
	window::{Fullscreen, Window, WindowAttributes},
};

/// starts the game's event and blocks on it till its done
pub fn run<G: 'static, S: 'static>(
	engine: Engine<G, S>,
	event_loop: EventLoop<()>,
) -> Result<(), EngineError> {
	let mut engine = engine;
	event_loop.run_app(&mut engine)?;

	Ok(())
}

pub fn configure_window(attributes: WindowAttributes, fullscreen: bool) -> WindowAttributes {
	attributes.with_fullscreen(if fullscreen {
		Some(Fullscreen::Borderless(None))
	} else {
		None
	})
}

/// holds reference to the result of the async task that creates the renderer
pub struct RendererInit {
	result: Option<Result<Renderer, CreateRendererError>>,
}
impl RendererInit {
	/// creates the renderer blocking (this is just for a shared api between native and web platform)
	pub fn start(window: Arc<Window>, vsync: bool) -> Self {
		Self {
			result: Some(pollster::block_on(Renderer::new(window, vsync))),
		}
	}
	/// immediately returns the renderer result and also removes the result stored in [self]
	pub fn poll(&mut self) -> Option<CreateRendererResult> {
		self.result.take()
	}
}
