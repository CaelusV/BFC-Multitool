use anyhow::Result;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{team::TeamName, tournament::GroupID};

#[derive(Deserialize, Serialize, Clone)]
pub struct Fixture {
	pub team1: TeamName,
	pub team2: TeamName,
	pub score1: u8,
	pub score2: u8,
	pub pen1: Option<u8>,
	pub pen2: Option<u8>,
	#[serde(rename = "group_id")]
	pub group: Option<GroupID>,
}

impl Fixture {
	pub fn loser(&self) -> Result<Option<TeamName>> {
		match self.winner() {
			Ok(Some(t)) if t == self.team1 => Ok(Some(self.team2)),
			Ok(Some(t)) if t == self.team2 => Ok(Some(self.team1)),
			r => r, // Draw or error.
		}
	}

	pub fn winner(&self) -> Result<Option<TeamName>> {
		match (self.pen1, self.pen2) {
			(None, Some(pen_goals)) => {
				return Err(FixtureError::MissingPenalties1(
					self.team1.to_string(),
					self.team2.to_string(),
					pen_goals.clone(),
				)
				.into())
			}
			(Some(pen_goals), None) => {
				return Err(FixtureError::MissingPenalties2(
					self.team1.to_string(),
					self.team2.to_string(),
					pen_goals.clone(),
				)
				.into())
			}
			(Some(pen_goals1), Some(pen_goals2)) if pen_goals1 == pen_goals2 => {
				return Err(FixtureError::InvalidPenalties(
					self.team1.to_string(),
					self.team2.to_string(),
				)
				.into())
			}
			_ => (),
		}

		let winner = if self.pen1.is_none() {
			if self.score1 > self.score2 {
				Some(self.team1)
			} else if self.score2 > self.score1 {
				Some(self.team2)
			} else {
				None
			}
		} else {
			match self.pen1 > self.pen2 {
				true => Some(self.team1),
				false => Some(self.team2),
			}
		};

		Ok(winner)
	}
}

#[derive(Error, Debug)]
pub enum FixtureError {
	#[error("{0} vs {1}: Couldn't determine a winner, because pen1 and pen2 are equal.")]
	InvalidPenalties(String, String),
	#[error("{0} vs {1}: Expected pen1, found pen2 = {2}.")]
	MissingPenalties1(String, String, u8),
	#[error("{0} vs {1}: Expected pen2, found pen1 = {2}.")]
	MissingPenalties2(String, String, u8),
}

#[derive(Serialize, Clone)]
pub struct GreatestFixture {
	pub fixture: Fixture,
	pub tournament_name: String,
}

impl GreatestFixture {
	pub fn from(fixture: &Fixture, tournament_name: &str) -> Self {
		Self {
			fixture: fixture.clone(),
			tournament_name: tournament_name.to_string(),
		}
	}
}
