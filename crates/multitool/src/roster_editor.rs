use std::fmt::Display;

use iced::{
	font,
	widget::{button, column, combo_box, radio, row, scrollable, table, text, text_input, toggler},
	Alignment::Center,
	Element, Font,
};
use strum::VariantArray;

use lineupper::{
	player::{Medal, Player, PlayerState, Position},
	roster::Roster,
};

use crate::{Message, CHECKBOX_WIDTH, COMBO_BOX_WIDTH, MARGIN, RADIO_WIDTH, TEXT_INPUT_WIDTH};

pub struct RosterEditor {
	pub rows: [RosterRow; 23],
	pub captain: Option<usize>,
	pub team: String,
}

impl RosterEditor {
	const NAME_PLACEHOLDERS: [&'static str; 23] = [
		"Gianluigi Buffon",
		"Cristian Zaccardo",
		"Fabio Grosso",
		"Daniele De Rossi",
		"Fabio Cannavaro",
		"Andrea Barzagli",
		"Alessandro Del Piero",
		"Gennaro Gattuso",
		"Luca Toni",
		"Francesco Totti",
		"Alberto Gilardino",
		"Angelo Peruzzi",
		"Alessandro Nesta",
		"Marco Amelia",
		"Vincenzo Iaquinta",
		"Mauro Camoranesi",
		"Simone Barone",
		"Filippo Inzaghi",
		"Gianluca Zambrotta",
		"Simone Perrotta",
		"Andrea Pirlo",
		"Massimo Oddo",
		"Marco Materazzi",
	];

	pub fn editor(&self) -> Element<'_, Message> {
		let menu = column![row![
			text("Team:"),
			text_input("Ex: Italy", &self.team)
				.width(TEXT_INPUT_WIDTH)
				.on_input(Message::TeamNameChanged)
				.on_paste(Message::TeamNameChanged),
			button("Import").on_press(Message::ImportPressed),
			button("Export").on_press(Message::ExportPressed),
		]
		.align_y(Center)
		.spacing(MARGIN)]
		.align_x(Center);

		let bold = |header| {
			text(header)
				.font(Font {
					weight: font::Weight::Bold,
					..Font::DEFAULT
				})
				.center()
		};

		let table_columns = [
			table::column(bold("ID"), |row: &RosterRow| text(&row.id))
				.align_x(Center)
				.align_y(Center),
			table::column(bold("Name"), |row: &RosterRow| {
				let current_row = row.id as usize - 1; // Really ugly having to do this. Too bad!
				text_input(
					&format!("Ex: {}", Self::NAME_PLACEHOLDERS[current_row]),
					&row.name,
				)
				.on_input(move |n| Message::NameChanged(n, current_row))
			})
			.width(TEXT_INPUT_WIDTH)
			.align_x(Center)
			.align_y(Center),
			table::column(bold("Position"), |row: &RosterRow| {
				let current_row = row.id as usize - 1; // Really ugly having to do this. Too bad!
				combo_box(
					&row.position.state,
					"Position...",
					row.position.selected.as_ref(),
					move |p| Message::PositionChanged(p, current_row),
				)
			})
			.width(COMBO_BOX_WIDTH)
			.align_x(Center)
			.align_y(Center),
			table::column(bold("Medal"), |row: &RosterRow| {
				let current_row = row.id as usize - 1; // Really ugly having to do this. Too bad!
				combo_box(
					&row.medal.state,
					"Medal...",
					row.medal.selected.as_ref(),
					move |m| Message::MedalChanged(m, current_row),
				)
			})
			.width(COMBO_BOX_WIDTH)
			.align_x(Center)
			.align_y(Center),
			table::column(bold("Active"), |row: &RosterRow| {
				let current_row = row.id as usize - 1; // Really ugly having to do this. Too bad!
				toggler(row.active).on_toggle(move |c| Message::ActiveChanged(c, current_row))
			})
			.width(CHECKBOX_WIDTH)
			.align_x(Center)
			.align_y(Center),
			table::column(bold("Captain"), |row: &RosterRow| {
				let current_row = row.id as usize - 1; // Really ugly having to do this. Too bad!
				radio("", current_row, self.captain, Message::CaptainChanged)
			})
			.width(RADIO_WIDTH)
			.align_x(Center)
			.align_y(Center),
			table::column(bold("Portrait Name (optional)"), |row: &RosterRow| {
				let current_row = row.id as usize - 1; // Really ugly having to do this. Too bad!
				text_input(&format!("Ex: portrait{}", &row.id), &row.portrait_name)
					.on_input(move |n| Message::PortraitNameChanged(n, current_row))
			})
			.width(TEXT_INPUT_WIDTH)
			.align_x(Center)
			.align_y(Center),
		];

