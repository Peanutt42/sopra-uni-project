use serde::{Deserialize, Serialize};
use sphynx_input::Input;
use sphynx_rendering::Renderer;

use crate::screens::{GameScreen, StartScreen};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Screen {
	Start(StartScreen),
	Game(GameScreen),
}
impl Default for Screen {
	fn default() -> Self {
		Self::Start(StartScreen::default())
	}
}
impl Screen {
	/// updates the screen and returns a new screen optionally if the screen should change
	pub fn update(
		&mut self,
		continue_running: &mut bool,
		input: &Input,
		dt: f32,
		frame: u64,
	) -> Option<Screen> {
		match self {
			Self::Start(start) => start.update(continue_running, input, dt, frame),
			Self::Game(game) => game.update(input, dt, frame),
		}
	}

	pub fn render(&self, renderer: &mut Renderer) {
		match self {
			Self::Start(start) => start.render(renderer),
			Self::Game(game) => game.render(renderer),
		}
	}

	pub fn egui_render(&mut self, ctx: &egui::Context) {
		match self {
			Self::Start(start) => start.egui_render(ctx),
			Self::Game(game) => game.egui_render(ctx),
		}
	}
}
