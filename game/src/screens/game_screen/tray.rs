use crate::{
	GamePhase,
	board_layout::{BoardLayout, TRAY_ROW_DEPTH, TRAY_SLOT_WIDTH},
	screens::game_screen::{board_view::BoardProjection, placement::ShipInstance},
};
use egui::{Context, PointerButton, Rect};

pub fn tray_slots(
	instances: &[ShipInstance],
	layout: &BoardLayout,
	projection: &BoardProjection,
) -> Vec<(usize, Rect)> {
	instances
		.iter()
		.filter(|instance| !instance.placed)
		.map(|instance| {
			let center = layout.tray_slot(instances.len(), instance.id);
			let min = projection.project(
				center.x - TRAY_SLOT_WIDTH * 0.5,
				center.z - TRAY_ROW_DEPTH * 0.5,
			);
			let max = projection.project(
				center.x + TRAY_SLOT_WIDTH * 0.5,
				center.z + TRAY_ROW_DEPTH * 0.5,
			);
			(instance.id, Rect::from_two_pos(min, max))
		})
		.collect()
}

pub fn drag_started(ctx: &Context, slots: &[(usize, Rect)], phase: GamePhase) -> Option<usize> {
	if phase != GamePhase::Placement || ctx.egui_wants_pointer_input() {
		return None;
	}
	if !ctx.input(|input| input.pointer.button_pressed(PointerButton::Primary)) {
		return None;
	}

	let pointer = ctx.input(|input| input.pointer.hover_pos())?;
	slots
		.iter()
		.find(|(_, rect)| rect.contains(pointer))
		.map(|(id, _)| *id)
}
