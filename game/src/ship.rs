use crate::Player;
use serde::{Deserialize, Serialize};

// TODO: add Custom type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShipType {
	#[serde(rename = "EXPLORER")]
	Explorer,
	#[serde(rename = "CRUISER")]
	Cruiser,
	#[serde(rename = "MOTHERSHIP")]
	Mothership,
}
impl ShipType {
	pub fn cell_count(self) -> usize {
		match self {
			Self::Explorer => 1,
			Self::Cruiser => 2,
			Self::Mothership => 3,
		}
	}

	pub fn occupied_cells(self, start: Position, orientation: Orientation) -> Vec<Position> {
		let step = orientation.step();
		(0..self.cell_count() as i32)
			.map(|offset| Position::new(start.x + step.x * offset, start.y + step.y * offset))
			.collect()
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ship {
	owner: Player,
	/// `type` is a keyword, hence `ship_type`.
	#[serde(alias = "type_")]
	ship_type: ShipType,
	start_cell: Position,
	orientation: Orientation,
}
impl Ship {
	pub fn new(
		owner: Player,
		ship_type: ShipType,
		start: Position,
		orientation: Orientation,
	) -> Self {
		Self {
			owner,
			ship_type,
			start_cell: start,
			orientation,
		}
	}

	pub fn owner(&self) -> &Player {
		&self.owner
	}

	pub fn ship_type(&self) -> ShipType {
		self.ship_type
	}

	pub fn start_cell(&self) -> Position {
		self.start_cell
	}

	pub fn set_start_cell(&mut self, start: Position) {
		self.start_cell = start;
	}

	pub fn orientation(&self) -> Orientation {
		self.orientation
	}

	pub fn set_orientation(&mut self, orientation: Orientation) {
		self.orientation = orientation;
	}

	pub fn occupied_cells(&self) -> Vec<Position> {
		self.ship_type
			.occupied_cells(self.start_cell, self.orientation)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
	pub x: i32,
	pub y: i32,
}
impl Position {
	pub fn new(x: i32, y: i32) -> Self {
		Self { x, y }
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Orientation {
	#[default]
	Normal,
	CW90,
	CW180,
	CW270,
}
impl Orientation {
	pub fn step(self) -> Position {
		match self {
			Self::Normal => Position::new(1, 0),
			Self::CW90 => Position::new(0, 1),
			Self::CW180 => Position::new(-1, 0),
			Self::CW270 => Position::new(0, -1),
		}
	}

	pub fn offset(self, index: usize) -> Position {
		let step = self.step();
		let index = index as i32;
		Position::new(step.x * index, step.y * index)
	}

	pub fn turn_cw(self) -> Self {
		match self {
			Self::Normal => Self::CW90,
			Self::CW90 => Self::CW180,
			Self::CW180 => Self::CW270,
			Self::CW270 => Self::Normal,
		}
	}

	pub fn turn_ccw(self) -> Self {
		match self {
			Self::Normal => Self::CW270,
			Self::CW90 => Self::Normal,
			Self::CW180 => Self::CW90,
			Self::CW270 => Self::CW180,
		}
	}
}
