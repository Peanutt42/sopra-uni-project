use crate::{
	GamePhase, Position,
	board_layout::{BOARD_Y, BoardLayout, BoardSide, CELL_SIZE},
	screens::game_screen::placement::DragPreview,
};
use cgmath::{Matrix4, SquareMatrix, Vector3, Vector4};
use egui::{Align2, Color32, Context, LayerId, Painter, Pos2, Rect, Shape, Stroke};
use sphynx_rendering::Camera;

pub struct BoardProjection {
	proj_view: Matrix4<f32>,
	viewport: Rect,
}
impl BoardProjection {
	pub fn new(camera: &Camera, viewport: Rect) -> Self {
		let aspect = Camera::get_aspect(viewport.width(), viewport.height());
		Self {
			proj_view: camera.get_projection_view_matrix(aspect),
			viewport,
		}
	}

	pub fn project(&self, x: f32, z: f32) -> Pos2 {
		let clip = self.proj_view * Vector4::new(x, BOARD_Y, z, 1.0);
		let ndc = clip / clip.w;
		Pos2::new(
			self.viewport.min.x + (ndc.x + 1.0) * 0.5 * self.viewport.width(),
			self.viewport.min.y + (1.0 - ndc.y) * 0.5 * self.viewport.height(),
		)
	}

	fn hit_on_plane(&self, pos: Pos2, plane_y: f32) -> Option<Vector3<f32>> {
		if self.viewport.width() <= 0.0 || self.viewport.height() <= 0.0 {
			return None;
		}

		let inverse = self.proj_view.invert()?;
		let unproject = |depth: f32| {
			let clip = inverse
				* Vector4::new(
					(pos.x - self.viewport.min.x) / self.viewport.width() * 2.0 - 1.0,
					1.0 - (pos.y - self.viewport.min.y) / self.viewport.height() * 2.0,
					depth,
					1.0,
				);
			Vector3::new(clip.x / clip.w, clip.y / clip.w, clip.z / clip.w)
		};

		let near = unproject(0.0);
		let far = unproject(1.0);
		let direction = far - near;
		if direction.y.abs() < f32::EPSILON {
			return None;
		}

		let t = (plane_y - near.y) / direction.y;
		if !t.is_finite() || t < 0.0 {
			return None;
		}

		Some(near + direction * t)
	}
}

pub fn hovered_cell(
	ctx: &Context,
	projection: &BoardProjection,
	layout: &BoardLayout,
) -> Option<Position> {
	let pos = ctx.input(|input| input.pointer.hover_pos())?;
	let hit = projection.hit_on_plane(pos, BOARD_Y)?;
	layout.cell_at(hit, BoardSide::Player)
}

pub struct ViewState<'a> {
	pub hovered_cell: Option<Position>,
	pub preview: Option<&'a DragPreview>,
	pub phase: GamePhase,
	pub show_tray_label: bool,
}

pub fn draw(
	ctx: &Context,
	projection: &BoardProjection,
	layout: &BoardLayout,
	state: ViewState<'_>,
) {
	let painter = ctx.layer_painter(LayerId::background());

	draw_board(&painter, projection, layout, BoardSide::Player, |x, y| {
		let cell = Position::new(x as i32, y as i32);
		if state
			.preview
			.is_some_and(|preview| preview.overlap_cells.contains(&cell))
		{
			Color32::from_rgba_unmultiplied(230, 70, 70, 110)
		} else if state.hovered_cell == Some(cell) {
			Color32::from_white_alpha(40)
		} else {
			Color32::TRANSPARENT
		}
	});
	draw_board_label(&painter, projection, layout, BoardSide::Player, "Player");

	if state.phase == GamePhase::Attack {
		draw_board(&painter, projection, layout, BoardSide::Enemy, |_, _| {
			Color32::TRANSPARENT
		});
		draw_board_label(&painter, projection, layout, BoardSide::Enemy, "Enemy");
	} else if state.show_tray_label {
		draw_label(
			&painter,
			projection,
			layout.tray_label_position(),
			Align2::CENTER_TOP,
			"Ships to place:",
		);
	}

	if let Some(preview) = state.preview {
		draw_preview(&painter, projection, layout, preview);
	}
}

fn cell_corners(
	projection: &BoardProjection,
	layout: &BoardLayout,
	side: BoardSide,
	cell: Position,
) -> [Pos2; 4] {
	let center = layout.cell_center(side, cell);
	let half_cell = CELL_SIZE * 0.5;
	[
		projection.project(center.x - half_cell, center.z - half_cell),
		projection.project(center.x + half_cell, center.z - half_cell),
		projection.project(center.x + half_cell, center.z + half_cell),
		projection.project(center.x - half_cell, center.z + half_cell),
	]
}

fn draw_cell(
	painter: &Painter,
	projection: &BoardProjection,
	layout: &BoardLayout,
	side: BoardSide,
	cell: Position,
	fill: Color32,
	stroke: Stroke,
) {
	painter.add(Shape::convex_polygon(
		cell_corners(projection, layout, side, cell).to_vec(),
		fill,
		stroke,
	));
}

fn draw_board(
	painter: &Painter,
	projection: &BoardProjection,
	layout: &BoardLayout,
	side: BoardSide,
	fill: impl Fn(usize, usize) -> Color32,
) {
	let field_size = layout.field_size();
	let stroke = Stroke::new(1.0, Color32::WHITE);

	for x in 0..field_size.x {
		for y in 0..field_size.y {
			draw_cell(
				painter,
				projection,
				layout,
				side,
				Position::new(x as i32, y as i32),
				fill(x, y),
				stroke,
			);
		}
	}
}

fn draw_preview(
	painter: &Painter,
	projection: &BoardProjection,
	layout: &BoardLayout,
	preview: &DragPreview,
) {
	let stroke = Stroke::new(
		2.0,
		if preview.valid {
			Color32::WHITE
		} else {
			Color32::from_rgb(255, 100, 100)
		},
	);

	for &cell in &preview.cells {
		draw_cell(
			painter,
			projection,
			layout,
			BoardSide::Player,
			cell,
			Color32::TRANSPARENT,
			stroke,
		);
	}
}

fn draw_board_label(
	painter: &Painter,
	projection: &BoardProjection,
	layout: &BoardLayout,
	side: BoardSide,
	label: &str,
) {
	draw_label(
		painter,
		projection,
		layout.label_position(side),
		Align2::CENTER_BOTTOM,
		label,
	);
}

fn draw_label(
	painter: &Painter,
	projection: &BoardProjection,
	(x, z): (f32, f32),
	align: Align2,
	label: &str,
) {
	painter.text(
		projection.project(x, z),
		align,
		label,
		egui::FontId::proportional(25.0),
		Color32::WHITE,
	);
}
