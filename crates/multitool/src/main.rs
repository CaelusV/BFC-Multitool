#![windows_subsystem = "windows"] // Hide console window on Windows in release.

use iced::{
	widget::{button, column, container},
	window::Settings,
	Alignment::{Center, Start},
	Element, Function,
	Length::Fill,
	Task,
};
use rfd::FileDialog;
use tokio::runtime::Runtime;

use lineupper::{
	create::{create_team_file, FormatType},
	roster::{Roster, RosterFile},
	slugify,
};
use multitool::{
	messenger::Messenger,
	roster_editor::{RosterEditor, RosterRow},
	tools::{Tool, Tools},
	Message, MARGIN,
};
use strum_macros::Display;

static ICON: &[u8] = include_bytes!("../icon.png");
const ICON_HEIGHT: u32 = 256;
const ICON_WIDTH: u32 = 256;

fn main() -> iced::Result {
	let image = image::load_from_memory(ICON).unwrap();
	let icon =
		iced::window::icon::from_rgba(image.as_bytes().to_vec(), ICON_WIDTH, ICON_HEIGHT).unwrap();

	let settings = Settings {
		size: (850, 800).into(),
		min_size: Some((850, 400).into()),
		position: iced::window::Position::Centered,
		visible: true,
		resizable: true,
		closeable: true,
		minimizable: true,
		decorations: true,
		level: iced::window::Level::Normal,
		icon: Some(icon),
		..Default::default()
	};

	iced::application(Application::default, Application::update, Application::view)
		.antialiasing(true)
		.title(Application::title)
		.window(settings)
		.run()
}

#[derive(Default, Display)]
enum Page {
	#[default]
	RosterEditor,
	Tools,
}

struct Application {
	roster_editor: RosterEditor,
	tools: Tools,
	page: Page,
	// Just so I can block on some async functions.
	runtime: Runtime,
}

impl Application {
	fn title(&self) -> String {
		const VERSION: &str = env!("CARGO_PKG_VERSION");
		format!("BFC Multitool {VERSION} by CaelusV – {}", self.page)
	}

