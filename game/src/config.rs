use crate::ShipType;
use serde::{Deserialize, Serialize};
use sphynx_logging::{debug, error};
use std::{num::NonZero, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
	pub server: WSServerConfig,
	pub board: FieldSize,
	pub ships: Vec<ShipConfig>,
	#[serde(rename = "lightningEvent")]
	pub lightning_event: LightningEventConfig,
}
impl Default for Config {
	fn default() -> Self {
		Self {
			server: WSServerConfig::default(),
			board: FieldSize::default(),
			ships: vec![
				ShipConfig {
					type_: ShipType::Explorer,
					amount: NonZero::new(1).unwrap(),
				},
				ShipConfig {
					type_: ShipType::Cruiser,
					amount: NonZero::new(1).unwrap(),
				},
				ShipConfig {
					type_: ShipType::Mothership,
					amount: NonZero::new(1).unwrap(),
				},
			],
			lightning_event: LightningEventConfig::default(),
		}
	}
}
impl Config {
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
		Self::get_config_dir().join("configuration.json")
	}

	/// returns if the configuration is valid (there is a very small chance false positives when
	/// ship shapes are very weird, too demanding to check)
	pub(crate) fn validate(&self) -> Result<(), ConfigValidationError> {
		let total_field_cell_count = self.board.x * self.board.y;
		let total_ship_cell_count: usize = self
			.ships
			.iter()
			.map(|ship_config| ship_config.type_.get_cell_count() * ship_config.amount.get())
			.sum();

		if total_field_cell_count < total_ship_cell_count {
			Err(ConfigValidationError::TooSmallFieldSize {
				total_field_cell_count,
				total_ship_cell_count,
			})
		} else {
			Ok(())
		}
	}

	pub(crate) fn load_from_json(json: &str) -> serde_json::Result<Self> {
		serde_json::from_str(json)
	}

	/// tries to load the configuration file but returns default configuration if file loading fails
	/// or parsing failed. on failure, the file is backed up with ".bak" and the default config is written
	pub fn load() -> Self {
		let filepath = Self::get_filepath();
		debug!(
			Game,
			"loading configuration file from {}",
			filepath.display()
		);
		match std::fs::read_to_string(&filepath) {
			Ok(json) => match Self::load_from_json(&json) {
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
		let config = Self::default();
		config.save();
		config
	}

	pub(crate) fn save_to_json(&self) -> serde_json::Result<String> {
		serde_json::to_string_pretty(self)
	}

	/// will log error on failure, as there is no good error handling for this case
	pub fn save(&self) {
		let filepath = Self::get_filepath();
		debug!(Game, "saving configuration file to {}", filepath.display());
		let json = self.save_to_json().expect("should always serialize");
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
}

#[derive(Debug, Clone, Copy, Error)]
pub enum ConfigValidationError {
	#[error(
		"there are only {total_field_cell_count} cells on the board but all ships occupy {total_ship_cell_count} cells in total"
	)]
	TooSmallFieldSize {
		total_field_cell_count: usize,
		total_ship_cell_count: usize,
	},
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WSServerConfig {
	// TODO: parse dont validate: String -> URL
	/// None -> disabled
	pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipConfig {
	#[serde(rename = "type")] // type is keyword
	pub type_: ShipType,
	pub amount: NonZero<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightningEventConfig {
	pub enabled: bool,
	/// first round is 0
	#[serde(rename = "startRound")]
	pub start_round: usize,
}
impl Default for LightningEventConfig {
	fn default() -> Self {
		Self {
			enabled: true,
			start_round: 0,
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSize {
	pub x: usize,
	pub y: usize,
}
impl Default for FieldSize {
	fn default() -> Self {
		Self { x: 10, y: 5 }
	}
}
