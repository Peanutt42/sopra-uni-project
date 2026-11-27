use crate::{FieldSize, GamePhase, Position};
use cgmath::Vector3;
use sphynx_rendering::Camera;

pub const CELL_SIZE: f32 = 2.0;
pub const BOARD_Y: f32 = -1.0;
pub const BOARD_GAP: f32 = 8.0;
pub const BOARD_LABEL_SPACE: f32 = 2.0;
pub const LABEL_OFFSET: f32 = 1.5;
pub const TRAY_SLOT_WIDTH: f32 = CELL_SIZE * 3.5;
pub const TRAY_ROW_DEPTH: f32 = CELL_SIZE;
pub const TRAY_GAP: f32 = 2.5;
const SCREEN_FILL: f32 = 0.85;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardSide {
	Player,
	Enemy,
}

#[derive(Debug, Clone)]
pub struct BoardLayout {
	field_size: FieldSize,
	game_phase: GamePhase,
}
impl BoardLayout {
	pub fn new(field_size: FieldSize, game_phase: GamePhase) -> Self {
		Self {
			field_size,
			game_phase,
		}
	}

	pub fn field_size(&self) -> &FieldSize {
		&self.field_size
	}

	pub fn half_extents(&self) -> (f32, f32) {
		(
			self.field_size.x as f32 * CELL_SIZE / 2.0,
			self.field_size.y as f32 * CELL_SIZE / 2.0,
		)
	}

	pub fn center_z(&self, side: BoardSide) -> f32 {
		let depth = self.field_size.y as f32 * CELL_SIZE;

		match (self.game_phase, side) {
			(GamePhase::Placement, BoardSide::Player) => 0.0,
			(GamePhase::Placement, BoardSide::Enemy) => -(depth + BOARD_GAP) / 2.0,
			(GamePhase::Attack, BoardSide::Player) => (depth + BOARD_GAP) / 2.0,
			(GamePhase::Attack, BoardSide::Enemy) => -(depth + BOARD_GAP) / 2.0,
		}
	}

	pub fn view_size(&self, aspect: f32, tray_slot_count: usize) -> f32 {
		let board_width = self.field_size.x as f32 * CELL_SIZE;
		let board_depth = self.field_size.y as f32 * CELL_SIZE;
		let (board_count, gap) = match self.game_phase {
			GamePhase::Placement => (1.0, 0.0),
			GamePhase::Attack => (2.0, BOARD_GAP),
		};

		let (columns, rows) = self.tray_grid(tray_slot_count);
		let (tray_width, tray_depth) = if self.game_phase == GamePhase::Placement && rows > 0 {
			(
				columns as f32 * TRAY_SLOT_WIDTH,
				TRAY_GAP + rows as f32 * TRAY_ROW_DEPTH,
			)
		} else {
			(0.0, 0.0)
		};

		let content_width = board_width.max(tray_width) + 2.0 * BOARD_LABEL_SPACE;
		let content_height = board_depth * board_count + gap + 2.0 * BOARD_LABEL_SPACE + tray_depth;
		let safe_aspect = Camera::get_aspect(aspect, 1.0);

		(content_height / SCREEN_FILL).max(content_width / (safe_aspect * SCREEN_FILL))
	}

	pub fn tray_grid(&self, slot_count: usize) -> (usize, usize) {
		if slot_count == 0 {
			return (0, 0);
		}

		let (half_width, _) = self.half_extents();
		let max_columns = (half_width * 2.0 / TRAY_SLOT_WIDTH).floor().max(1.0) as usize;
		let columns = slot_count.min(max_columns);
		(columns, slot_count.div_ceil(columns))
	}

	pub fn tray_slot(&self, slot_count: usize, index: usize) -> Vector3<f32> {
		let (_, half_depth) = self.half_extents();
		let (columns, _) = self.tray_grid(slot_count);
		let width = columns as f32 * TRAY_SLOT_WIDTH;
		let tray_top = self.center_z(BoardSide::Player) + half_depth + TRAY_GAP;

		Vector3::new(
			-width * 0.5 + (index % columns) as f32 * TRAY_SLOT_WIDTH + TRAY_SLOT_WIDTH * 0.5,
			BOARD_Y,
			tray_top + (index / columns) as f32 * TRAY_ROW_DEPTH + TRAY_ROW_DEPTH * 0.5,
		)
	}

	pub fn tray_label_position(&self) -> (f32, f32) {
		let (_, half_depth) = self.half_extents();
		(
			0.0,
			self.center_z(BoardSide::Player) + half_depth + TRAY_GAP * 0.5,
		)
	}

	pub fn cell_center(&self, side: BoardSide, cell: Position) -> Vector3<f32> {
		let (half_width, half_depth) = self.half_extents();
		Vector3::new(
			-half_width + (cell.x as f32 + 0.5) * CELL_SIZE,
			BOARD_Y,
			self.center_z(side) - half_depth + (cell.y as f32 + 0.5) * CELL_SIZE,
		)
	}

	pub fn cell_at(&self, hit: Vector3<f32>, side: BoardSide) -> Option<Position> {
		let center_z = self.center_z(side);
		let (half_width, half_depth) = self.half_extents();
		let x = ((hit.x + half_width) / CELL_SIZE).floor();
		let y = ((hit.z - center_z + half_depth) / CELL_SIZE).floor();

		if x >= 0.0 && x < self.field_size.x as f32 && y >= 0.0 && y < self.field_size.y as f32 {
			Some(Position::new(x as i32, y as i32))
		} else {
			None
		}
	}

	pub fn label_position(&self, side: BoardSide) -> (f32, f32) {
		let (_, half_depth) = self.half_extents();
		(0.0, self.center_z(side) - half_depth - LABEL_OFFSET)
	}
}
