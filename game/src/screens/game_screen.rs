use cgmath::{Deg, Euler, Quaternion, Vector3};
use serde::{Deserialize, Serialize};
use sphynx_input::Input;
use sphynx_logging::info;
use sphynx_rendering::{LightUniform, Renderer, Transform};

use crate::{Screen, screens::StartScreen};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameScreen {
	area_id: egui::Id,
	go_to_start_screen: bool,
	elapsed_seconds: f32,
}
impl Default for GameScreen {
	fn default() -> Self {
		Self {
			area_id: egui::Id::unique("game_screen_menu"),
			go_to_start_screen: false,
			elapsed_seconds: 0.0,
		}
	}
}
impl GameScreen {
	pub fn update(&mut self, _input: &Input, dt: f32, _frame: u64) -> Option<Screen> {
		self.elapsed_seconds += dt;

		if self.go_to_start_screen {
			info!(Game, "Switching to StartScreen");
			return Some(Screen::Start(StartScreen::default()));
		}

		None
	}

	pub fn render(&self, renderer: &mut Renderer) {
		renderer.ship_instances = vec![
			Transform::new(
				Vector3::new(-15.0, -1.0, -15.0),
				Quaternion::from(Euler {
					x: Deg(0.0),
					y: Deg(90.0 * self.elapsed_seconds),
					z: Deg(0.0),
				}),
				Vector3::new(0.05, 0.05, 0.05),
			),
			Transform::new(
				Vector3::new(15.0, -1.0, -15.0),
				Quaternion::from(Euler {
					x: Deg(0.0),
					y: Deg(-90.0 * self.elapsed_seconds),
					z: Deg(0.0),
				}),
				Vector3::new(0.15, 0.15, 0.15),
			),
		];

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
