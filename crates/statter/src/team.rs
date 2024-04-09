use std::fmt;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::{fixture::GreatestFixture, tournament::Participation};

#[derive(Clone, Serialize)]
pub struct MatchupHistory {
	pub opponent_name: TeamName,
	pub goals_against: u32,
	pub goals_for: u32,
	pub penalties_played: u32, // Penalty shoot-out.
	pub penalties_goals_against: u32,
	pub penalties_goals_for: u32,
	pub wins: u32,
	draws: u32,
	pub losses: u32,
}

impl MatchupHistory {
	pub fn from(
		opponent_name: TeamName,
		goals_against: u32,
		goals_for: u32,
		penalties_played: u32,
		penalties_goals_against: u32,
		penalties_goals_for: u32,
		wins: u32,
		draws: u32,
		losses: u32,
	) -> Self {
		MatchupHistory {
			opponent_name,
			goals_against,
			goals_for,
			penalties_played,
			penalties_goals_against,
			penalties_goals_for,
			wins,
			draws,
			losses,
		}
	}

	pub fn add(&mut self, other: &Self) -> Result<()> {
		if self.opponent_name != other.opponent_name {
			return Err(anyhow!(
				"Can't add 'MatchupHistory's with different names together"
			));
		}

		self.goals_against += other.goals_against;
		self.goals_for += other.goals_for;
		self.penalties_played += other.penalties_played;
		self.penalties_goals_against += other.penalties_goals_against;
		self.penalties_goals_for += other.penalties_goals_for;
		self.wins += other.wins;
		self.draws += other.draws;
		self.losses += other.losses;
		Ok(())
	}
}

#[derive(Clone, Serialize)]
pub struct Team {
	pub name: TeamName,
	pub goals_against: u32,
	pub goals_for: u32,
	pub penalties_played: u32, // Penalty shoot-out.
	pub penalties_goals_against: u32,
	pub penalties_goals_for: u32,
	pub wins: u32,
	pub draws: u32,
	pub losses: u32,
	// We don't want these to show up in tournament file.
	greatest_win: Option<GreatestFixture>,
	greatest_loss: Option<GreatestFixture>,
	pub matchups: Option<Vec<MatchupHistory>>,
	pub participations: Option<Vec<Participation>>,
	pub head_to_head: Option<u8>,
}

impl Team {
	pub fn from(name: TeamName) -> Team {
		Team {
			name,
			goals_against: 0,
			goals_for: 0,
			penalties_played: 0,
			penalties_goals_against: 0,
			penalties_goals_for: 0,
			wins: 0,
			draws: 0,
			losses: 0,
			greatest_win: None,
			greatest_loss: None,
			matchups: None,
			participations: None,
			head_to_head: None,
		}
	}

	pub fn add(&mut self, other: &mut Self) -> Result<()> {
		assert_eq!(self.name, other.name);
		self.goals_against += other.goals_against;
		self.goals_for += other.goals_for;
		self.penalties_played += other.penalties_played;
		self.penalties_goals_against += other.penalties_goals_against;
		self.penalties_goals_for += other.penalties_goals_for;
		self.wins += other.wins;
		self.draws += other.draws;
		self.losses += other.losses;
		if let Some(other_greatest_loss) = other.greatest_loss.as_ref() {
			self.try_add_greatest_loss(other_greatest_loss)?;
		}
		if let Some(other_greatest_win) = other.greatest_win.as_ref() {
			self.try_add_greatest_win(other_greatest_win)?;
		}

		if self.matchups.is_none() {
			self.matchups = other.matchups.clone();
		} else {
			if let (Some(matchups_self), Some(matchups_other)) =
				(self.matchups.as_mut(), other.matchups.as_mut())
			{
				// FIXME: Create helper method to add/insert matchup.
				for matchup in matchups_other {
					if let Some(matchup_self) = matchups_self
						.iter_mut()
						.find(|m| m.opponent_name == matchup.opponent_name)
					{
						matchup_self.add(&matchup)?;
					} else {
						matchups_self.push(matchup.clone());
					}
				}
			}
		}
		if let Some(other_participations) = &mut other.participations {
			let participations = self.participations.get_or_insert(Vec::new());
			participations.append(other_participations);
		}
		Ok(())
	}

	pub fn filename(&self) -> String {
		self.name.to_string().to_lowercase().replace(' ', "-") + ".toml"
	}

	pub fn get_greatest_loss(&self) -> Option<&GreatestFixture> {
		self.greatest_loss.as_ref()
	}

	pub fn get_greatest_win(&self) -> Option<&GreatestFixture> {
		self.greatest_win.as_ref()
	}

