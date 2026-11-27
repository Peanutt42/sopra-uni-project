#![allow(clippy::not_unsafe_ptr_arg_deref)]

use cgmath::{Deg, Point3};
use sphynx_input::Input;
use sphynx_rendering::{Camera, CameraUniform, Renderer};

pub mod hot_reload_api;

mod screens;
pub use screens::Screen;

pub mod ui;

mod board_layout;

mod ship_render;

mod game_phase;
pub use game_phase::GamePhase;

mod camera_controller;
pub use camera_controller::CameraController;

mod config;
pub use config::{Config, FieldSize, ShipConfig};

mod ship;
pub use ship::{Orientation, Position, Ship, ShipType};

mod board_state;
pub use board_state::BoardState;

mod player;
pub use player::Player;

/// Just to verify if hot-reload actually happened
pub const VERSION: u64 = 1;

/// Stores the games state
pub struct Game {
	pub config: Config,
	pub screen: Screen,
	pub world: hecs::World,
	pub spawn_counter: u32,
	pub camera_controller: CameraController,
	pub camera: Camera,
	pub start_camera: Camera,
}

/// Represents the serialized game state needed when hot-reloading (different from the actual game
/// saving mechanic!)
pub struct SerializedGame {
	pub config: Config,
	pub spawn_counter: u32,
	pub world_snapshot: sphynx_hecs_component::serialization::Snapshot,
	pub serialized_screen: serde_json::Value,
	pub camera_controller: CameraController,
	pub camera: Camera,
	pub start_camera: Camera,
}
impl Game {
	fn active_camera(&self, aspect: f32) -> Camera {
		match &self.screen {
			Screen::Start(_) => self.start_camera,
			Screen::Game(screen) => self
				.camera
				.with_view_size(screen.view_size(&self.config, aspect)),
		}
	}
}

pub fn game_create() -> Game {
	let config = Config::load();

	let world = hecs::World::new();

	Game {
		config,
		world,
		spawn_counter: 0,
		screen: Screen::default(),
		camera: Camera::orthographic(
			Point3::new(0.0, 20.0, 0.0),
			Deg(-90.0).into(),
			Deg(-90.0).into(),
			55.0,
			0.0,
			1000.0,
		),
		start_camera: Camera::perspective(
			Point3::new(0.0, 5.0, 20.0),
			Deg(-90.0).into(),
			Deg(0.0).into(),
			90.0,
			0.1,
			1000.0,
		),
		camera_controller: CameraController::new(4.0, 0.005),
	}
}

/// returns whether the game should continue running
pub fn game_update(game: &mut Game, input: &Input, dt: f32, frame: u64) -> bool {
	// Free-fly only applies to the start camera; the board camera stays top-down.
	if matches!(game.screen, Screen::Start(_)) {
		game.camera_controller
			.update(input, &mut game.start_camera, dt);
	}

	let mut continue_running = true;
	if let Some(new_screen) = game.screen.update(&mut continue_running, input, dt, frame) {
		game.screen = new_screen;
	}
	continue_running
}

pub fn game_render(game: &Game, renderer: &mut Renderer) {
	let aspect = Camera::get_aspect_from_swapchain(&renderer.swapchain_config);
	renderer.camera_uniform = CameraUniform::from_camera(&game.active_camera(aspect), aspect);

	game.screen.render(renderer, &game.config);
}

pub fn game_egui_render(game: &mut Game, ctx: &egui::Context) {
	let viewport = ctx.viewport_rect();
	let aspect = Camera::get_aspect(viewport.width(), viewport.height());
	let camera = game.active_camera(aspect);

	game.screen.egui_render(ctx, &camera, &game.config);
}