		let table = table(table_columns, &self.rows).padding(MARGIN * 0.7);
		let scrollable_table = scrollable(table).spacing(MARGIN);
		column![menu, scrollable_table]
			.align_x(Center)
			.spacing(MARGIN * 2.0)
			.into()
	}
}

impl Default for RosterEditor {
	fn default() -> Self {
		let mut rows: [RosterRow; 23] = Default::default();
		for x in 0..23 {
			rows[x].id = x as u8 + 1;
		}

		Self {
			rows,
			captain: None,
			team: String::new(),
		}
	}
}

pub struct ComboState<A> {
	state: combo_box::State<A>,
	selected: Option<A>,
}

impl<A: Clone + Default + Display + VariantArray> Default for ComboState<A> {
	fn default() -> Self {
		ComboState {
			state: combo_box::State::new(A::VARIANTS.to_vec()),
			selected: Some(A::default()),
		}
	}
}

impl<A: Clone + Display + VariantArray> ComboState<A> {
	fn new(selected: Option<A>) -> Self {
		ComboState {
			state: combo_box::State::new(A::VARIANTS.to_vec()),
			selected,
		}
	}

	pub fn set(&mut self, new_select: A) {
		self.selected = Some(new_select);
	}
}

#[derive(Default)]
pub struct RosterRow {
	pub id: u8,
	pub name: String,
	pub position: ComboState<Position>,
	pub medal: ComboState<Medal>,
	pub active: bool,
	pub portrait_name: String,
}

impl RosterRow {
	fn from_player(player: Player) -> RosterRow {
		RosterRow {
			id: player.id,
			name: player.name,
			position: ComboState::new(Some(player.position)),
			medal: ComboState::new(Some(player.medal.unwrap_or_default())),
			active: player.active,
			portrait_name: player.portrait_name.unwrap_or_default(),
		}
	}

	pub fn from_roster(roster: Roster) -> ([RosterRow; 23], Option<usize>) {
		let mut captain = None;
		for p in roster.reserve.iter().chain(roster.active.iter()) {
			if p.captain.is_some_and(|b| b) {
				captain = Some(p.id as usize - 1);
			}
		}

		// Convert all players to RosterRow, and move reserve and active into 1 vec.
		let mut roster: Vec<RosterRow> = roster
			.active
			.into_iter()
			.chain(roster.reserve.into_iter())
			.map(|p| RosterRow::from_player(p))
			.collect();

		// Sort them by the ID.
		roster.sort_by(|a, b| a.id.cmp(&b.id));
		let roster = roster.try_into().unwrap_or_else(|v: Vec<RosterRow>| {
			panic!("Expected Roster of 23 players, found {}", v.len())
		});
		(roster, captain)
	}

	pub fn to_roster(rows: &[RosterRow], captain: Option<usize>) -> Roster {
		let player_states: Vec<_> = rows
			.iter()
			.map(|player| {
				PlayerState::from(
					player.id,
					player.name.clone(),
					player.position.selected.unwrap_or_default(),
					player.medal.selected,
					player.active,
					match captain {
						Some(r) => r == player.id as usize - 1,
						None => false,
					},
					match player.portrait_name.as_str() {
						"" => None,
						_ => Some(player.portrait_name.clone()),
					},
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
