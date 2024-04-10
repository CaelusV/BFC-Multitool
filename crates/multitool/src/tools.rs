use core::fmt;
use std::{ffi::OsStr, path::PathBuf};

use eframe::egui::{self, Align, Color32, Layout, Margin, Response, Ui};
use egui_extras::{Size, StripBuilder};

use crate::{message::Message, widget_creator};

#[derive(Default)]
pub struct Tools {
	lineupper_target_path: Option<PathBuf>,
	statter_target_path: Option<PathBuf>,
}

impl Tools {
	fn browse(&mut self, ui: &mut Ui, tool: &ToolItem) {
		if widget_creator::button(ui, "Browse", Layout::left_to_right(Align::Center)).clicked() {
			if let Some(path) = rfd::FileDialog::new().pick_folder() {
				match tool {
					ToolItem::LineUpper => self.lineupper_target_path = Some(path),
					ToolItem::Statter => self.statter_target_path = Some(path),
				}
			}
		}
	}

	pub fn hstrip(&mut self, tool: ToolItem, ui: &mut Ui) -> Response {
		StripBuilder::new(ui)
			.size(Size::exact(65.0))
			.size(Size::remainder().at_least(170.0))
			.size(Size::exact(65.0))
			.horizontal(|mut strip| {
				strip.cell(|ui| {
					self.browse(ui, &tool);
				});

				strip.cell(|ui| {
					self.progress(ui, &tool);
				});

				strip.cell(|ui| self.run(ui, &tool));
			})
	}

	// FIXME: Currently nothing is async, so progress won't be shown. Ironically.
	fn progress(&mut self, ui: &mut Ui, tool: &ToolItem) {
		egui::Frame::none()
			.fill(Color32::LIGHT_GRAY)
			.inner_margin(Margin::same(4.0))
			.show(ui, |ui| {
				ui.with_layout(
					Layout::left_to_right(egui::Align::Min)
						.with_main_align(egui::Align::Center)
						.with_main_justify(true),
					|ui| {
						let path = match tool {
							ToolItem::LineUpper => &self.lineupper_target_path,
							ToolItem::Statter => &self.statter_target_path,
						};

						let folder = match &path {
							Some(path) => path
								.file_name()
								.unwrap_or(OsStr::new(".."))
								.to_str()
								.unwrap_or("Folder name unparsable"),
							None => "No folder targeted",
						};

						ui.colored_label(Color32::BLACK, format!("{} ({folder})", tool));
					},
				);
			});
	}

	fn run(&mut self, ui: &mut Ui, tool: &ToolItem) {
		if widget_creator::button(ui, "Run", Layout::left_to_right(Align::Center)).clicked() {
			let path = match tool {
				&ToolItem::LineUpper if self.lineupper_target_path.is_some() => {
					self.lineupper_target_path.as_ref().unwrap()
				}
				&ToolItem::Statter if self.statter_target_path.is_some() => {
					self.statter_target_path.as_ref().unwrap()
				}
				_ => {
					Message::error_message("Run Error", "No folder was targeted.");
					return;
				}
			};

			if let Some(output_path) = rfd::FileDialog::new()
				.set_title("Choose location to save output")
				.pick_folder()
			{
				let error;
				match tool {
					&ToolItem::LineUpper => {
						error = lineupper::create::create_team_and_portraits(path, &output_path)
					}
					&ToolItem::Statter => {
						error = statter::entry::run_tournaments(path, &output_path)
					}
				}
				if let Err(e) = error {
					Message::error_message("Run Error", &e.to_string());
				}
			}
		}
	}
}

#[derive(Debug, PartialEq)]
pub enum ToolItem {
	LineUpper,
	Statter,
}

impl fmt::Display for ToolItem {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}
