use eframe::{
	egui::{
		self,
		style::{ScrollStyle, Selection, Spacing, Widgets},
		Align, Checkbox, Color32, Layout, Margin, RichText, Rounding, ScrollArea, Stroke, Style,
		TextEdit, Ui, Vec2, Visuals, WidgetText,
	},
	epaint::Shadow,
};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};
use lineupper::{
	create::{create_team_file, FormatType},
	player::{PlayerState, Position},
	roster::{Roster, RosterFile},
	slugify,
};
use rfd::FileDialog;
use strum::VariantArray;

use crate::{message::Message, setup::setup_custom_fonts, widget_creator};

pub struct RosterEditor {
	rows: [RosterRow; 23],
	team: String,
}

impl RosterEditor {
	const INNER_MARGIN: Margin = Margin::same(5.0);

	pub fn editor(&mut self, ui: &mut Ui) {
		// FIXME: Remove this here, and in menu. Add instead a style that is persistent.
		// Instead of creating it constantly.
		Self::set_style(ui.style_mut());

		egui::Frame::window(ui.style())
			.inner_margin(Margin::symmetric(6.0, Self::INNER_MARGIN.left)) // x breaks striped if not same as inner_margin, or if spacing.x too high.
			.rounding(Rounding::ZERO)
			.show(ui, |ui| {
				ScrollArea::vertical().show(ui, |ui| {
					TableBuilder::new(ui)
						.column(Column::auto().resizable(true).at_least(20.0)) // ID.
						.column(Column::remainder().resizable(true).at_least(250.0)) // Name.
						.columns(Column::auto().resizable(true).at_least(100.0), 2) // Position, Medal.
						.column(Column::auto().resizable(true).at_least(72.0)) // Captain.
						.column(Column::auto().at_least(40.0).at_most(50.0)) // Active.
						.striped(true)
						.cell_layout(Layout::left_to_right(Align::Center))
						.header(20.0, |mut header| {
							header.col(|ui| {
								ui.heading("ID");
							});
							header.col(|ui| {
								ui.heading("Name");
							});
							header.col(|ui| {
								ui.heading("Position");
							});
							header.col(|ui| {
								ui.heading("Medal");
							});
							header.col(|ui| {
								ui.heading("Captain");
							});
							header.col(|ui| {
								ui.heading("Active");
							});
						})
						.body(|body| {
							let row_height = 22.0;
							let num_rows = 23;

							body.rows(row_height, num_rows, |mut row| {
								// ID.
								let row_idx = row.index();
								row.col(|ui| {
									ui.label(format!("{}", self.rows[row_idx].id));
								});
								// Name.
								row.col(|ui| {
									if ui
										.add(
											TextEdit::singleline(&mut self.rows[row_idx].name)
												.desired_width(f32::INFINITY),
										)
										.changed()
									{
										// Might do something in the future.
									}
								});
								// Position.
								row.col(|ui| {
									// NOTE: Hack to fix popup styling.
									ui.ctx().style_mut(|p_ui| {
										Self::set_style(p_ui);
									});
									egui::ComboBox::from_id_source("position{row_idx}")
										.selected_text(&self.rows[row_idx].position.to_string())
										.show_ui(ui, |ui| {
											for pos in Position::VARIANTS {
												ui.selectable_value(
													&mut self.rows[row_idx].position,
													*pos,
													pos.to_string(),
												);
											}
										});
								});
								// Medal.
								row.col(|ui| {
									egui::ComboBox::from_id_source("medal{row_idx}")
										.selected_text(&self.rows[row_idx].medal)
										.show_ui(ui, |ui| {
											ui.selectable_value(
												&mut self.rows[row_idx].medal,
												Medal::None,
												&Medal::None,
											);
											ui.selectable_value(
												&mut self.rows[row_idx].medal,
												Medal::Silver,
												&Medal::Silver,
											);
											ui.selectable_value(
												&mut self.rows[row_idx].medal,
												Medal::Gold,
												&Medal::Gold,
											);
										});

									// NOTE: Reset style after hack.
									ui.ctx().style_mut(|p_ui| {
										*p_ui = Style::default();
									});
									setup_custom_fonts(ui.ctx()) // Setup fonts after resetting style.
								});
								// Captain.
								row.col(|ui| {
									if ui
										.add(egui::RadioButton::new(
											self.rows[row_idx].captain == true,
											format!("{}", &mut self.rows[row_idx].captain),
										))
										.clicked()
									{
										for row in &mut self.rows {
											row.captain = false;
										}

										self.rows[row_idx].captain = true;
									}
								});
								// Active checkbox.
								row.col(|ui| {
									ui.with_layout(Layout::top_down(Align::Center), |ui| {
										ui.add(Checkbox::without_text(
											&mut self.rows[row_idx].active,
										));
									});
								});
							});
						});
				})
			});
	}

	pub fn heading(&mut self, ui: &mut Ui) {
		let mut margin = Self::INNER_MARGIN;
		margin.bottom = 10.0;
		egui::Frame::none().outer_margin(margin).show(ui, |ui| {
			ui.with_layout(Layout::top_down(Align::Center), |ui| {
				ui.label(RichText::new("Roster Editor").size(32.0));
			});
		});
	}

