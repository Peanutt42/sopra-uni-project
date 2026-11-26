#![allow(clippy::not_unsafe_ptr_arg_deref)]

use cgmath::{Deg, Point3};
use sphynx_input::Input;
use sphynx_rendering::{Camera, CameraUniform, Renderer};

pub mod hot_reload_api;

mod screens;
pub use screens::Screen;

pub mod ui;

mod camera_controller;
pub use camera_controller::CameraController;

/// Just to verify if hot-reload actually happened
pub const VERSION: u64 = 1;

/// Stores the games state
pub struct Game {
	pub screen: Screen,
	pub world: hecs::World,
	pub spawn_counter: u32,
	pub camera_controller: CameraController,
	pub camera: Camera,
}

/// Represents the serialized game state needed when hot-reloading (different from the actual game
/// saving mechanic!)
pub struct SerializedGame {
	pub spawn_counter: u32,
	pub world_snapshot: sphynx_hecs_component::serialization::Snapshot,
	pub serialized_screen: serde_json::Value,
	pub camera_controller: CameraController,
	pub camera: Camera,
}

pub fn game_create() -> Game {
	let world = hecs::World::new();

	Game {
		world,
		spawn_counter: 0,
		screen: Screen::default(),
		camera: Camera::new(
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
	game.camera_controller.update(input, &mut game.camera, dt);

	let mut continue_running = true;
	if let Some(new_screen) = game.screen.update(&mut continue_running, input, dt, frame) {
		game.screen = new_screen;
	}
	continue_running
}

pub fn game_render(game: &Game, renderer: &mut Renderer) {
	renderer.camera_uniform = CameraUniform::from_camera(
		&game.camera,
		Camera::get_aspect_from_swapchain(&renderer.swapchain_config),
	);

	game.screen.render(renderer);
}

pub fn game_egui_render(game: &mut Game, ctx: &egui::Context) {
	game.screen.egui_render(ctx);
}
