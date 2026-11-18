use sphynx_input::Input;
use sphynx_logging::*;
use sphynx_rendering::{CreateRendererError, Renderer};
use std::sync::Arc;
use thiserror::Error;
use web_time::Instant;
use winit::{
	application::ApplicationHandler,
	dpi::{PhysicalPosition, PhysicalSize},
	event::{DeviceEvent, MouseButton, WindowEvent},
	event_loop::{ActiveEventLoop, EventLoop},
	keyboard::KeyCode,
	window::{CursorGrabMode, Fullscreen, Window, WindowAttributes},
};

mod game;
pub use game::GameApi;

mod platform;

/// all errors the engine can have (mainly while starting)
#[derive(Debug, Error)]
pub enum EngineError {
	#[error("failed to create renderer: {0}")]
	CreateRenderer(#[from] CreateRendererError),
	#[error(transparent)]
	Winit(#[from] winit::error::EventLoopError),
}

#[derive(Debug, Default)]
pub struct EngineConfig {
	pub fullscreen: bool,
	pub vsync: bool,
	pub on_exiting_callback: Option<fn()>,
}
impl EngineConfig {
	pub fn new(fullscreen: bool, vsync: bool) -> Self {
		Self {
			fullscreen,
			vsync,
			on_exiting_callback: None,
		}
	}

	pub fn new_with_exiting_callback(
		fullscreen: bool,
		vsync: bool,
		on_exiting_callback: fn(),
	) -> Self {
		Self {
			fullscreen,
			vsync,
			on_exiting_callback: Some(on_exiting_callback),
		}
	}

	/// runs the engine with [self] configuration
	pub fn run<G: 'static, S: 'static>(
		self,
		game_api: GameApi<G, S>,
		#[cfg(feature = "hot-reload")] hot_reload_observer: hot_lib_reloader::LibReloadObserver,
	) -> Result<(), EngineError> {
		let engine = Engine::<G, S>::new(
			self,
			game_api,
			#[cfg(feature = "hot-reload")]
			hot_reload_observer,
		)?;
		let event_loop = EventLoop::new()?;
		engine.run(event_loop)
	}
}

/// mega object of everything the engine owns
pub(crate) struct Engine<G, S> {
	config: EngineConfig,
	/// None while the window has not yet been created
	window: Option<Arc<Window>>,
	/// None until the renderer has finished initializing
	renderer: Option<Renderer>,
	/// Renderer creation can be asynchronous (web); the event loop collects it
	/// here and moves it into `renderer` once ready.
	renderer_init: Option<platform::RendererInit>,
	input: Input,
	last_update_time: Instant,
	frame_counter: u64,
	restore_cursor_position: PhysicalPosition<f64>, // where should the captures cursor be restore to
	game_api: GameApi<G, S>,
	/// this is a mut ptr since it should support hot-reloading the game code -> memory layout shifts
	game_ptr: *mut G,
	#[cfg(feature = "hot-reload")]
	hot_reload_observer: hot_lib_reloader::LibReloadObserver,
	#[cfg(any(debug_assertions, feature = "debug"))]
	inspector_state: sphynx_egui::InspectorState,
	#[cfg(any(debug_assertions, feature = "debug"))]
	show_inspector: bool,
}

#[cfg(debug_assertions)]
const BUILD_CONFIG: &str = "Debug";

#[cfg(all(not(debug_assertions), feature = "debug"))]
const BUILD_CONFIG: &str = "Release (with Debug)";

#[cfg(all(not(debug_assertions), not(feature = "debug")))]
const BUILD_CONFIG: &str = "Release";

impl<G: 'static, S: 'static> Engine<G, S> {
	fn new(
		config: EngineConfig,
		game_api: GameApi<G, S>,
		#[cfg(feature = "hot-reload")] hot_reload_observer: hot_lib_reloader::LibReloadObserver,
	) -> Result<Self, EngineError> {
		info!(General, "=== INITIALIZING ===");

		let game_ptr = (game_api.create)();

		Ok(Self {
			config,
			window: None,
			renderer: None,
			renderer_init: None,
			input: Input::new(),
			last_update_time: Instant::now(),
			frame_counter: 0,
			restore_cursor_position: PhysicalPosition::new(0.0, 0.0),
			game_ptr,
			game_api,
			#[cfg(feature = "hot-reload")]
			hot_reload_observer,
			#[cfg(any(debug_assertions, feature = "debug"))]
			inspector_state: sphynx_egui::InspectorState::default(),
			#[cfg(any(debug_assertions, feature = "debug"))]
			show_inspector: false,
		})
	}

	/// runs the event loop with [self] as the [ApplicationHandler]
	pub fn run(self, event_loop: EventLoop<()>) -> Result<(), EngineError> {
		platform::run(self, event_loop)
	}

	/// polls the RendererInit if it has finished initializing the renderer
	fn poll_renderer_init(&mut self) {
		let Some(renderer_init) = self.renderer_init.as_mut() else {
			return;
		};

		let Some(result) = renderer_init.poll() else {
			return;
		};

		self.renderer_init = None;

		match result {
			Ok(renderer) => {
				info!(Rendering, "=== RENDERER READY ===");
				self.last_update_time = Instant::now();
				self.renderer = Some(renderer);
			}
			Err(error) => {
				error!(Rendering, "failed to initialize the renderer: {error:#}");
				std::process::exit(1);
			}
		}
	}

	fn window_attributes(&self) -> WindowAttributes {
		platform::configure_window(
			Window::default_attributes()
				.with_title(format!("SphynxEngine [{}]", BUILD_CONFIG))
				.with_inner_size(PhysicalSize::new(1920.0, 1080.0)),
			self.config.fullscreen,
		)
	}

	/// updates all subsystems and renders one frame
	fn update(&mut self, event_loop: &ActiveEventLoop) {
		let window = self
			.window
			.as_mut()
			.expect("Window should have been created by now");
		let renderer = self
			.renderer
			.as_mut()
			.expect("Renderer should have been created by now");

		let now = Instant::now();
		let delta_time = (now - self.last_update_time).as_secs_f32();
		self.last_update_time = now;

		if self.input.was_key_pressed(KeyCode::F11) {
			if self.config.fullscreen {
				window.set_fullscreen(None);
				self.config.fullscreen = false;
			} else {
				window.set_fullscreen(Some(Fullscreen::Borderless(None)));
				self.config.fullscreen = true;
			}
		}

		// captured cursor
		if self.input.was_mouse_button_pressed(MouseButton::Right) {
			self.restore_cursor_position = self.input.cursor_position;
			let _ = window.set_cursor_grab(CursorGrabMode::Locked);
		} else if self.input.was_mouse_button_released(MouseButton::Right) {
			let _ = window.set_cursor_grab(CursorGrabMode::None);
			let _ = window.set_cursor_position(self.restore_cursor_position);
		}
		window.set_cursor_visible(!self.input.is_mouse_button_down(MouseButton::Right));

		#[cfg(feature = "hot-reload")]
		game::try_reload_game(
			&self.hot_reload_observer,
			&self.game_api,
			&mut self.game_ptr,
		);

		let continue_running = unsafe {
			(self.game_api.update)(self.game_ptr, &self.input, delta_time, self.frame_counter)
		};

		if !continue_running {
			event_loop.exit();
		}

		unsafe {
			(self.game_api.render)(self.game_ptr, renderer);
		}

		#[cfg(any(debug_assertions, feature = "debug"))]
		let (world_ptr, component_types) = (
			unsafe { (self.game_api.get_hecs_world)(self.game_ptr) },
			(self.game_api.get_component_types)(),
		);

		renderer.update(|ctx| {
			#[cfg(any(debug_assertions, feature = "debug"))]
			{
				sphynx_egui::debug_window(
					ctx,
					delta_time,
					(self.game_api.version)(),
					&mut self.show_inspector,
				);

				if self.show_inspector && !world_ptr.is_null() {
					sphynx_egui::entity_inspector(
						ctx,
						unsafe { &mut *world_ptr },
						&component_types,
						&mut self.inspector_state,
					);
				}
			}

			unsafe { (self.game_api.egui_render)(self.game_ptr, ctx) };
		});

		self.input.clear_frame();

		self.frame_counter += 1;
	}
}

impl<G: 'static, S: 'static> ApplicationHandler for Engine<G, S> {
	fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
		let window = Arc::new(
			event_loop
				.create_window(self.window_attributes())
				.expect("failed to create window"),
		);

		self.window = Some(Arc::clone(&window));
		self.renderer_init = Some(platform::RendererInit::start(window, self.config.vsync));

		// on native, this runs blocking, on web, this starts a async task
		self.poll_renderer_init();
	}

	fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
		self.poll_renderer_init();

		if let Some(window) = self.window.as_ref() {
			window.request_redraw();
		}
	}