	pub fn menu(&mut self, ui: &mut Ui) {
		egui::Frame::none()
			.inner_margin(Margin::symmetric(Self::INNER_MARGIN.left, 0.0))
			.show(ui, |ui| {
				StripBuilder::new(ui)
					.size(Size::exact(45.0))
					.size(Size::exact(180.0))
					.size(Size::exact(60.0))
					.size(Size::exact(60.0))
					.cell_layout(Layout::left_to_right(Align::Center))
					.horizontal(|mut strip| {
						strip.cell(|ui| {
							Self::set_style(ui.style_mut());
							ui.label("Team:");
						});

						strip.cell(|ui| {
							Self::set_style(ui.style_mut());
							if ui
								.add(
									TextEdit::singleline(&mut self.team)
										.desired_width(f32::INFINITY),
								)
								.changed()
							{
								// Might do something in the future.
							}
						});

						strip.cell(|ui| {
							if widget_creator::button(
								ui,
								"Import",
								Layout::left_to_right(Align::Center),
							)
							.clicked()
							{
								if let Some(path) = FileDialog::new()
									.set_title("Import MSRF roster file")
									.add_filter("Mister Skeleton Roster Format", &["msrf"])
									.add_filter("Tom's Obvious Minimal Language", &["toml"])
									.pick_file()
								{
									match FormatType::from_extension(path.extension()) {
										Some(FormatType::MSRF) => {
											match RosterFile::get_rosterfile(path) {
												Ok(roster_file) => {
													match Roster::from_rosterfile(&roster_file) {
														Ok(roster) => {
															let rows = RosterRow::from_roster(roster);
															self.rows = rows;
															self.team = roster_file.team;
														}
														Err(e) => Message::error_message(
															"Import Error",
															&e.to_string(),
														),
													};
												}
												Err(rosterfile_error) => {
													Message::error_message(
														"Import Error",
														&rosterfile_error.to_string(),
													);
												}
											}
										}
										Some(FormatType::TOML) => {
											match Roster::from_toml(path) {
												Ok(roster) => {
													let rows = RosterRow::from_roster(roster);
													self.rows = rows;
													self.team = "".to_string();
												}
												Err(roster_error) => {
													Message::error_message(
														"Import Error",
														&roster_error.to_string(),
													);
												}
											}
										}
										None => Message::error_message(
											"Import Error",
											"Failed to parse file extension",
										),
									}


								}
							}
						});

						strip.cell(|ui| {
							if widget_creator::button(
								ui,
								"Export",
								Layout::left_to_right(Align::Center),
							)
							.clicked()
							{
								if let Some(save_path) = FileDialog::new()
									.set_title("Export roster file")
									.set_file_name(slugify(&self.team))
									.add_filter("Mister Skeleton Roster Format", &["msrf"])
									.add_filter("Tom's Obvious Minimal Language", &["toml"])
									.save_file()
								{
									match FormatType::from_extension(save_path.extension()) {
										Some(format_type) => {
											if let Err(e) = create_team_file(
												&self.team,
												RosterRow::to_roster(&self.rows),
												&save_path,
												format_type,
											) {
												Message::error_message(
													"Export Error",
													&e.to_string(),
												);
											}
										}
										None => Message::error_message(
											"Export Error",
											"Failed to parse file extension",
										),
									}
								}
							}
						});
					});
			});
	}

	fn set_spacing(spacing: &mut Spacing) {
		let mut scroll = ScrollStyle::solid();
		scroll.bar_width = 16.0;

		spacing.scroll = scroll;
		spacing.item_spacing = Vec2::new(10.0, 8.0);
	}

	fn set_style(style: &mut Style) {
		Self::set_spacing(&mut style.spacing);
		Self::set_visuals(&mut style.visuals);
	}

