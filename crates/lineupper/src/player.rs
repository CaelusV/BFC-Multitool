use anyhow::Result;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, VariantArray};
use thiserror::Error;

#[derive(Serialize, Deserialize, Default, Clone, Copy, Display, VariantArray)]
pub enum Medal {
	#[serde(skip_serializing)]
	#[default]
	None,
	Silver,
	Gold,
}

#[derive(Error, Debug, PartialEq)]
pub(crate) enum PlayerError {
	#[error("'{0}' is an invalid portrait name.")]
	InvalidPortraitName(String),
	#[error("'{0}' has an invalid ID.")]
	InvalidID(String),
	#[error("'{0}' is missing one or more player attributes.")]
	MissingAttributes(String),
	#[error("String isn't a player.")]
	NotAPlayer,
	#[error("'{0}' is not a player position.")]
	NotAPosition(String),
}

pub enum PlayerState {
	Active(Player),
	Reserve(Player),
}

impl PlayerState {
	pub fn from(
		id: u8,
		name: String,
		position: Position,
		medal: Option<Medal>,
		active: bool,
		captain: bool,
		portrait_name: Option<String>,
	) -> Self {
		let captain = match captain {
			true => Some(true),
			false => None,
		};

		let player = Player {
			active,
			captain,
			id,
			medal,
			name,
			portrait_name,
			position,
		};

		if active {
			Self::Active(player)
		} else {
			Self::Reserve(player)
		}
	}

	pub(crate) fn from_string(s: String) -> Result<PlayerState> {
		let parts = s.trim().split("+++").collect::<Vec<_>>();
		if parts.len() < 3 {
			match parts[0].to_ascii_lowercase().starts_with("xxx") {
				true => return Err(PlayerError::MissingAttributes(s).into()),
				false => return Err(PlayerError::NotAPlayer.into()),
			}
		}

		let id =
			if parts[0].trim_end().len() >= 5 && parts[0].to_ascii_lowercase().starts_with("xxx") {
				str::parse::<u8>(&parts[0][3..=4])
					.or_else(|_| return Err(PlayerError::InvalidID(s.clone())))?
			} else {
				return Err(PlayerError::InvalidID(s).into());
			};

		let position = Position::try_from(parts[2].trim())
			.map_err(|_| PlayerError::NotAPosition(parts[2].trim().to_string()))?;

		let mut name = parts[1].to_string();

		let medal = if name.contains("(g)") {
			name = name.replace("(g)", "");
			Some(Medal::Gold)
		} else if name.contains("(s)") {
			name = name.replace("(s)", "");
			Some(Medal::Silver)
		} else {
			None
		};

		let captain = match name.contains("(c)") {
			true => {
				name = name.replace("(c)", "");
				Some(true)
			}
			false => None,
		};

		// Get portrait name.
		let mut portrait_name = None;
		let portrait_name_kw = "[p=";
		let portrait_name_from = name.find(portrait_name_kw);
		let portrait_name_to = match portrait_name_from {
			Some(idx) => name[idx..].find(']'),
			None => None,
		};

		if let (Some(p_from), Some(p_to)) = (portrait_name_from, portrait_name_to) {
			let p_to = p_from + p_to;
			let p_name = name[p_from + portrait_name_kw.len()..p_to].to_string();

			if p_name
				.chars()
				.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
				&& p_name.len() > 0
			{
				portrait_name = Some(p_name);
			} else {
				return Err(PlayerError::InvalidPortraitName(p_name).into());
			}
			name.replace_range(p_from..=p_to, "");
		}

		let mut active = false;
		if name.contains("(a)") {
			name = name.replace("(a)", "");
			active = true;
		}

		let player = Player {
			active,
			captain,
			id,
			medal,
			// Trim name after removing tags from name.
			name: name.trim().to_string(),
			portrait_name,
			position,
		};

		if active {
			return Ok(PlayerState::Active(player));
		}

		Ok(PlayerState::Reserve(player))
	}

	pub(crate) fn to_msrf_string(&self) -> String {
		let (active, player) = match self {
			PlayerState::Active(p) => (true, p),
			PlayerState::Reserve(p) => (false, p),
		};

		let mut tags = String::from(" ");

		if active {
			tags += "(a) ";
		}

		match player.medal {
			Some(Medal::Gold) => tags += "(g) ",
			Some(Medal::Silver) => tags += "(s) ",
			_ => {}
		};

		if let Some(true) = player.captain {
			tags += "(c) "
		}

		if let Some(p) = &player.portrait_name {
			tags += &format!("[p={p}]");
		}

		format!(
			"XXX{:02} +++ {}{} +++ {}",
			player.id,
			player.name,
			tags.trim_end(),
			player.position
		)
	}
}

#[derive(Serialize, Deserialize)]
pub struct Player {
	pub id: u8,
	pub name: String,
	pub position: Position,
	pub medal: Option<Medal>,
	pub captain: Option<bool>,
	#[serde(skip)]
	pub active: bool,
	pub portrait_name: Option<String>,
}

impl Player {}

#[derive(
	Serialize,
	Deserialize,
	VariantArray,
	Default,
	Debug,
	Display,
	EnumString,
	Clone,
	Copy,
	Ord,
	PartialOrd,
	Eq,
	PartialEq,
)]
pub enum Position {
	GK,
	LB,
	#[default]
	CB,
	RB,
	DMF,
	LMF,
	CMF,
	RMF,
	AMF,
	LWF,
	RWF,
	SS,
	CF,
}
