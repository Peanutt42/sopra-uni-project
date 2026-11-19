#[cfg(target_arch = "wasm32")]
mod browser {
	use game::{Game, SerializedGame};
	use sphynx_core::{EngineConfig, GameApi};
	use wasm_bindgen::prelude::*;

	/// entry point for the web (for native desktop platforms, see ../../../src/main.rs)
	#[wasm_bindgen(start)]
	pub fn start() -> Result<(), JsValue> {
		console_error_panic_hook::set_once();
		hide_loading_overlay();

		let game_api = GameApi::<Game, SerializedGame> {
			create: game::hot_reload_api::sphynx_engine_game_create,
			destroy: game::hot_reload_api::sphynx_engine_game_destroy,
			update: game::hot_reload_api::sphynx_engine_game_update,
			render: game::hot_reload_api::sphynx_engine_game_render,
			egui_render: game::hot_reload_api::sphynx_engine_game_egui_render,
			serialize: game::hot_reload_api::sphynx_engine_game_serialize,
			deserialize: game::hot_reload_api::sphynx_engine_game_deserialize,
			version: game::hot_reload_api::sphynx_engine_game_version,
			get_hecs_world: game::hot_reload_api::sphynx_engine_game_get_hecs_world,
			get_component_types: game::hot_reload_api::sphynx_engine_game_get_component_types,
		};

		EngineConfig::new_with_exiting_callback(false, true, on_exiting_callback)
			.run(game_api)
			.map_err(|error| JsValue::from_str(&format!("{error:#}")))
	}

	fn hide_loading_overlay() {
		if let Some(loading) = web_sys::window()
			.and_then(|window| window.document())
			.and_then(|document| document.get_element_by_id("loading"))
		{
			let _ = loading.set_attribute("hidden", "");
		}
	}

	fn on_exiting_callback() {
		if let Some(window) = web_sys::window() {
			let _ = window.close();
		}
	}
}
