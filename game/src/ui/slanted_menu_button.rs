use cgmath::{Deg, Rad};
use egui::{
	Color32, Pos2, Rect, Response, RichText, Sense, Shape, Stroke, TextStyle, TextWrapMode, Ui,
	Vec2, WidgetText, vec2,
};

const BUTTON_SIZE: Vec2 = Vec2 { x: 400.0, y: 75.0 };
const BUTTON_COLOR_PRESSED: Color32 = Color32::from_rgb(40, 80, 130);
const BUTTON_COLOR_HOVERED: Color32 = Color32::from_rgb(70, 130, 200);
const BUTTON_COLOR_NORMAL: Color32 = Color32::from_rgb(35, 65, 100);
const OUTLINE_WIDTH: f32 = 5.0;
const FOCUS_RING_OFFSET: f32 = 3.0;
const FOCUS_RING_WIDTH: f32 = 7.5;
const SLANT_ANGLE: Deg<f32> = Deg(20.0);

pub fn slanted_menu_button(ui: &mut Ui, text: &str) -> Response {
	let (rect, response) = ui.allocate_exact_size(BUTTON_SIZE, Sense::click());

	let painter = ui.painter();

	let slant_angle_rad: Rad<f32> = SLANT_ANGLE.into();
	let slant_amount = rect.height() * slant_angle_rad.0.tan();

	let points = get_slanted_rect_points(rect, slant_amount);

	let color = if response.is_pointer_button_down_on() {
		BUTTON_COLOR_PRESSED
	} else if response.hovered() {
		BUTTON_COLOR_HOVERED
	} else {
		BUTTON_COLOR_NORMAL
	};

	if response.has_focus() {
		let ring_rect = rect.expand2(vec2(
			FOCUS_RING_OFFSET + OUTLINE_WIDTH / 2.0,
			FOCUS_RING_OFFSET,
		));
		let ring_slant_amount = ring_rect.height() * slant_angle_rad.0.tan();
		let ring_points = get_slanted_rect_points(ring_rect, ring_slant_amount);
		let ring_color = ui.style().visuals.selection.stroke.color;

		painter.add(Shape::convex_polygon(
			ring_points,
			Color32::TRANSPARENT,
			Stroke::new(FOCUS_RING_WIDTH, ring_color),
		));
	}

	painter.add(Shape::convex_polygon(
		points.clone(),
		color,
		Stroke::new(OUTLINE_WIDTH, Color32::WHITE),
	));

	let rich_text = RichText::new(text).size(28.0).strong();
	let text_widget: WidgetText = rich_text.into();
	let galley = text_widget.into_galley(
		ui,
		Some(TextWrapMode::Extend),
		f32::INFINITY,
		TextStyle::Button,
	);
	let text_pos = rect.center() - galley.size() / 2.0;
	painter.galley(text_pos, galley, Color32::WHITE);

	response
}

fn get_slanted_rect_points(rect: Rect, slant: f32) -> Vec<Pos2> {
	vec![
		rect.left_top() + vec2(slant, 0.0),
		rect.right_top(),
		rect.right_bottom() - vec2(slant, 0.0),
		rect.left_bottom(),
	]
}
