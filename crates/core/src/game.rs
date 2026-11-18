use sphynx_input::Input;
use sphynx_rendering::Renderer;

/// vtable of function ptrs for interacting with the game library without linking to the games
/// functions directly in order to support hot-reloading the game library.
pub struct GameApi<Game, SerializedGame> {
	pub create: fn() -> *mut Game,
	pub destroy: unsafe fn(game: *mut Game),
	/// returns whether the game engine should continue running or shutdown
	pub update: unsafe fn(game: *mut Game, input: &Input, dt: f32, frame: u64) -> bool,
	pub render: unsafe fn(game: *const Game, renderer: &mut Renderer),
	pub egui_render: unsafe fn(game: *mut Game, ctx: &egui::Context),
	pub serialize: unsafe fn(game: *const Game) -> SerializedGame,
	pub deserialize: fn(serialized_game: SerializedGame) -> *mut Game,
	pub version: fn() -> u64,
	/// readonly for debugging using egui entity inspector
	pub get_hecs_world: unsafe fn(game: *mut Game) -> *mut hecs::World,
	pub get_component_types: fn() -> Vec<&'static sphynx_hecs_component::ComponentType>,
}

/// checks if the game library has been updated and hot-reloads by serializing a snapshot of the
/// game state, load the new library and deserializing from the snapshot.
#[cfg(feature = "hot-reload")]
pub fn try_reload_game<Game, SerializedGame>(
	observer: &hot_lib_reloader::LibReloadObserver,
	game_api: &GameApi<Game, SerializedGame>,
	game: &mut *mut Game,
) {
	use std::time::Duration;

	fn time<R>(lambda: impl FnOnce() -> R) -> (std::time::Duration, R) {
		let start = std::time::Instant::now();
		let result = lambda();
		(start.elapsed(), result)
	}

	let Some(blocker) = observer.wait_for_about_to_reload_timeout(Duration::ZERO) else {
		return;
	};

	let reload_start = std::time::Instant::now();
	println!("=== HOT-RELOAD STARTED ===");

	let (serialize_duration, serialized_game) = time(|| unsafe { (game_api.serialize)(*game) });

	let (destroy_duration, _) = time(|| {
		unsafe { (game_api.destroy)(*game) };
		*game = std::ptr::null_mut();
	});

	let (reload_lib_duration, _) = time(|| {
		// allow the reload to proceed and wait for the new library.
		drop(blocker);
		observer.wait_for_reload();
	});

	let (deserialize_duration, new_game) = time(|| (game_api.deserialize)(serialized_game));
	*game = new_game;

	let total_duration = reload_start.elapsed();
	println!(
		"=== HOT-RELOADED FINISHED (total took {total_duration:?}) ===\n game_serialize took {serialize_duration:?}\n game_destroy took {destroy_duration:?}\n lib reloading took {reload_lib_duration:?}\n game_deserialize took {deserialize_duration:?}"
	);
}
