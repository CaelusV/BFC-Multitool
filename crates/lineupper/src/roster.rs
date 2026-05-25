use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::{
	fs::{self, File},
	io::{AsyncBufReadExt, BufReader},
};

use crate::player::{Player, PlayerState};
use common::errors::{PlayerError, RosterFileError, ToolError};

#[derive(Serialize, Deserialize)]
pub struct Roster {
	pub active: Vec<Player>,
	pub reserve: Vec<Player>,
}

impl Roster {
	pub async fn from_rosterfile(roster_file: &RosterFile) -> Result<Self, ToolError> {
		let file = File::open(&roster_file.path).await?;
		let reader = BufReader::new(file);
		let mut active_players = Vec::new();
		let mut reserve_players = Vec::new();

		let mut lines = reader.lines();
		loop {
			match lines.next_line().await {
				Ok(None) => break,
				Ok(Some(line)) => match PlayerState::from_string(line) {
					Ok(PlayerState::Active(p)) => active_players.push(p),
					Ok(PlayerState::Reserve(p)) => reserve_players.push(p),
					Err(ToolError::PlayerError(e)) if e == PlayerError::NotAPlayer => (),
					Err(e) => return Err(e),
				},
				Err(e) => {
					return Err(RosterFileError::ReadLineFailure(
						roster_file.path.display().to_string(),
						e.to_string(),
					)
					.into())
				}
			}
		}

		Ok(Self::from_players(active_players, reserve_players))
	}

	pub async fn from_toml(path: PathBuf) -> Result<Self, ToolError> {
		let roster_string = fs::read_to_string(path).await?;
		Ok(toml::from_str(&roster_string)?)
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
		let mut roster: Vec<(String, u8)> = roster
			.active
			.iter()
			.chain(roster.reserve.iter())
			.map(|p| {
				let ps = PlayerState::from(
					p.id,
					p.name.clone(),
					p.position,
					p.medal,
					p.active,
					p.captain.unwrap_or_default(),
					p.portrait_name.clone(),
				);
				(PlayerState::to_msrf_string(&ps), p.id)
			})
			.collect();

		roster.sort_by(|(_, a), (_, b)| a.cmp(b));
		let msrf_strings: Vec<String> = roster.into_iter().map(|(s, _)| s).collect();
		let msrf_header = format!("---{team}---\n\nCURRENT LINE-UP:\n\n");

		msrf_header + &msrf_strings.join("\n")
	}
}

pub struct RosterFile {
	pub path: PathBuf,
	pub team: String,
}

impl RosterFile {
	pub async fn get_rosterfile(path: PathBuf) -> Result<RosterFile, ToolError> {
		let file_extension = Path::new(&path)
			.extension()
			.ok_or_else(|| RosterFileError::NotARosterFile)?
			.to_ascii_lowercase()
			.into_string()
			.map_err(|e| RosterFileError::InvalidExtension(e))?;

		if !path.is_file() || file_extension != "msrf" {
			return Err(RosterFileError::NotARosterFile.into());
		}

		let file = File::open(&path).await?;
		let mut reader = BufReader::new(file);
		let mut file_header = String::new();
		reader.read_line(&mut file_header).await?;
		let file_header = file_header.trim();
		let team = file_header.replace('-', "").trim().to_string();

		if !file_header.starts_with("---") || !file_header.ends_with("---") || team.is_empty() {
			return Err(RosterFileError::MissingHeader.into());
		}

		Ok(RosterFile { path, team })
	}

	pub(crate) async fn get_rosterfiles(folder: &PathBuf) -> Result<Vec<RosterFile>, ToolError> {
		let mut rosterfiles = Vec::new();
		let mut entries = fs::read_dir(folder).await?;

		loop {
			match entries.next_entry().await {
				Ok(None) => break,
				Ok(Some(entry)) => match Self::get_rosterfile(entry.path()).await {
					Ok(r) => rosterfiles.push(r),
					Err(ToolError::RosterFileError(e)) if e == RosterFileError::NotARosterFile => {
						()
					}
					Err(e) => return Err(e),
				},
				Err(e) => return Err(e.into()),
			}
		}

		Ok(rosterfiles)
	}
}