	fn window_event(
		&mut self,
		event_loop: &winit::event_loop::ActiveEventLoop,
		_window_id: winit::window::WindowId,
		event: WindowEvent,
	) {
		#[allow(unused)]
		let Some(window) = self.window.as_ref() else {
			return;
		};
		let Some(renderer) = self.renderer.as_mut() else {
			return;
		};

		renderer.egui_renderer.handle_input(window.as_ref(), &event);

		match event {
			WindowEvent::Resized(new_size) => {
				renderer.resize(new_size);
			}
			WindowEvent::RedrawRequested => {
				self.update(event_loop);
			}
			WindowEvent::CloseRequested => event_loop.exit(),

			// INPUT
			WindowEvent::KeyboardInput { event, .. } => self.input.handle_keyboard(event),
			WindowEvent::MouseInput { button, state, .. } => self.input.handle_mouse(button, state),
			WindowEvent::CursorMoved { position, .. } => {
				self.input.handle_cursor_movement(position)
			}
			_ => {}
		}
	}

	fn device_event(
		&mut self,
		_event_loop: &ActiveEventLoop,
		_device_id: winit::event::DeviceId,
		event: DeviceEvent,
	) {
		if let DeviceEvent::MouseMotion { delta } = event {
			self.input.handle_mouse_movement(delta);
		}
	}

	fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
		info!(General, "=== SHUTDOWN ===");

		if let Some(on_exiting_callback) = self.config.on_exiting_callback {
			on_exiting_callback();
		}
	}
}
impl<G, S> Drop for Engine<G, S> {
	fn drop(&mut self) {
		unsafe { (self.game_api.destroy)(self.game_ptr) };
		self.game_ptr = std::ptr::null_mut();
	}
}
