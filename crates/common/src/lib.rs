use serde::{Deserialize, Serialize};
use strum_macros;

#[derive(Debug, Clone)]
pub struct Progress {
	pub percent: f32,
}

#[derive(
	Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, strum_macros::Display,
)]
pub enum TeamName {
	Unknown,
	#[serde(rename = "Alpha Space Bros")]
	#[strum(to_string = "Alpha Space Bros")]
	AlphaSpaceBros,
	Autoism,
	#[serde(rename = "Big Funky")]
	#[strum(to_string = "Big Funky")]
	BigFunky,
	#[serde(rename = "Bone Zone")]
	#[strum(to_string = "Bone Zone")]
	BoneZone,
	#[serde(rename = "Cartoons FC")]
	#[strum(to_string = "Cartoons FC")]
	CartoonsFC,
	Cursed,
	Disney,
	#[serde(rename = "FC Fine Dining")]
	#[strum(to_string = "FC Fine Dining")]
	FCFineDining,
	#[serde(rename = "FC PC")]
	#[strum(to_string = "FC PC")]
	FCPC,
	#[serde(rename = "Fink Ployd")]
	#[strum(to_string = "Fink Ployd")]
	FinkPloyd,
	Gambit,
	#[serde(rename = "HmX Gaming")]
	#[strum(to_string = "HmX Gaming")]
	HmXGaming,
	Legoland,
	Moai,
	Nintendont,
	#[serde(rename = "The Chairs")]
	#[strum(to_string = "The Chairs")]
	TheChairs,
	#[serde(rename = "The Dump")]
	#[strum(to_string = "The Dump")]
	TheDump,
	Vidya,
}

pub mod errors {
	use image::ImageError;
	use std::{ffi::OsString, io, sync::Arc};
	use thiserror::Error;

	use crate::TeamName;

	#[derive(Error, Debug, Clone)]
	pub enum ToolError {
		#[error("{0}")]
		CreationError(#[from] CreationError),
		#[error("{0}")]
		EntryError(#[from] EntryError),
		#[error("{0}")]
		FixtureError(#[from] FixtureError),
		#[error("{0}")]
		PlayerError(#[from] PlayerError),
		#[error("{0}")]
		RosterFileError(#[from] RosterFileError),
		#[error("{0}")]
		TeamError(#[from] TeamError),
		#[error("{0}")]
		TournamentError(#[from] TournamentError),
		#[error("{0}")]
		TomlSerError(#[from] toml::ser::Error),
		#[error("{0}")]
		TomlDeError(#[from] toml::de::Error),
		#[error("{0}")]
		IoError(#[from] Arc<io::Error>),
		#[error("{0}")]
		ImageError(#[from] Arc<ImageError>),
	}

	impl From<io::Error> for ToolError {
		fn from(value: io::Error) -> Self {
			ToolError::IoError(Arc::new(value))
		}
	}

	impl From<ImageError> for ToolError {
		fn from(value: ImageError) -> Self {
			ToolError::ImageError(Arc::new(value))
		}
	}

	#[derive(Error, Debug, Clone)]
	pub enum CreationError {
		#[error("No roster files round")]
		NoRosterFiles,
		#[error("{0}")]
		CouldNotCreateFolder(String),
	}

	#[derive(Error, Debug, Clone)]
	pub enum EntryError {
		#[error("Failed to retrieve participations for {0}.")]
		MissingTeamParticipation(TeamName),
		#[error("{0}: No placement was found for {1}, even though they participated.")]
		MissingTeamPlacement(String, TeamName),
		#[error("No tournament files have been found.")]
		MissingTournamentFiles,
		#[error("Failed to read file(s) in the source path: {0}.")]
		SourcePathReadError(String),
	}

	#[derive(Error, Debug, Clone)]
	pub enum FixtureError {
		#[error("{0} vs {1}: Couldn't determine a winner, because pen1 and pen2 are equal.")]
		InvalidPenalties(String, String),
		#[error("{0} vs {1}: Expected pen1, found pen2 = {2}.")]
		MissingPenalties1(String, String, u8),
		#[error("{0} vs {1}: Expected pen2, found pen1 = {2}.")]
		MissingPenalties2(String, String, u8),
	}

	#[derive(Error, Debug, Clone, PartialEq)]
	pub enum PlayerError {
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

	#[derive(Error, Debug, Clone, PartialEq)]
	pub enum RosterFileError {
		#[error("Not a roster file.")]
		NotARosterFile,
		#[error("Roster file is missing a header.")]
		MissingHeader,
		#[error("File extension '{0:?}' couldn't be converted")]
		InvalidExtension(OsString),
		#[error("Failed to read line in '{0}': {1}.")]
		ReadLineFailure(String, String),
	}

	#[derive(Error, Debug, Clone)]
	pub enum TeamError {
		#[error("Can't add together 'MatchupHistory's with non-matching team names.")]
		MatchupsNameMismatch,
		#[error("Can't add together stats from non-matching teams.")]
		TeamAdditionMismatch,
	}

	#[derive(Error, Debug, Clone)]
	pub enum TournamentError {
		#[error(
			"{0}: Comparing {1} & {2} group stage performance, but missing at least one team."
		)]
		ComparisonMissingTeam(String, TeamName, TeamName),
		#[error("Missing or incorrect head-to-head: {0} ({1}): Couldn't resolve ordering between {2} and {3}.")]
		HeadToHeadError(String, String, TeamName, TeamName),
		#[error("{0}: Expected {1} losers bracket fixtures, found {2}. NOTICE: There are {3} teams playing.")]
		IncorrectBracketFixtureCount(String, usize, usize, usize),
		#[error("{0}: Expected {1} playoff teams from group stage, found {2}.")]
		IncorrectTeamsFromGroups(String, usize, usize),
		#[error("{0}: Expected {1} playoff teams, found {2}.")]
		IncorrectPlayoffTeamsAmount(String, usize, usize),
		#[error("{0}: {1}")]
		InvalidGrandFinal(String, String),
		#[error("{0} (Groups): {1} vs {2} is missing a group.")]
		MissingGroupID(String, TeamName, TeamName),
		#[error("{0}: Ran group stage, despite no group stage existing.")]
		MissingGroupStage(String),
		#[error("{0}: Found no qualifying teams from group stage when sorting.")]
		MissingQualifiedTeams(String),
		#[error("{0}: Missing wildcard candidate.")]
		MissingWildcard(String),
		#[error("{0} (Playoffs): {1} vs {2} ended in draw.")]
		PlayoffFixtureDraw(String, TeamName, TeamName),
		#[error("{0} ({1}): {2}")]
		UpdateTeamsFailure(String, String, String),
		#[error("This should never fail: SORTING_PREV_FIXTURE_ERROR.")]
		SortingPreviousFixtureError,
	}
}
