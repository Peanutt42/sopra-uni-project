use serde::{Deserialize, Serialize};
use sphynx_input::Input;
use sphynx_logging::info;
use sphynx_rendering::Renderer;

use crate::Screen;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartScreen {
	area_id: egui::Id,
	go_to_game_screen: bool,
	quit_game: bool,
}
impl Default for StartScreen {
	fn default() -> Self {
		Self {
			area_id: egui::Id::unique("start_screen_menu"),
			go_to_game_screen: false,
			quit_game: false,
		}
	}
}
impl StartScreen {
	pub fn update(
		&mut self,
		continue_running: &mut bool,
		_input: &Input,
		_dt: f32,
		_frame: u64,
	) -> Option<Screen> {
		if self.quit_game {
			*continue_running = false;
			return None;
		}

		if self.go_to_game_screen {
			info!(Game, "Switching to GameScreen");
			return Some(Screen::Game(Box::default()));
		}

		None
	}

	pub fn render(&self, renderer: &mut Renderer, _config: &crate::Config) {
		renderer.cube_instances.clear();
		renderer.ship_instances.clear();
	}

	pub fn egui_render(&mut self, ctx: &egui::Context) {
		egui::Area::new(self.area_id)
			.anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
			.show(ctx, |ui| {
				ui.vertical_centered(|ui| {
					ui.set_width(ui.available_width());

					ui.label(
						egui::RichText::new("Sopra Versenken")
							.strong()
							.size(80.0)
							.heading(),
					);

					ui.add_space(100.0);

					if crate::ui::slanted_menu_button(ui, "Go to Game Screen").clicked() {
						self.go_to_game_screen = true;
					}

					ui.add_space(10.0);

					if crate::ui::slanted_menu_button(ui, "Quit Game").clicked() {
						self.quit_game = true;
					}
				});
			});
	}
}
