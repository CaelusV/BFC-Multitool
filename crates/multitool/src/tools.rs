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
		row![
			self.tool_section(Tool::LineUpper),
			self.tool_section(Tool::Statter)
		]
		.padding(MARGIN * 2.0)
		.spacing(MARGIN * 24.0)
		.into()
	}

	fn tool_section(&self, tool: Tool) -> Element<'_, Message> {
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

		let (source_path, destination_path, header) = match tool {
			Tool::LineUpper => (
				self.lineupper_source.as_ref(),
				self.lineupper_destination.as_ref(),
				header("LineUpper"),
			),
			Tool::Statter => (
				self.statter_source.as_ref(),
				self.statter_destination.as_ref(),
				header("Statter"),
			),
		};

		let browse_source = row![
			button("Source...").on_press(Message::BrowseSource(tool)),
			text(folder_name(source_path))
		]
		.spacing(MARGIN * 2.0)
		.align_y(Center);

		let browse_destination = row![
			button("Destination...").on_press(Message::BrowseDestination(tool)),
			text(folder_name(destination_path))
		]
		.spacing(MARGIN * 2.0)
		.align_y(Center);

		column![
			header.align_y(Center),
			browse_source,
			browse_destination,
			if source_path.is_some() && destination_path.is_some() {
				button("Run!")
					.style(button::success)
					.on_press(Message::Run(tool))
			} else {
				button("Run").style(button::success)
			},
		]
		.spacing(MARGIN * 2.0)
		.into()
	}
}

#[derive(Clone, Copy, Display)]
pub enum Tool {
	LineUpper,
	Statter,
}
