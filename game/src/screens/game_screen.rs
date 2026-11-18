use cgmath::{Quaternion, Vector3, Zero};
use serde::{Deserialize, Serialize};
use sphynx_input::Input;
use sphynx_logging::info;
use sphynx_rendering::{LightUniform, Renderer, Transform};

use crate::{Screen, screens::StartScreen};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameScreen {
	area_id: egui::Id,
	go_to_start_screen: bool,
}
impl Default for GameScreen {
	fn default() -> Self {
		Self {
			area_id: egui::Id::unique("game_screen_menu"),
			go_to_start_screen: false,
		}
	}
}
impl GameScreen {
	pub fn update(&mut self, _input: &Input, _dt: f32, _frame: u64) -> Option<Screen> {
		if self.go_to_start_screen {
			info!(Game, "Switching to StartScreen");
			return Some(Screen::Start(StartScreen::default()));
		}

		None
	}

	pub fn render(&self, renderer: &mut Renderer) {
		renderer.instances.resize(2, Transform::default());
		renderer.instances[0] = Transform::new(
			Vector3::new(0.0, -1.0, 0.0),
			Quaternion::zero(),
			Vector3::new(100.0, 1.0, 100.0),
		);
		renderer.instances[1] = Transform::new(
			Vector3::new(0.0, 0.5, -2.0),
			Quaternion::zero(),
			Vector3::new(0.1, 0.1, 0.1),
		);

		renderer.light_uniform = LightUniform::new([0.0, 0.0, -2.0], [0.05, 0.38, 0.5]);
	}

	pub fn egui_render(&mut self, ctx: &egui::Context) {
		egui::Area::new(self.area_id)
			.anchor(egui::Align2::LEFT_TOP, egui::vec2(50.0, 50.0))
			.show(ctx, |ui| {
				if crate::ui::slanted_menu_button(ui, "Go to StartScreen").clicked() {
					self.go_to_start_screen = true;
				}
			});
	}
}
