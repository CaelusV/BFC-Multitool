use std::{ffi::OsStr, path::PathBuf};

use eframe::egui::{self, Align, Color32, Layout, Margin, Response, Ui};
use egui_extras::{Size, StripBuilder};

use crate::widget_creator;

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

						ui.colored_label(Color32::BLACK, format!("{} ({folder})", tool.to_str()));
					},
				);
			});
	}

	fn run(&mut self, ui: &mut Ui, tool: &ToolItem) {
		if widget_creator::button(ui, "Run", Layout::left_to_right(Align::Center)).clicked() {
			if let Some(output_path) = rfd::FileDialog::new()
				.set_title("Choose location to save output folder")
				.pick_folder()
			{
				match tool {
					&ToolItem::Statter => {
						if let Some(path) = &self.statter_target_path {
							statter::entry::run_tournaments(path, &output_path);
						}
					}
					&ToolItem::LineUpper => {
						if let Some(path) = &self.lineupper_target_path {
							lineupper::create::create_team_and_portraits(path, &output_path)
						}
					}
				}
			}
		}
	}
}

pub enum ToolItem {
	LineUpper,
	Statter,
}

impl ToolItem {
	fn to_str(&self) -> &'static str {
		match self {
			ToolItem::LineUpper => "LineUpper",
			ToolItem::Statter => "Statter",
		}
	}
}