	fn update(&mut self, message: Message) -> Task<Message> {
		match message {
			Message::SwitchToRosterEditor => {
				self.page = Page::RosterEditor;
				Task::none()
			}
			Message::SwitchToTools => {
				self.page = Page::Tools;
				Task::none()
			}
			Message::NameChanged(row, name) => {
				self.roster_editor.rows[row].name = name;
				Task::none()
			}
			Message::PositionChanged(row, position) => {
				self.roster_editor.rows[row].position.set(position);
				Task::none()
			}
			Message::MedalChanged(row, medal) => {
				self.roster_editor.rows[row].medal.set(medal);
				Task::none()
			}
			Message::CaptainChanged(row) => {
				self.roster_editor.captain = Some(row);
				Task::none()
			}
			Message::ActiveChanged(row, is_active) => {
				self.roster_editor.rows[row].active = is_active;
				Task::none()
			}
			Message::PortraitNameChanged(row, pname) => {
				self.roster_editor.rows[row].portrait_name = pname;
				Task::none()
			}
			Message::TeamNameChanged(tname) => {
				self.roster_editor.team = tname;
				Task::none()
			}
			Message::ImportPressed => {
				if let Some(path) = FileDialog::new()
					.set_title("Import MSRF roster file")
					.add_filter("Mister Skeleton Roster Format", &["msrf"])
					.add_filter("Tom's Obvious Minimal Language", &["toml"])
					.pick_file()
				{
					match FormatType::from_extension(path.extension()) {
						Some(FormatType::MSRF) => {
							match self.runtime.block_on(RosterFile::get_rosterfile(path)) {
								Ok(roster_file) => {
									match self
										.runtime
										.block_on(Roster::from_rosterfile(&roster_file))
									{
										Ok(roster) => {
											let (rows, captain) = RosterRow::from_roster(roster);
											self.roster_editor.rows = rows;
											self.roster_editor.captain = captain;
											self.roster_editor.team = roster_file.team;
										}
										Err(e) => {
											Messenger::error_message("Import Error", &e.to_string())
										}
									};
								}
								Err(rosterfile_error) => {
									Messenger::error_message(
										"Import Error",
										&rosterfile_error.to_string(),
									);
								}
							}
						}
						Some(FormatType::TOML) => match self
							.runtime
							.block_on(Roster::from_toml(path))
						{
							Ok(roster) => {
								let (rows, captain) = RosterRow::from_roster(roster);
								self.roster_editor.rows = rows;
								self.roster_editor.captain = captain;
								self.roster_editor.team = "".to_string();
							}
							Err(roster_error) => {
								Messenger::error_message("Import Error", &roster_error.to_string());
							}
						},
						None => Messenger::error_message(
							"Import Error",
							"Failed to parse file extension",
						),
					}
				}
				Task::none()
			}
			Message::ExportPressed => {
				if let Some(save_path) = FileDialog::new()
					.set_title("Export roster file")
					.set_file_name(slugify(&self.roster_editor.team))
					.add_filter("Mister Skeleton Roster Format", &["msrf"])
					.add_filter("Tom's Obvious Minimal Language", &["toml"])
					.save_file()
				{
					match FormatType::from_extension(save_path.extension()) {
						Some(format_type) => {
							if let Err(e) = self.runtime.block_on(create_team_file(
								&self.roster_editor.team,
								RosterRow::to_roster(
									&self.roster_editor.rows,
									self.roster_editor.captain,
								),
								&save_path.parent().unwrap(),
								format_type,
							)) {
								Messenger::error_message("Export Error", &e.to_string());
							}
						}
						None => Messenger::error_message(
							"Export Error",
							"Failed to parse file extension",
						),
					}
				}
				Task::none()
			}
			Message::BrowseSource(tool) => {
				if let Some(path) = rfd::FileDialog::new().pick_folder() {
					match tool {
						Tool::LineUpper => self.tools.lineupper.source = Some(path),
						Tool::Statter => self.tools.statter.source = Some(path),
					}
				}
				Task::none()
			}
			Message::BrowseDestination(tool) => {
				if let Some(path) = rfd::FileDialog::new().pick_folder() {
					match tool {
						Tool::LineUpper => self.tools.lineupper.destination = Some(path),
						Tool::Statter => self.tools.statter.destination = Some(path),
					}
				}
				Task::none()
			}
			Message::RunTool(tool) => {
				let (source, destination) = match tool {
					Tool::LineUpper => (
						self.tools.lineupper.source.as_ref(),
						self.tools.lineupper.destination.as_ref(),
					),
					Tool::Statter => (
						self.tools.statter.source.as_ref(),
						self.tools.statter.destination.as_ref(),
					),
				};

				let task = if let (Some(source), Some(destination)) = (source, destination) {
					self.tools.start(tool, source.clone(), destination.clone())
				} else {
					Messenger::info_message(
						"Run Error",
						"A source or destination folder wasn't selected.",
					);
					return Task::none();
				};
				task.map(Message::UpdateTool.with(tool))
			}
			Message::UpdateTool(tool, update) => {
				self.tools.update(tool, update);
				Task::none()
			}
		}
	}

	fn view(&self) -> Element<'_, Message> {
		let content: Element<Message> = match self.page {
			Page::RosterEditor => {
				column![
					column![button("Tools")
						.style(button::secondary)
						.on_press(Message::SwitchToTools)]
					.width(Fill)
					.align_x(Start),
					self.roster_editor.editor(),
				]
			}
			Page::Tools => {
				column![
					column![button("Roster Editor")
						.style(button::secondary)
						.on_press(Message::SwitchToRosterEditor)]
					.width(Fill)
					.align_x(Start),
					self.tools.tools(),
				]
			}
		}
		.align_x(Center)
		.spacing(MARGIN)
		.width(Fill)
		.into();

		container(content).padding(MARGIN * 2.0).into()
	}
}

impl Default for Application {
	fn default() -> Self {
		Application {
			roster_editor: RosterEditor::default(),
			tools: Tools::default(),
			page: Page::default(),
			runtime: Runtime::new().unwrap(),
		}
	}
}
