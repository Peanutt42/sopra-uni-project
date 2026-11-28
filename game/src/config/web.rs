use crate::Config;
use sphynx_logging::{debug, error};

const LOCAL_STORAGE_ITEM_KEY: &str = "config";

/// tries to load the configuration from localStorage, using default on failure
pub fn load() -> Config {
	match load_impl() {
		Ok(config) => {
			debug!(Game, "successfully loaded configuration");
			config
		}
		Err(e) => {
			error!(
				Game,
				"failed to load configuration: {e}. using default configuration instead"
			);
			let config = Config::default();
			save(&config);
			config
		}
	}
}

/// will log error on failure, as there is no good error handling for this case
pub fn save(config: &Config) {
	debug!(Game, "saving configuration to localStorage");
	match get_local_storage() {
		Ok(local_storage) => {
			let json = config.save_to_json().expect("should always serialize");
			match local_storage.set_item(LOCAL_STORAGE_ITEM_KEY, &json) {
				Ok(()) => debug!(Game, "successfully saved configuration to localStorage"),
				Err(e) => error!(
					Game,
					"failed to save configuration to localStorage: {}",
					js_value_to_string(&e)
				),
			}
		}
		Err(e) => error!(Game, "{e}"),
	}
}

fn get_local_storage() -> Result<web_sys::Storage, String> {
	web_sys::window()
		.expect("failed to get web window")
		.local_storage()
		.map_err(|e| js_value_to_string(&e))?
		.ok_or("no localStorage object found".to_string())
}

fn load_impl() -> Result<Config, String> {
	debug!(Game, "loading configuration file from localStorage");

	let local_storage = get_local_storage()?;

	match local_storage.get_item(LOCAL_STORAGE_ITEM_KEY) {
		Ok(Some(json)) => match Config::load_from_json(&json) {
			Ok(config) => match config.validate() {
				Ok(()) => Ok(config),
				Err(e) => Err(format!("configuration present but is invalid: {e}")),
			},
			Err(e) => Err(format!("failed to parse existing configuration: {e}")),
		},
		Ok(None) => Err("no configuration present".to_string()),
		Err(e) => Err(format!(
			"failed to load configuration: {}",
			js_value_to_string(&e)
		)),
	}
}

fn js_value_to_string(value: &wasm_bindgen::JsValue) -> String {
	value
		.as_string()
		.or_else(|| {
			js_sys::JSON::stringify(value)
				.ok()
				.and_then(|s| s.as_string())
		})
		.unwrap_or_else(|| "JavaScript error (unprintable value)".to_owned())
}
