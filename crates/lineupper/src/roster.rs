use std::{
	ffi::OsString,
	fs::{self, File},
	io::{BufRead, BufReader},
	path::{Path, PathBuf},
};

use crate::player::{Player, PlayerError, PlayerState};
use anyhow::{anyhow, Result};
use serde::Serialize;
use thiserror::Error;

#[derive(Serialize)]
pub struct Roster {
	pub active: Vec<Player>,
	pub reserve: Vec<Player>,
}

impl Roster {
	pub fn from(roster_file: &RosterFile) -> Result<Roster> {
		let file = File::open(&roster_file.path)?;
		let reader = BufReader::new(file);
		let mut active_players = Vec::new();
		let mut reserve_players = Vec::new();

		for line in reader.lines() {
			match line {
				Ok(l) => match PlayerState::from_string(l) {
					Ok(PlayerState::Active(p)) => active_players.push(p),
					Ok(PlayerState::Reserve(p)) => reserve_players.push(p),
					Err(e) if e.downcast_ref() == Some(&PlayerError::NotAPlayer) => (),
					Err(e) => return Err(e),
				},
				Err(e) => {
					return Err(anyhow!(
						"Failed to read line in '{}': {}.",
						roster_file.path.display(),
						e.to_string()
					))
				}
			}
		}

		Ok(Self::from_players(active_players, reserve_players))
	}

	pub fn from_players(active: Vec<Player>, reserve: Vec<Player>) -> Self {
		Roster { active, reserve }
	}

	pub(crate) fn player_count(&self) -> usize {
		self.active.len() + self.reserve.len()
	}

	pub(crate) fn players(
		&self,
	) -> std::iter::Chain<std::slice::Iter<'_, Player>, std::slice::Iter<'_, Player>> {
		(&self.active).iter().chain(&self.reserve)
	}

	pub(crate) fn sort<'a>(&mut self) {
		self.active
			.sort_by(|a, b| a.position.cmp(&b.position).then(a.name.cmp(&b.name)));
		self.reserve
			.sort_by(|a, b| a.position.cmp(&b.position).then(a.name.cmp(&b.name)));
	}

	pub(crate) fn to_msrf_string(team: &str, roster: &Roster) -> String {
		let mut roster_active: Vec<(String, u8)> = roster
			.active
			.iter()
			.map(|p| {
				let captain = if let Some(true) = p.captain {
					true
				} else {
					false
				};
				let ps =
					PlayerState::from(captain, p.id, p.medal, p.name.clone(), p.position, true);
				(PlayerState::to_msrf_string(&ps), p.id)
			})
			.collect();

		let mut roster_reserve: Vec<(String, u8)> = roster
			.reserve
			.iter()
			.map(|p| {
				let captain = if let Some(true) = p.captain {
					true
				} else {
					false
				};
				let ps =
					PlayerState::from(captain, p.id, p.medal, p.name.clone(), p.position, false);
				(PlayerState::to_msrf_string(&ps), p.id)
			})
			.collect();

		roster_active.append(&mut roster_reserve);
		roster_active.sort_by(|(_, a), (_, b)| a.cmp(b));
		let msrf_strings: Vec<String> = roster_active.into_iter().map(|(s, _)| s).collect();
		let msrf_header = format!("---{team}---\n\nCURRENT LINE-UP:\n\n");

		msrf_header + &msrf_strings.join("\n")
	}
}

#[derive(Error, Debug, PartialEq)]
pub enum RosterFileError {
	#[error("Not a roster file.")]
	NotARosterFile,
	#[error("Roster file is missing a header.")]
	MissingHeader,
	#[error("File extension '{0:?}' couldn't be converted")]
	InvalidExtension(OsString),
}

pub struct RosterFile {
	pub path: PathBuf,
	pub team: String,
}

impl RosterFile {
	pub fn get_rosterfile(path: PathBuf) -> Result<RosterFile> {
		let file_extension = Path::new(&path)
			.extension()
			.ok_or_else(|| RosterFileError::NotARosterFile)?
			.to_ascii_lowercase()
			.into_string()
			.map_err(|e| RosterFileError::InvalidExtension(e))?;

		if !path.is_file() || file_extension != "msrf" {
			return Err(RosterFileError::NotARosterFile.into());
		}

		let file = File::open(&path)?;
		let mut reader = BufReader::new(file);
		let mut file_header = String::new();
		reader.read_line(&mut file_header)?;
		let file_header = file_header.trim();
		let team = file_header.replace('-', "").trim().to_string();

		if !file_header.starts_with("---") || !file_header.ends_with("---") || team.is_empty() {
			return Err(RosterFileError::MissingHeader.into());
		}

		Ok(RosterFile { path, team })
	}

	pub(crate) fn get_rosterfiles(folder: &PathBuf) -> Result<Vec<RosterFile>> {
		let mut rosterfiles = Vec::new();
		let entries = fs::read_dir(folder)?;

		for entry in entries {
			match entry {
				Ok(entry) => match Self::get_rosterfile(entry.path()) {
					Ok(r) => rosterfiles.push(r),
					Err(e) if e.downcast_ref() == Some(&RosterFileError::NotARosterFile) => (),
					Err(e) => return Err(e),
				},
				Err(e) => return Err(e.into()),
			}
		}

		Ok(rosterfiles)
	}
}
