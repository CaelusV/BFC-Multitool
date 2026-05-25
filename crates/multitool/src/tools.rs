use std::{ffi::OsStr, path::PathBuf};

use common::{errors::ToolError, Progress};
use iced::{
	font, task,
	widget::{button, column, progress_bar, row, text},
	Alignment::Center,
	Element, Font, Task,
};
use strum_macros::Display;

use crate::{messenger::Messenger, Message, MARGIN};

#[derive(Clone, Copy, Display)]
pub enum Tool {
	LineUpper,
	Statter,
}

#[derive(Default)]
pub struct ToolState {
	pub source: Option<PathBuf>,
	pub destination: Option<PathBuf>,
	state: WorkState,
}

#[derive(Default, Clone)]
enum WorkState {
	#[default]
	Idle,
	Working {
		progress: f32,
		_task: task::Handle,
	},
	Finished,
	Errored,
}

#[derive(Debug, Clone)]
pub enum WorkUpdate {
	Working(Progress),
	Finished(Result<(), ToolError>),
}

#[derive(Default)]
pub struct Tools {
	pub lineupper: ToolState,
	pub statter: ToolState,
}

impl Tools {
	pub fn start(&mut self, tool: Tool, source: PathBuf, destination: PathBuf) -> Task<WorkUpdate> {
		let tool_state = match tool {
			Tool::LineUpper => &mut self.lineupper,
			Tool::Statter => &mut self.statter,
		};
		match tool_state.state {
			WorkState::Idle | WorkState::Finished | WorkState::Errored => {
				let (task, handle) = match tool {
					Tool::LineUpper => Task::sip(
						lineupper::create::create_team_and_portraits(
							source.clone(),
							destination.clone(),
						),
						WorkUpdate::Working,
						WorkUpdate::Finished,
					)
					.abortable(),
					Tool::Statter => Task::sip(
						statter::entry::run_tournaments(source.clone(), destination.clone()),
						WorkUpdate::Working,
						WorkUpdate::Finished,
					)
					.abortable(),
				};

				tool_state.state = WorkState::Working {
					progress: 0.0,
					_task: handle.abort_on_drop(),
				};

				task
			}
			WorkState::Working { .. } => Task::none(),
		}
	}

	pub fn update(&mut self, tool: Tool, update: WorkUpdate) {
		let work_state = match tool {
			Tool::LineUpper => &mut self.lineupper.state,
			Tool::Statter => &mut self.statter.state,
		};

		if let WorkState::Working { progress, .. } = work_state {
			match update {
				WorkUpdate::Working(new_progress) => *progress = new_progress.percent,
				WorkUpdate::Finished(result) => {
					*work_state = match result {
						Ok(_) => WorkState::Finished,
						Err(e) => {
							Messenger::error_message("Run Error", &e.to_string());
							WorkState::Errored
						}
					};
				}
			}
		}
	}

	pub fn tools(&self) -> Element<'_, Message> {
		row![
			self.tool_section(Tool::LineUpper),
			self.tool_section(Tool::Statter)
		]
		.padding(MARGIN * 2.0)
		.spacing(MARGIN * 8.0)
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

		let (source_path, destination_path, work_state, header) = match tool {
			Tool::LineUpper => (
				self.lineupper.source.as_ref(),
				self.lineupper.destination.as_ref(),
				self.lineupper.state.clone(),
				header("LineUpper"),
			),
			Tool::Statter => (
				self.statter.source.as_ref(),
				self.statter.destination.as_ref(),
				self.statter.state.clone(),
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

		let mut run_button = button("Run!")
			.style(button::success)
			.on_press(Message::RunTool(tool));

		let (progress, tool_status_text) = match work_state {
			WorkState::Idle | WorkState::Errored
				if source_path.is_some() && destination_path.is_some() =>
			{
				(0.0, "Ready!".to_string())
			}
			WorkState::Idle | WorkState::Errored => {
				run_button = button("Run!").style(button::success);
				(0.0, "Missing path(s).".to_string())
			}
			WorkState::Working { progress, _task } => (progress, format!("{progress:.2}%")),
			WorkState::Finished => (100.0, "Done!".to_string()),
		};

		let button_row = row![run_button, text(tool_status_text),]
			.align_y(Center)
			.spacing(MARGIN * 2.0);

		column![
			header.align_y(Center),
			browse_source,
			browse_destination,
			progress_bar(0.0..=100.0, progress),
			button_row,
		]
		.spacing(MARGIN * 2.0)
		.into()
	}
}
