use crate::ShipType;
use serde::{Deserialize, Serialize};
use std::num::NonZero;
use thiserror::Error;

// loading and saving uses localStorage
#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
use web::{load, save};

// loading and saving uses relative config file
#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
use native::{load, save};

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
					ship_type: ShipType::Explorer,
					amount: NonZero::new(1).unwrap(),
				},
				ShipConfig {
					ship_type: ShipType::Cruiser,
					amount: NonZero::new(1).unwrap(),
				},
				ShipConfig {
					ship_type: ShipType::Mothership,
					amount: NonZero::new(1).unwrap(),
				},
			],
			lightning_event: LightningEventConfig::default(),
		}
	}
}
impl Config {
	/// Merges duplicate ship types.
	pub fn normalized_ships(&self) -> Vec<ShipConfig> {
		let mut normalized: Vec<ShipConfig> = Vec::new();

		for ship_config in &self.ships {
			if let Some(existing) = normalized
				.iter_mut()
				.find(|existing| existing.ship_type == ship_config.ship_type)
			{
				let amount = existing.amount.get() + ship_config.amount.get();
				existing.amount =
					NonZero::new(amount).expect("sum of non-zero amounts is non-zero");
			} else {
				normalized.push(ship_config.clone());
			}
		}

		normalized
	}

	/// Returns whether the configuration can be played with.
	pub(crate) fn validate(&self) -> Result<(), ConfigValidationError> {
		if self.board.x == 0 || self.board.y == 0 {
			return Err(ConfigValidationError::EmptyBoard);
		}

		let ship_configs = self.normalized_ships();
		let max_ship_length = ship_configs
			.iter()
			.map(|ship_config| ship_config.ship_type.cell_count())
			.max()
			.unwrap_or(0);

		if max_ship_length > self.board.x.max(self.board.y) {
			return Err(ConfigValidationError::ShipDoesNotFit {
				cell_count: max_ship_length,
				board_x: self.board.x,
				board_y: self.board.y,
			});
		}

		let total_field_cell_count = self.board.x.saturating_mul(self.board.y);
		let total_ship_cell_count = ship_configs.iter().fold(0usize, |total, ship_config| {
			total.saturating_add(
				ship_config
					.ship_type
					.cell_count()
					.saturating_mul(ship_config.amount.get()),
			)
		});

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

	pub fn load() -> Self {
		load() // either in ./native.rs or ./web.rs
	}

	pub(crate) fn save_to_json(&self) -> serde_json::Result<String> {
		serde_json::to_string_pretty(self)
	}

	pub fn save(&self) {
		save(self); // either in ./native.rs or ./web.rs
	}
}

#[derive(Debug, Clone, Copy, Error)]
pub enum ConfigValidationError {
	#[error("the board must have at least one cell in each direction")]
	EmptyBoard,
	#[error("a ship occupying {cell_count} cells does not fit on a {board_x}x{board_y} board")]
	ShipDoesNotFit {
		cell_count: usize,
		board_x: usize,
		board_y: usize,
	},
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
	pub ship_type: ShipType,
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
