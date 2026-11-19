// engine knows this, no need for many duplicate # Safety docs
#![allow(clippy::missing_safety_doc)]

use crate::{Game, Screen, SerializedGame};
use sphynx_input::Input;
use sphynx_logging::error;
use sphynx_rendering::Renderer;

/// creates external game structure
#[cfg_attr(feature = "hot-reload", unsafe(no_mangle))]
pub fn sphynx_engine_game_create() -> *mut Game {
	Box::into_raw(Box::new(crate::game_create()))
}

/// deletes external game structure
#[cfg_attr(feature = "hot-reload", unsafe(no_mangle))]
pub unsafe fn sphynx_engine_game_destroy(game: *mut Game) {
	if !game.is_null() {
		drop(unsafe { Box::from_raw(game) });
	}
}

#[cfg_attr(feature = "hot-reload", unsafe(no_mangle))]
pub unsafe fn sphynx_engine_game_update(
	game: *mut Game,
	input: &Input,
	dt: f32,
	frame: u64,
) -> bool {
	assert!(!game.is_null());

	let game = unsafe { &mut *game };

	crate::game_update(game, input, dt, frame)
}

#[cfg_attr(feature = "hot-reload", unsafe(no_mangle))]
pub unsafe fn sphynx_engine_game_render(game: *const Game, renderer: &mut Renderer) {
	if game.is_null() {
		return;
	}

	let game = unsafe { &*game };

	crate::game_render(game, renderer);
}

#[cfg_attr(feature = "hot-reload", unsafe(no_mangle))]
pub unsafe fn sphynx_engine_game_egui_render(game: *mut Game, ctx: &egui::Context) {
	if game.is_null() {
		return;
	}

	let game = unsafe { &mut *game };

	crate::game_egui_render(game, ctx);
}

/// gets run before hot-reloading
#[cfg_attr(feature = "hot-reload", unsafe(no_mangle))]
pub unsafe fn sphynx_engine_game_serialize(game: *const Game) -> SerializedGame {
	if game.is_null() {
		panic!("game is null");
	}
	let game = unsafe { &*game };

	let serialized_screen = match serde_json::to_value(&game.screen) {
		Ok(serialized_screen) => serialized_screen,
		Err(err) => {
			panic!("[GAME] failed to serialize screen state before reload: {err}");
		}
	};

	match sphynx_hecs_component::serialization::serialize(&game.world) {
		Ok(world_snapshot) => SerializedGame {
			spawn_counter: game.spawn_counter,
			world_snapshot,
			serialized_screen,
			camera: game.camera.clone(),
			camera_controller: game.camera_controller,
		},
		Err(err) => {
			panic!("[GAME] failed to serialize world before reload: {err}");
		}
	}
}

/// gets run after hot-reloading, restoring the state before the hot-reload
#[cfg_attr(feature = "hot-reload", unsafe(no_mangle))]
pub fn sphynx_engine_game_deserialize(serialized_game: SerializedGame) -> *mut Game {
	let screen = match serde_json::from_value(serialized_game.serialized_screen) {
		Ok(screen) => screen,
		Err(e) => {
			error!(
				Game,
				"failed to deserialize game state from before hot reloading, \
					creating default StartScreen from scratch: {e}"
			);
			Screen::default()
		}
	};

	let game =
		match sphynx_hecs_component::serialization::deserialize(serialized_game.world_snapshot) {
			Ok(world) => Game {
				world,
				spawn_counter: serialized_game.spawn_counter,
				screen,
				camera: serialized_game.camera,
				camera_controller: serialized_game.camera_controller,
			},
			Err(e) => {
				error!(
					Game,
					"failed to deserialize game snapshot from before hot reloading, \
					recreating game from scratch: {e}"
				);
				crate::game_create()
			}
		};

	Box::into_raw(Box::new(game))
}

/// simply for verifying if the hot-reload was successfull
#[cfg_attr(feature = "hot-reload", unsafe(no_mangle))]
pub fn sphynx_engine_game_version() -> u64 {
	crate::VERSION
}

/// only used for the egui entity inspector debug window
#[cfg_attr(feature = "hot-reload", unsafe(no_mangle))]
pub unsafe fn sphynx_engine_game_get_hecs_world(game: *mut Game) -> *mut hecs::World {
	if game.is_null() {
		panic!("game is null");
	}
	let game = unsafe { &mut *game };
	&mut game.world
}

/// only used for the egui entity inspector debug window
#[cfg_attr(feature = "hot-reload", unsafe(no_mangle))]
pub fn sphynx_engine_game_get_component_types() -> Vec<&'static sphynx_hecs_component::ComponentType>
{
	sphynx_hecs_component::registered_component_types()
		.into_iter()
		.collect()
}
