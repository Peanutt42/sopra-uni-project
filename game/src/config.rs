use serde::{Deserialize, Serialize};
use sphynx_logging::error;
use std::{num::NonZero, path::PathBuf};

use crate::ShipType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
	pub server: WSServerConfig,
	pub board: FieldSize,
	pub ships: Vec<ShipConfig>,
	#[serde(rename = "lightningEvent")]
	pub lightning_event: LightningEventConfig,
}
impl Config {
	fn get_filepath() -> PathBuf {
		// with "-peter-hennig" so that there should be no name clashing with other sopra projects
		directories::ProjectDirs::from("de", "uni-ulm", "sopra-ship-happens-hennig-peter")
			.expect("failed to create ProjectDirs")
			.config_dir()
			.join("configuration.json")
	}

	pub(crate) fn load_from_json(json: &str) -> serde_json::Result<Self> {
		serde_json::from_str(json)
	}

	/// tries to load the configuration file but returns default configuration if file loading fails
	/// or parsing failed. on failure, the file is not (YET!) overriden by this function
	pub fn load() -> Self {
		let filepath = Self::get_filepath();
		match std::fs::read_to_string(&filepath) {
			Ok(json) => match Self::load_from_json(&json) {
				Ok(config) => return config,
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
					"failed to load config file at {}: {e}. using default config",
					filepath.display()
				);
			}
		}

		todo!("create default configuration")
	}
}

#[derive(Debug, Clone /*, Default*/, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSize {
	pub x: usize,
	pub y: usize,
}
