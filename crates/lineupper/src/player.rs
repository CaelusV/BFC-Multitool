use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use strum_macros::VariantArray;
use thiserror::Error;

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum Medal {
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
}

#[derive(PartialEq)]
pub enum PlayerState {
	Active(Player),
	Reserve(Player),
}

impl PlayerState {
	pub fn from(
		captain: bool,
		id: u8,
		medal: Option<Medal>,
		name: String,
		position: Position,
		active: bool,
	) -> Self {
		let captain = match captain {
			true => Some(true),
			false => None,
		};

		let player = Player {
			captain,
			id,
			medal,
			name,
			portrait_name: None,
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

		let position = Position::from(parts[2].trim())?;

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
		let active;

		let player = match self {
			PlayerState::Active(p) => {
				active = true;
				p
			}
			PlayerState::Reserve(p) => {
				active = false;
				p
			}
		};

		let mut tags = if active {
			String::from(" (a) ")
		} else {
			String::from(" ")
		};

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

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Player {
	pub captain: Option<bool>,
	pub id: u8,
	pub medal: Option<Medal>,
	pub name: String,
	pub portrait_name: Option<String>,
	pub position: Position,
}

impl Player {}

#[derive(Serialize, Deserialize, VariantArray, Default, Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
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

impl Position {
	fn from(str: &str) -> Result<Self> {
		Ok(match str {
			"GK" => Self::GK,
			"LB" => Self::LB,
			"CB" => Self::CB,
			"RB" => Self::RB,
			"DMF" => Self::DMF,
			"LMF" => Self::LMF,
			"CMF" => Self::CMF,
			"RMF" => Self::RMF,
			"AMF" => Self::AMF,
			"LWF" => Self::LWF,
			"RWF" => Self::RWF,
			"SS" => Self::SS,
			"CF" => Self::CF,
			_ => return Err(anyhow!("'{str}' is not a player position.")),
		})
	}
}

impl std::fmt::Display for Position {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{self:?}")
	}
}
