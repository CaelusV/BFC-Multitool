use lineupper::player::{Medal, Position};
use tools::Tool;

pub mod messenger;
pub mod roster_editor;
pub mod tools;

pub const CHECKBOX_WIDTH: f32 = 60.0;
pub const COMBO_BOX_WIDTH: f32 = 100.0;
pub const RADIO_WIDTH: f32 = 70.0;
pub const TEXT_INPUT_WIDTH: f32 = 200.0;
pub const MARGIN: f32 = 6.0;

#[derive(Clone)]
pub enum Message {
	SwitchToRosterEditor,
	SwitchToTools,
	NameChanged(usize, String),
	PositionChanged(usize, Position),
	MedalChanged(usize, Medal),
	CaptainChanged(usize),
	ActiveChanged(usize, bool),
	PortraitNameChanged(usize, String),
	TeamNameChanged(String),
	ImportPressed,
	ExportPressed,
	BrowseSource(Tool),
	BrowseDestination(Tool),
	RunTool(Tool),
}
