use std::{
	ffi::OsStr,
	fs,
	path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use image::ImageReader;

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

pub fn create_team_and_portraits(source: &PathBuf, destination: &PathBuf) -> Result<()> {
	let rosterfiles = RosterFile::get_rosterfiles(source)?;
	if rosterfiles.is_empty() {
		return Err(anyhow!("No roster files found."));
	}

	for roster_file in rosterfiles {
		let roster = Roster::from_rosterfile(&roster_file)?;

		convert_portraits(&roster_file.team, &roster, source, destination)?;
		create_team_file(&roster_file.team, roster, destination, FormatType::TOML)?;
	}
	Ok(())
}

pub fn create_team_file(
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

	let extension;
	let file = match format_type {
		FormatType::TOML => {
			roster.sort();
			extension = ".toml";
			toml::to_string(&roster)?
		}
		FormatType::MSRF => {
			extension = ".msrf";
			Roster::to_msrf_string(team, &roster)
		}
	};

	if !destination_folder.is_dir() {
		if let Err(e) = fs::create_dir_all(destination_folder) {
			return Err(anyhow!("Failed to create destination folder: {e}"));
		}
	}

	fs::write(destination_folder.join(slugify(team) + extension), file)?;
	Ok(())
}

fn convert_portraits(team: &str, roster: &Roster, source: &Path, destination: &Path) -> Result<()> {
	let dds_relative_name = format!("{}_dds", slugify(team));
	let dds_folder = source.join(&dds_relative_name);
	if !dds_folder.is_dir() {
		return Err(anyhow!(
			"Can't rename portraits because the dds folder '{}' doesn't exist.",
			dds_folder.to_string_lossy()
		));
	}

	if !destination.is_dir() {
		if let Err(e) = fs::create_dir_all(destination) {
			return Err(anyhow!("Failed to create portrait folder: {e}"));
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
			if let Err(e) = fs::create_dir(team_destination_folder) {
				return Err(anyhow!(
					"Failed to create destination folder for {team}: {e}"
				));
			}
		}

		let img = ImageReader::open(dds_path)?.decode()?;
		img.save(png_path)?;
	}
	Ok(())
}
