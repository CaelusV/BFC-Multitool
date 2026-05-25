use std::{
	ffi::OsStr,
	path::{Path, PathBuf},
};

use anyhow::{Error, Result};
use iced::task::{Straw, sipper};
use image::ImageReader;
use thiserror::Error;
use tokio::fs;

use shared::Progress;

use crate::{
	roster::{Roster, RosterFile},
	slugify,
};

pub enum FormatType {
	TOML,
	MSRF,
}

impl FormatType {
	pub fn from_extension(extension: Option<&OsStr>) -> Option<FormatType> {
		match extension?.to_str()?.to_lowercase().as_str() {
			"toml" => Some(FormatType::TOML),
			"msrf" => Some(FormatType::MSRF),
			_ => None,
		}
	}
}

#[derive(Error, Debug)]
pub(crate) enum CreationError {
	#[error("No roster files round")]
	NoRosterFiles,
	#[error("{0}")]
	CouldNotCreateFolder(String),
}

pub async fn create_team_and_portraits(source: PathBuf, destination: PathBuf) -> impl Straw<(), Progress, Error> {
    sipper(async move |mut progress| {
        let _ = progress.send(Progress { percent: 0.0 }).await;

        let rosterfiles = RosterFile::get_rosterfiles(&source).await?;
    	if rosterfiles.is_empty() {
    		return Err(CreationError::NoRosterFiles.into());
    	}

        // I guess getting the files count as 5%.
        let fraction_per_file = 95.0 / rosterfiles.len() as f32;
    	for (index, roster_file) in rosterfiles.iter().enumerate() {
            let _ = progress.send(Progress { percent: 5.0 + fraction_per_file * index as f32 }).await;

    		let roster = Roster::from_rosterfile(&roster_file).await?;
    		convert_portraits(&roster_file.team, &roster, &source, &destination).await?;
    		create_team_file(&roster_file.team, roster, &destination, FormatType::TOML).await?;
    	}
        let _ = progress.send(Progress { percent: 100.0 }).await;
    	Ok(())
    })
}

pub async fn create_team_file(
	team: &str,
	mut roster: Roster,
	destination_folder: &Path,
	format_type: FormatType,
) -> Result<()> {
	if roster.player_count() < 23 {
		eprintln!(
			"ATTENTION: Creating '{}' team file with fewer than 23 players.",
			team
		)
	}

	let (file, extension) = match format_type {
		FormatType::TOML => {
			roster.sort();
			(toml::to_string(&roster)?, ".toml")
		}
		FormatType::MSRF => {
			(Roster::to_msrf_string(team, &roster), ".msrf")
		}
	};

	if !destination_folder.is_dir() {
		if let Err(e) = fs::create_dir_all(destination_folder).await {
			return Err(CreationError::CouldNotCreateFolder(format!("Failed to create destination folder: {e}")).into());
		}
	}

	fs::write(destination_folder.join(slugify(team) + extension), file).await?;
	Ok(())
}

async fn convert_portraits(team: &str, roster: &Roster, source: &Path, destination: &Path) -> Result<()> {
	let dds_relative_name = format!("{}_dds", slugify(team));
	let dds_folder = source.join(&dds_relative_name);
	if !dds_folder.is_dir() {
		return Err(CreationError::CouldNotCreateFolder(
			format!("Can't rename portraits because the dds folder '{}' doesn't exist.",
			dds_folder.to_string_lossy())
		).into());
	}

	if !destination.is_dir() {
		if let Err(e) = fs::create_dir_all(destination).await {
			return Err(CreationError::CouldNotCreateFolder(format!("Failed to create portrait folder: {e}")).into());
		}
	}

	for player in roster.players() {
		// Convert .dds (e.g. "player_XXX03.dds") to .png (e.g. "example-name.png").
		// Converted portraits are placed in a separate folder.
		let default_name = format!("player_XXX{:02}", player.id);
		let dds_path = source.join(&dds_folder).join(format!("{default_name}.dds"));

		if !dds_path.is_file() {
			eprintln!(
				"WARNING: Couldn't convert '{}' to png because the file doesn't exist.",
				dds_path.to_string_lossy()
			);
			continue;
		}

		// Convert portraits.
		let team_destination_folder = destination.join(slugify(team));
		let png_path = if let Some(s) = &player.portrait_name {
			team_destination_folder.join(format!("{}.png", s))
		} else {
			team_destination_folder.join(default_name + ".png")
		};

		if !team_destination_folder.is_dir() {
			if let Err(e) = fs::create_dir(team_destination_folder).await {
				return Err(CreationError::CouldNotCreateFolder(
					format!("Failed to create destination folder for {team}: {e}")
				).into());
			}
		}

		let img = ImageReader::open(dds_path)?.decode()?;
		img.save(png_path)?;
	}
	Ok(())
}
