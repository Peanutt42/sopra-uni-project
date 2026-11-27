use crate::{
	Orientation, Position, Ship,
	board_layout::{BoardLayout, BoardSide, CELL_SIZE},
};
use cgmath::{Deg, Quaternion, Rotation3, Vector3};
use sphynx_rendering::{ModelBounds, Transform};

const SHIP_WIDTH_FILL: f32 = 0.9;

fn orientation_yaw(orientation: Orientation) -> Deg<f32> {
	match orientation {
		Orientation::Normal => Deg(90.0),
		Orientation::CW90 => Deg(0.0),
		Orientation::CW180 => Deg(-90.0),
		Orientation::CW270 => Deg(180.0),
	}
}

pub fn cells_center(layout: &BoardLayout, side: BoardSide, cells: &[Position]) -> Vector3<f32> {
	cells.iter().fold(Vector3::new(0.0, 0.0, 0.0), |sum, cell| {
		sum + layout.cell_center(side, *cell)
	}) / cells.len() as f32
}

pub fn transform_for(
	center: Vector3<f32>,
	cell_count: usize,
	orientation: Orientation,
	bounds: &ModelBounds,
) -> Transform {
	let size = bounds.size();
	let width_scale = SHIP_WIDTH_FILL * CELL_SIZE / size.x.max(f32::EPSILON);
	let length_scale = cell_count as f32 * CELL_SIZE / size.z.max(f32::EPSILON);

	Transform::new(
		center,
		Quaternion::from_axis_angle(Vector3::new(0.0, 1.0, 0.0), orientation_yaw(orientation)),
		Vector3::new(width_scale, width_scale, length_scale),
	)
}

pub fn ship_transform(
	ship: &Ship,
	side: BoardSide,
	layout: &BoardLayout,
	bounds: &ModelBounds,
) -> Transform {
	let cells = ship.occupied_cells();
	transform_for(
		cells_center(layout, side, &cells),
		cells.len(),
		ship.orientation(),
		bounds,
	)
}
