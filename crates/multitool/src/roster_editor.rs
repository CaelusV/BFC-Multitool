use eframe::egui::{
	self, Align, Checkbox, CornerRadius, Layout, Margin, Rangef, RichText, TextEdit, Ui, WidgetText,
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

use crate::{message::Message, widget_creator};

pub struct RosterEditor {
	rows: [RosterRow; 23],
	team: String,
}

impl RosterEditor {
	const INNER_MARGIN: Margin = Margin::same(5);

	pub fn editor(&mut self, ui: &mut Ui) {
		egui::Frame::window(ui.style())
			.inner_margin(Margin::symmetric(6, Self::INNER_MARGIN.left)) // x breaks striped if not same as inner_margin, or if spacing.x too high.
			.corner_radius(CornerRadius::ZERO)
			.show(ui, |ui| {
				TableBuilder::new(ui)
					.column(Column::auto()) // ID.
					.column(Column::remainder().range(Rangef::new(100.0, 300.0))) // Name.
					.columns(Column::auto(), 2) // Position, Medal.
					.column(Column::auto()) // Captain.
					.column(Column::auto()) // Active.
					.striped(true)
					.auto_shrink(true)
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
								egui::ComboBox::from_id_salt("position{row_idx}")
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
								egui::ComboBox::from_id_salt("medal{row_idx}")
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
									ui.add(Checkbox::without_text(&mut self.rows[row_idx].active));
								});
							});
						});
					});
			});
	}

	pub fn heading(&mut self, ui: &mut Ui) {
		let mut margin = Self::INNER_MARGIN;
		margin.bottom = 10;
		egui::Frame::NONE.outer_margin(margin).show(ui, |ui| {
			ui.with_layout(Layout::top_down(Align::Center), |ui| {
				ui.label(RichText::new("Roster Editor").size(32.0));
			});
		});
	}

	pub fn menu(&mut self, ui: &mut Ui) {
		egui::Frame::NONE
			.inner_margin(Margin::symmetric(Self::INNER_MARGIN.left, 0))
			.show(ui, |ui| {
				ui.with_layout(Layout::top_down(Align::Center), |ui| {
					StripBuilder::new(ui)
						.size(Size::exact(45.0))
						.size(Size::exact(180.0))
						.size(Size::exact(60.0))
						.size(Size::exact(60.0))
						.cell_layout(Layout::left_to_right(Align::Center))
						.horizontal(|mut strip| {
							strip.cell(|ui| {
								ui.label("Team:");
							});

							strip.cell(|ui| {
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
														match Roster::from_rosterfile(&roster_file)
														{
															Ok(roster) => {
																let rows =
																	RosterRow::from_roster(roster);
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
											Some(FormatType::TOML) => match Roster::from_toml(path)
											{
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
											},
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
													&save_path.parent().unwrap(),
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
				})
			});
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
