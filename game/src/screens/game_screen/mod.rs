use crate::{
	BoardState, Config, FieldSize, GamePhase, Orientation, Screen,
	board_layout::{BoardLayout, BoardSide},
	screens::StartScreen,
	ship_render::{cells_center, ship_transform, transform_for},
};
use egui::Context;
use serde::{Deserialize, Serialize};
use sphynx_input::Input;
use sphynx_logging::info;
use sphynx_rendering::{Camera, LightUniform, Renderer};

mod board_view;
mod placement;
mod tray;

use board_view::{BoardProjection, ViewState, hovered_cell};
use placement::{
	DragPreview, DragState, all_ships_placed, dragged_ship_index, finish_drag, preview,
	ship_inventory, update_input,
};
use tray::tray_slots;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameScreen {
	area_id: egui::Id,
	go_to_start_screen: bool,
	elapsed_seconds: f32,
	phase: GamePhase,
	player_board: BoardState,
	enemy_board: BoardState,
	drag_state: Option<DragState>,
	#[serde(default)]
	preview: Option<DragPreview>,
	#[serde(default)]
	confirm_attack: bool,
}
impl Default for GameScreen {
	fn default() -> Self {
		let field_size = FieldSize::default();
		Self {
			area_id: egui::Id::unique("game_screen_menu"),
			go_to_start_screen: false,
			elapsed_seconds: 0.0,
			phase: GamePhase::Placement,
			player_board: BoardState::new(field_size.clone()),
			enemy_board: BoardState::new(field_size),
			drag_state: None,
			preview: None,
			confirm_attack: false,
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

	pub fn view_size(&self, config: &Config, aspect: f32) -> f32 {
		let tray_slots = if self.phase == GamePhase::Placement {
			config
				.normalized_ships()
				.iter()
				.map(|ship| ship.amount.get())
				.sum()
		} else {
			0
		};

		BoardLayout::new(config.board.clone(), self.phase).view_size(aspect, tray_slots)
	}

	pub fn render(&self, renderer: &mut Renderer, config: &Config) {
		let layout = BoardLayout::new(self.player_board.field_size.clone(), self.phase);
		let bounds = renderer.ship_model_bounds;
		let instances = ship_inventory(config, &self.player_board);
		let hidden_ship = if self.preview.is_some() {
			dragged_ship_index(&self.drag_state)
		} else {
			None
		};

		renderer.ship_instances.clear();
		for (index, ship) in self.player_board.ships.iter().enumerate() {
			if Some(index) == hidden_ship {
				continue;
			}
			renderer
				.ship_instances
				.push(ship_transform(ship, BoardSide::Player, &layout, &bounds));
		}
		if self.phase == GamePhase::Attack {
			for ship in &self.enemy_board.ships {
				renderer.ship_instances.push(ship_transform(
					ship,
					BoardSide::Enemy,
					&layout,
					&bounds,
				));
			}
		} else {
			let hidden_instance = match (&self.drag_state, &self.preview) {
				(Some(DragState::New { instance_id, .. }), Some(_)) => Some(*instance_id),
				_ => None,
			};
			let unplaced: Vec<_> = instances
				.iter()
				.filter(|instance| !instance.placed)
				.collect();

			for instance in unplaced {
				if Some(instance.id) == hidden_instance {
					continue;
				}
				renderer.ship_instances.push(transform_for(
					layout.tray_slot(instances.len(), instance.id),
					instance.ship_type.cell_count(),
					Orientation::Normal,
					&bounds,
				));
			}
		}

		if let Some(preview) = &self.preview {
			let cells = preview
				.ship_type
				.occupied_cells(preview.start, preview.orientation);
			renderer.ship_instances.push(transform_for(
				cells_center(&layout, BoardSide::Player, &cells),
				cells.len(),
				preview.orientation,
				&bounds,
			));
		}

		renderer.light_uniform = LightUniform::new([0.0, 20.0, 0.0], [0.05, 0.38, 0.5]);
	}

	pub fn egui_render(&mut self, ctx: &Context, camera: &Camera, config: &Config) {
		self.player_board.field_size = config.board.clone();
		self.enemy_board.field_size = config.board.clone();

		let modal_open = self.confirm_attack;
		let mut go_to_start = false;
		let instances = ship_inventory(config, &self.player_board);
		let placed = instances.iter().filter(|instance| instance.placed).count();
		let phase_text = match self.phase {
			GamePhase::Placement => {
				format!("Placement phase: {placed}/{} ships placed", instances.len())
			}
			GamePhase::Attack => "Attack phase (TODO)".to_owned(),
		};
		let phase_color = match self.phase {
			GamePhase::Placement => egui::Color32::from_rgb(120, 190, 255),
			GamePhase::Attack => egui::Color32::from_rgb(130, 235, 160),
		};

		egui::Area::new(self.area_id)
			.anchor(egui::Align2::LEFT_TOP, egui::vec2(50.0, 50.0))
			.show(ctx, |ui| {
				if crate::ui::slanted_menu_button(ui, "Go to StartScreen").clicked() && !modal_open
				{
					go_to_start = true;
				}

				ui.add_space(10.0);

				egui::Frame::new()
					.fill(egui::Color32::from_black_alpha(150))
					.inner_margin(egui::Margin::symmetric(12, 8))
					.corner_radius(8.0)
					.show(ui, |ui| {
						ui.label(
							egui::RichText::new(phase_text.as_str())
								.size(22.0)
								.strong()
								.color(phase_color),
						);
					});

				if self.phase == GamePhase::Placement {
					let ready = all_ships_placed(&instances);
					if ready && !modal_open {
						ui.add_space(10.0);
						let button = egui::Button::new(
							egui::RichText::new("Start attack phase!")
								.size(20.0)
								.strong(),
						)
						.min_size(egui::vec2(400.0, 56.0))
						.fill(egui::Color32::RED)
						.stroke(egui::Stroke::new(2.0, egui::Color32::WHITE));
						if ui.add(button).clicked() {
							self.confirm_attack = true;
						}
					}
				}
			});

		if modal_open {
			let mut cancel = false;
			let mut confirm = false;
			let response =
				egui::Modal::new(egui::Id::unique("confirm_attack_phase")).show(ctx, |ui| {
					ui.heading(
						egui::RichText::new("Are you sure you want to start the attack phase?")
							.size(22.0)
							.strong(),
					);
					ui.add_space(14.0);
					ui.horizontal(|ui| {
						if ui.button("Cancel").clicked() {
							cancel = true;
						}
						let button = egui::Button::new(egui::RichText::new("Attack").strong())
							.fill(egui::Color32::RED)
							.stroke(egui::Stroke::new(1.0, egui::Color32::WHITE));
						if ui.add(button).clicked() {
							confirm = true;
						}
					});
				});

			if response.should_close() {
				cancel = true;
			}
			if confirm {
				self.phase = GamePhase::Attack;
				self.drag_state = None;
				self.preview = None;
			}
			if confirm || cancel {
				self.confirm_attack = false;
			}
		}

		if go_to_start {
			self.go_to_start_screen = true;
		}

		let layout = BoardLayout::new(config.board.clone(), self.phase);
		let projection = BoardProjection::new(camera, ctx.viewport_rect());
		let hovered = hovered_cell(ctx, &projection, &layout);

		if !modal_open && self.drag_state.is_none() {
			let slots = tray_slots(&instances, &layout, &projection);
			if let Some(instance_id) = tray::drag_started(ctx, &slots, self.phase) {
				self.drag_state = Some(DragState::New {
					instance_id,
					orientation: Orientation::Normal,
				});
			}
		}

		if !modal_open {
			update_input(
				ctx,
				&mut self.drag_state,
				&mut self.player_board,
				hovered,
				self.phase,
			);
		}

		let mut drag_preview = if modal_open {
			None
		} else {
			preview(&self.drag_state, hovered, &instances, &self.player_board)
		};
		if !modal_open
			&& finish_drag(
				ctx,
				&mut self.drag_state,
				&mut self.player_board,
				drag_preview.as_ref(),
			) {
			drag_preview = None;
		}

		self.preview = drag_preview.clone();

		board_view::draw(
			ctx,
			&projection,
			&layout,
			ViewState {
				hovered_cell: hovered,
				preview: drag_preview.as_ref(),
				phase: self.phase,
				show_tray_label: instances.iter().any(|instance| !instance.placed),
			},
		);
	}
}