	pub fn reset_greatest(&mut self) {
		self.greatest_win = None;
		self.greatest_loss = None;
	}

	fn try_add_greatest(&mut self, g_fixture: &GreatestFixture, win: bool) -> Result<bool> {
		// Make sure the team has played this fixture.
		if g_fixture.fixture.team1 != self.name && g_fixture.fixture.team2 != self.name {
			return Ok(false);
		}

		if let Some(winner) = g_fixture.fixture.winner()? {
			if (win && winner != self.name) || (!win && winner == self.name) {
				return Ok(false);
			}
		}

		let greatest = if win {
			match self.greatest_win.as_ref() {
				Some(gw) => gw,
				None => {
					self.greatest_win = Some(g_fixture.clone());
					return Ok(true);
				}
			}
		} else {
			match self.greatest_loss.as_ref() {
				Some(gl) => gl,
				None => {
					self.greatest_loss = Some(g_fixture.clone());
					return Ok(true);
				}
			}
		};

		let greatest_score_diff = greatest.fixture.score1.abs_diff(greatest.fixture.score2);
		let fixture_score_diff = g_fixture.fixture.score1.abs_diff(g_fixture.fixture.score2);
		let greatest_total_goals = greatest.fixture.score1 + greatest.fixture.score2;
		let fixture_total_goals = g_fixture.fixture.score1 + g_fixture.fixture.score2;

		let mut is_greatest = false;
		if fixture_score_diff > greatest_score_diff
			|| fixture_score_diff == greatest_score_diff
				&& fixture_total_goals > greatest_total_goals
		{
			is_greatest = true;
		} else if let (
			Some(greatest_pen1),
			Some(greatest_pen2),
			Some(fixture_pen1),
			Some(fixture_pen2),
		) = (
			greatest.fixture.pen1,
			greatest.fixture.pen2,
			g_fixture.fixture.pen1,
			g_fixture.fixture.pen2,
		) {
			let greatest_pen_diff = greatest_pen1.abs_diff(greatest_pen2);
			let fixture_pen_diff = fixture_pen1.abs_diff(fixture_pen2);
			let greatest_pen_total = greatest_pen1 + greatest_pen2;
			let fixture_pen_total = fixture_pen1 + fixture_pen2;

			if fixture_pen_diff > greatest_pen_diff
				|| fixture_pen_diff == greatest_pen_diff && fixture_pen_total > greatest_pen_total
			{
				is_greatest = true;
			}
		}

		if is_greatest {
			match win {
				true => self.greatest_win = Some(g_fixture.clone()),
				false => self.greatest_loss = Some(g_fixture.clone()),
			}
		}

		Ok(is_greatest)
	}

	pub fn try_add_greatest_loss(&mut self, other: &GreatestFixture) -> Result<bool> {
		self.try_add_greatest(other, false)
	}

	pub fn try_add_greatest_win(&mut self, other: &GreatestFixture) -> Result<bool> {
		self.try_add_greatest(other, true)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum TeamName {
	Unknown,
	#[serde(rename = "Alpha Space Bros")]
	AlphaSpaceBros,
	Autoism,
	#[serde(rename = "Big Funky")]
	BigFunky,
	#[serde(rename = "Bone Zone")]
	BoneZone,
	#[serde(rename = "Cartoons FC")]
	CartoonsFC,
	Disney,
	Gambit,
	#[serde(rename = "HmX Gaming")]
	HmXGaming,
	Moai,
	Nintendont,
	#[serde(rename = "The Dump")]
	TheDump,
	Vidya,
}

impl fmt::Display for TeamName {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let s = match self {
			TeamName::Unknown => "Unknown",
			TeamName::AlphaSpaceBros => "Alpha Space Bros",
			TeamName::Autoism => "Autoism",
			TeamName::BigFunky => "Big Funky",
			TeamName::BoneZone => "Bone Zone",
			TeamName::CartoonsFC => "Cartoons FC",
			TeamName::Disney => "Disney",
			TeamName::Gambit => "Gambit",
			TeamName::HmXGaming => "HmX Gaming",
			TeamName::Moai => "Moai",
			TeamName::Nintendont => "Nintendont",
			TeamName::TheDump => "The Dump",
			TeamName::Vidya => "Vidya",
		};
		write!(f, "{s}")
	}
}

#[derive(Clone, Serialize)]
pub struct TeamPlacement {
	pub team: Team,
	pub placement: Option<u8>,
}

impl TeamPlacement {
	pub fn from(placement: Option<u8>, team: Team) -> Self {
		TeamPlacement { placement, team }
	}
}
