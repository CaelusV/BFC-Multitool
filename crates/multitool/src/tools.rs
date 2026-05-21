use std::{ffi::OsStr, path::PathBuf};

use iced::{
	font,
	widget::{button, column, row, text},
	Alignment::Center,
	Element, Font,
};
use strum_macros::Display;

use crate::{Message, MARGIN};

#[derive(Default)]
pub struct Tools {
	pub lineupper_source: Option<PathBuf>,
	pub lineupper_destination: Option<PathBuf>,
	pub statter_source: Option<PathBuf>,
	pub statter_destination: Option<PathBuf>,
}

impl Tools {
	pub fn tools(&self) -> Element<'_, Message> {
		let folder_name = |path: Option<&PathBuf>| match path {
			Some(path) => path
				.file_name()
				.unwrap_or(OsStr::new(".."))
				.to_str()
				.unwrap_or("Folder name unparsable")
				.to_owned(),
			None => "No folder targeted".to_owned(),
		};

		let header = |header| {
			text(header)
				.font(Font {
					weight: font::Weight::Bold,
					..Font::DEFAULT
				})
				.size(24)
				.center()
		};

		let run_button = || button("Run").style(button::success);

		// ########### LINEUPPER ############
		let lineupper_browse_source = row![
			button("Source...").on_press(Message::BrowseSource(Tool::LineUpper)),
			text(folder_name(self.lineupper_source.as_ref()))
		]
		.spacing(MARGIN * 2.0)
		.align_y(Center);

		let lineupper_browse_destination = row![
			button("Destination...").on_press(Message::BrowseDestination(Tool::LineUpper)),
			text(folder_name(self.lineupper_destination.as_ref()))
		]
		.spacing(MARGIN * 2.0)
		.align_y(Center);

		let lineupper = column![
			row![
				header("LineUpper"),
				if self.lineupper_source.is_some() && self.lineupper_destination.is_some() {
					run_button().on_press(Message::Run(Tool::LineUpper))
				} else {
					run_button()
				}
			]
			.spacing(MARGIN * 2.0)
			.align_y(Center),
			lineupper_browse_source,
			lineupper_browse_destination,
		]
		.spacing(MARGIN);

		// ########### STATTER ############
		let statter_browse_source = row![
			button("Source...").on_press(Message::BrowseSource(Tool::Statter)),
			text(folder_name(self.statter_source.as_ref()))
		]
		.spacing(MARGIN * 2.0)
		.align_y(Center);

		let statter_browse_destination = row![
			button("Destination...").on_press(Message::BrowseDestination(Tool::Statter)),
			text(folder_name(self.statter_destination.as_ref()))
		]
		.spacing(MARGIN * 2.0)
		.align_y(Center);

		let statter = column![
			row![
				header("Statter"),
				if self.statter_source.is_some() && self.statter_destination.is_some() {
					run_button().on_press(Message::Run(Tool::Statter))
				} else {
					run_button()
				}
			]
			.spacing(MARGIN * 2.0)
			.align_y(Center),
			statter_browse_source,
			statter_browse_destination,
		]
		.spacing(MARGIN);

		column![lineupper, statter,].spacing(MARGIN * 3.0).into()
	}
}

#[derive(Clone, Display)]
pub enum Tool {
	LineUpper,
	Statter,
}
