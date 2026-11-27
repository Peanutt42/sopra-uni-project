use crate::{FieldSize, Orientation, Position, Ship, ShipType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardState {
	pub field_size: FieldSize,
	pub attacked_cells: Vec<Position>,
	pub ships: Vec<Ship>,
}
impl BoardState {
	pub fn new(field_size: FieldSize) -> Self {
		Self {
			field_size,
			attacked_cells: Vec::new(),
			ships: Vec::new(),
		}
	}

	fn occupancy(&self) -> HashMap<Position, (usize, usize)> {
		self.ships
			.iter()
			.enumerate()
			.flat_map(|(ship_index, ship)| {
				ship.occupied_cells()
					.into_iter()
					.enumerate()
					.map(move |(grab_index, cell)| (cell, (ship_index, grab_index)))
			})
			.collect()
	}

	pub fn ship_at(&self, cell: Position) -> Option<(usize, usize)> {
		self.occupancy().get(&cell).copied()
	}

	pub fn in_bounds(&self, cell: Position) -> bool {
		cell.x >= 0
			&& (cell.x as usize) < self.field_size.x
			&& cell.y >= 0
			&& (cell.y as usize) < self.field_size.y
	}

	pub fn blocking_cells(&self, cells: &[Position], ignore_ship: Option<usize>) -> Vec<Position> {
		let occupancy = self.occupancy();
		cells
			.iter()
			.copied()
			.filter(|cell| {
				occupancy
					.get(cell)
					.is_some_and(|&(ship_index, _)| Some(ship_index) != ignore_ship)
			})
			.collect()
	}

	pub fn can_place(
		&self,
		ship_type: ShipType,
		start: Position,
		orientation: Orientation,
		ignore_ship: Option<usize>,
	) -> bool {
		let cells = ship_type.occupied_cells(start, orientation);
		cells.iter().all(|&cell| self.in_bounds(cell))
			&& self.blocking_cells(&cells, ignore_ship).is_empty()
	}
}
