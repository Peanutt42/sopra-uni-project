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
