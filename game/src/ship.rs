use serde::{Deserialize, Serialize};

// TODO: add Custom type
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ShipType {
	#[serde(rename = "EXPLORER")]
	Explorer,
	#[serde(rename = "CRUISER")]
	Cruiser,
	#[serde(rename = "MOTHERSHIP")]
	Mothership,
}
impl ShipType {
	/// returns the amount of cells that this ship occupies
	pub fn get_cell_count(&self) -> usize {
		match self {
			Self::Explorer => 1,
			Self::Cruiser => 2,
			Self::Mothership => 3,
		}
	}
}
