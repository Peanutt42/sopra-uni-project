use std::path::PathBuf;

use sphynx_logging::{debug, error};

use crate::Config;

fn get_config_dir() -> PathBuf {
	// with "-peter-hennig" so that there should be no name clashing with other sopra projects
	let project_dirs =
		directories::ProjectDirs::from("de", "uni-ulm", "sopra-ship-happens-hennig-peter")
			.expect("failed to create ProjectDirs");
	let config_dir = project_dirs.config_dir();
	std::fs::create_dir_all(config_dir).expect("failed to create config dir");
	config_dir.to_path_buf()
}

fn get_filepath() -> PathBuf {
	get_config_dir().join("configuration.json")
}

/// tries to load the configuration file but returns default configuration if file loading fails
/// or parsing failed. on failure, the file is backed up with ".bak" and the default config is written
pub fn load() -> Config {
	let filepath = get_filepath();
	debug!(
		Game,
		"loading configuration file from {}",
		filepath.display()
	);
	match std::fs::read_to_string(&filepath) {
		Ok(json) => match Config::load_from_json(&json) {
			Ok(config) => match config.validate() {
				Ok(()) => {
					debug!(Game, "successfully loaded configuration file");
					return config;
				}
				Err(e) => {
					error!(
						Game,
						"configuration successfully loaded but is invalid: {e}"
					);
				}
			},
			Err(e) => {
				error!(
					Game,
					"failed to parse existing config file at {}: {e}. using default config",
					filepath.display()
				);
			}
		},
		Err(e) => {
			error!(
				Game,
				"failed to load config file at {}: {e}. using default config and overriding config file, old file will be backed up",
				filepath.display()
			);
		}
	}

	// fallback: backup invalid configuration file if exists and write default config
	let mut backup_filepath = filepath.clone();
	backup_filepath.set_extension(".json.bak");
	let _ = std::fs::rename(&filepath, backup_filepath);
	let config = Config::default();
	save(&config);
	config
}

/// will log error on failure, as there is no good error handling for this case
pub fn save(config: &Config) {
	let filepath = get_filepath();
	debug!(Game, "saving configuration file to {}", filepath.display());
	let json = config.save_to_json().expect("should always serialize");
	match std::fs::write(&filepath, json) {
		Ok(()) => debug!(
			Game,
			"successfully saved configuration file to {}",
			filepath.display()
		),
		Err(e) => error!(
			Game,
			"failed to save configuration to {}: {e}",
			filepath.display()
		),
	}
}
