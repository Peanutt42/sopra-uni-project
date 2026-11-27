use egui::{Context, Key, PointerButton};
use serde::{Deserialize, Serialize};

use crate::{
	BoardState, Config, Orientation, Player, Position, Ship, ShipType,
	screens::game_screen::GamePhase,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DragState {
	New {
		instance_id: usize,
		orientation: Orientation,
	},
	Placed {
		ship_index: usize,
		grab_index: usize,
		orientation: Orientation,
	},
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShipInstance {
	pub id: usize,
	pub ship_type: ShipType,
	pub placed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DragPreview {
	pub start: Position,
	pub cells: Vec<Position>,
	pub overlap_cells: Vec<Position>,
	pub valid: bool,
	pub ship_type: ShipType,
	pub orientation: Orientation,
}

pub fn ship_inventory(config: &Config, board: &BoardState) -> Vec<ShipInstance> {
	let mut instances = Vec::new();
	let mut next_id = 0;

	for ship_config in config.normalized_ships() {
		let placed = board
			.ships
			.iter()
			.filter(|ship| ship.ship_type() == ship_config.ship_type)
			.count()
			.min(ship_config.amount.get());

		for occurrence in 0..ship_config.amount.get() {
			instances.push(ShipInstance {
				id: next_id,
				ship_type: ship_config.ship_type,
				placed: occurrence < placed,
			});
			next_id += 1;
		}
	}

	instances
}

pub fn all_ships_placed(instances: &[ShipInstance]) -> bool {
	instances.iter().all(|instance| instance.placed)
}

pub fn update_input(
	ctx: &Context,
	state: &mut Option<DragState>,
	board: &mut BoardState,
	hovered_cell: Option<Position>,
	phase: GamePhase,
) {
	let rotate = !ctx.egui_wants_keyboard_input() && ctx.input(|input| input.key_pressed(Key::R));
	if rotate {
		let drag_counter_clockwise = !ctx.input(|input| input.modifiers.shift);
		rotate_ship(state, board, hovered_cell, phase, drag_counter_clockwise);
	}

	if phase != GamePhase::Placement
		|| state.is_some()
		|| ctx.egui_wants_pointer_input()
		|| !ctx.input(|input| input.pointer.button_pressed(PointerButton::Primary))
	{
		return;
	}

	let Some(cell) = hovered_cell else {
		return;
	};
	let Some((ship_index, grab_index)) = board.ship_at(cell) else {
		return;
	};

	let orientation = board.ships[ship_index].orientation();
	*state = Some(DragState::Placed {
		ship_index,
		grab_index,
		orientation,
	});
}

fn rotate_ship(
	state: &mut Option<DragState>,
	board: &mut BoardState,
	hovered_cell: Option<Position>,
	phase: GamePhase,
	drag_counter_clockwise: bool,
) {
	if let Some(state) = state.as_mut() {
		match state {
			DragState::New { orientation, .. } | DragState::Placed { orientation, .. } => {
				*orientation = if drag_counter_clockwise {
					orientation.turn_ccw()
				} else {
					orientation.turn_cw()
				};
			}
		}
		return;
	}

	if phase != GamePhase::Placement {
		return;
	}

	let Some(cell) = hovered_cell else {
		return;
	};
	let Some((ship_index, grab_index)) = board.ship_at(cell) else {
		return;
	};

	let ship = &board.ships[ship_index];
	let ship_type = ship.ship_type();
	let new_orientation = ship.orientation().turn_cw();
	let start = start_for_pointer(cell, new_orientation, grab_index);

	if board.can_place(ship_type, start, new_orientation, Some(ship_index)) {
		let ship = &mut board.ships[ship_index];
		ship.set_start_cell(start);
		ship.set_orientation(new_orientation);
	}
}

pub fn dragged_ship_index(state: &Option<DragState>) -> Option<usize> {
	match state {
		Some(DragState::Placed { ship_index, .. }) => Some(*ship_index),
		_ => None,
	}
}

pub fn preview(
	state: &Option<DragState>,
	hovered_cell: Option<Position>,
	instances: &[ShipInstance],
	board: &BoardState,
) -> Option<DragPreview> {
	let state = state.as_ref()?;
	let pointer = hovered_cell?;

	let (ship_type, orientation, grab_index, ignore_ship) = match state {
		DragState::New {
			instance_id,
			orientation,
		} => {
			let ship_type = instances
				.iter()
				.find(|instance| instance.id == *instance_id)?
				.ship_type;
			(ship_type, *orientation, 0, None)
		}
		DragState::Placed {
			ship_index,
			grab_index,
			orientation,
		} => {
			let ship = board.ships.get(*ship_index)?;
			(
				ship.ship_type(),
				*orientation,
				*grab_index,
				Some(*ship_index),
			)
		}
	};

	let start = start_for_pointer(pointer, orientation, grab_index);
	let cells = ship_type.occupied_cells(start, orientation);
	let overlap_cells = board.blocking_cells(&cells, ignore_ship);
	let valid = overlap_cells.is_empty() && cells.iter().all(|cell| board.in_bounds(*cell));

	Some(DragPreview {
		start,
		cells,
		overlap_cells,
		valid,
		ship_type,
		orientation,
	})
}

pub fn finish_drag(
	ctx: &Context,
	state: &mut Option<DragState>,
	board: &mut BoardState,
	preview: Option<&DragPreview>,
) -> bool {
	if !ctx.input(|input| input.pointer.button_released(PointerButton::Primary)) || state.is_none()
	{
		return false;
	}

	let released = state.take().expect("drag state was checked");
	if let Some(preview) = preview
		&& preview.valid
	{
		match released {
			DragState::New { orientation, .. } => {
				board.ships.push(Ship::new(
					Player::Human1,
					preview.ship_type,
					preview.start,
					orientation,
				));
			}
			DragState::Placed {
				ship_index,
				orientation,
				..
			} => {
				if let Some(ship) = board.ships.get_mut(ship_index) {
					ship.set_start_cell(preview.start);
					ship.set_orientation(orientation);
				}
			}
		}
	}

	true
}

fn start_for_pointer(pointer: Position, orientation: Orientation, grab_index: usize) -> Position {
	let offset = orientation.offset(grab_index);
	Position::new(pointer.x - offset.x, pointer.y - offset.y)
}