	fn set_visuals(visuals: &mut Visuals) {
		let mut widgets = Widgets::dark();
		let color = Color32::from_rgb(45, 60, 70);
		let stroke_color = Color32::from_rgb(50, 80, 100);
		let bg_stroke = Stroke::new(1.0, stroke_color);
		let fg_stroke = Stroke::new(3.0, Color32::from_gray(200));
		let rounding = Rounding::same(2.0);

		let selected_color = Color32::from_rgb(40, 100, 150);
		let selected_stroke_color = Color32::from_rgb(120, 200, 250);
		let selected_bg_stroke = Stroke::new(1.0, selected_stroke_color);
		let selected_fg_stroke = Stroke::new(2.0, Color32::WHITE);

		// Controls resizable bars and header/label text.
		let mut non_interactive = widgets.noninteractive;
		non_interactive.bg_stroke = Stroke::new(1.0, Color32::DARK_GRAY);
		non_interactive.rounding = rounding;
		non_interactive.fg_stroke = Stroke::new(1.0, Color32::WHITE);
		widgets.noninteractive = non_interactive;

		// // Controls main combo-box, radio buttons, scrollbar and text in TextEdit.
		let mut inactive = widgets.inactive;
		inactive.bg_fill = color; // Radio button and scrollbar.
		inactive.weak_bg_fill = color; // Combo-box.
		inactive.bg_stroke = bg_stroke;
		inactive.rounding = rounding;
		inactive.fg_stroke = fg_stroke;
		widgets.inactive = inactive;

		// // Controls textfield, main combo-box, radio button, scrollbar when hovered.
		let mut hovered = widgets.hovered;
		hovered.bg_fill = selected_color; // Radio button and scrollbar.
		hovered.weak_bg_fill = selected_color; // Combo-box.
		hovered.bg_stroke = selected_bg_stroke;
		hovered.rounding = rounding;
		hovered.fg_stroke = selected_fg_stroke;
		widgets.hovered = hovered;

		// Controls main combo-box, radio button, scrollbar when clicking.
		let mut active = widgets.active;
		active.bg_fill = selected_color;
		active.weak_bg_fill = selected_color;
		active.bg_stroke = selected_bg_stroke;
		active.rounding = rounding;
		active.fg_stroke = selected_fg_stroke;
		widgets.active = active;

		// Controls main combo-box button when open.
		let mut open = widgets.open;
		open.weak_bg_fill = selected_color;
		open.bg_stroke = selected_bg_stroke;
		open.rounding = rounding;
		open.fg_stroke = selected_fg_stroke;
		widgets.open = open;

		visuals.widgets = widgets;

		visuals.extreme_bg_color = Color32::from_gray(30);
		visuals.faint_bg_color = Color32::from_gray(48);
		visuals.text_cursor = fg_stroke;
		visuals.window_fill = Color32::from_gray(40);
		visuals.window_stroke = Stroke::new(1.0, Color32::DARK_GRAY);
		visuals.window_shadow = Shadow::NONE;
		visuals.selection = Selection {
			bg_fill: selected_color,
			stroke: Stroke {
				color: Color32::WHITE,
				width: 1.0,
			},
		};
	}
}

impl Default for RosterEditor {
	fn default() -> Self {
		let mut rows: [RosterRow; 23] = Default::default();
		for x in 1..=23 {
			rows[x - 1].id = x as u8;
		}

		Self {
			rows,
			team: String::new(),
		}
	}
}

#[derive(Default, Ord, PartialOrd, PartialEq, Eq)]
struct RosterRow {
	id: u8,
	name: String,
	position: Position,
	medal: Medal,
	captain: bool,
	active: bool,
}

impl RosterRow {
	fn from_player(player: lineupper::player::Player, active: bool) -> RosterRow {
		RosterRow {
			id: player.id,
			name: player.name,
			position: player.position,
			medal: Medal::from_lineupper(player.medal),
			captain: match player.captain {
				Some(b) => b,
				None => false,
			},
			active,
		}
	}

	fn from_roster(roster: Roster) -> [RosterRow; 23] {
		// Convert all players to RosterRow, and move reserve and active into 1 vec.
		let mut roster_active: Vec<RosterRow> = roster
			.active
			.into_iter()
			.map(|p| RosterRow::from_player(p, true))
			.collect();
		let mut roster_reserve: Vec<RosterRow> = roster
			.reserve
			.into_iter()
			.map(|p| RosterRow::from_player(p, false))
			.collect();
		roster_active.append(&mut roster_reserve);

		// Sort them by the ID.
		roster_active.sort_by(|a, b| a.id.cmp(&b.id));
		roster_active
			.try_into()
			.unwrap_or_else(|v: Vec<RosterRow>| {
				panic!("Expected Roster of 23 players, found {}", v.len())
			})
	}

	fn to_roster(rows: &[RosterRow]) -> Roster {
		let player_states: Vec<_> = rows
			.iter()
			.map(|player| {
				PlayerState::from(
					player.captain,
					player.id,
					player.medal.to_lineupper(),
					player.name.clone(),
					player.position,
					player.active,
				)
			})
			.collect();
		let mut active_players = Vec::new();
		let mut reserve_players = Vec::new();

		for player_state in player_states {
			match player_state {
				PlayerState::Active(p) => active_players.push(p),
				PlayerState::Reserve(p) => reserve_players.push(p),
			}
		}

		Roster::from_players(active_players, reserve_players)
	}
}

#[derive(Default, PartialEq, Ord, PartialOrd, Eq)]
enum Medal {
	#[default]
	None,
	Silver,
	Gold,
}

impl Medal {
	fn to_lineupper(&self) -> Option<lineupper::player::Medal> {
		match self {
			Medal::Silver => Some(lineupper::player::Medal::Silver),
			Medal::Gold => Some(lineupper::player::Medal::Gold),
			Medal::None => None,
		}
	}

	fn from_lineupper(medal: Option<lineupper::player::Medal>) -> Medal {
		match medal {
			Some(lineupper::player::Medal::Silver) => Medal::Silver,
			Some(lineupper::player::Medal::Gold) => Medal::Gold,
			None => Medal::None,
		}
	}
}

impl From<&Medal> for WidgetText {
	fn from(value: &Medal) -> Self {
		match value {
			Medal::None => "No medal".into(),
			Medal::Silver => "Silver".into(),
			Medal::Gold => "Gold".into(),
		}
	}
}
