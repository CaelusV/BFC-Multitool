use std::{fs, path::{Path, PathBuf}};

use image::io::Reader as ImageReader;

use crate::{
	roster::{Roster, RosterFile},
	slugify,
};

pub enum FormatType {
	TOML,
	MSRF,
}

pub fn create_team_and_portraits(folder: &PathBuf, output_folder: &PathBuf) {
	let rosterfiles = RosterFile::get_rosterfiles(folder);
	if rosterfiles.is_empty() {
		eprintln!("Error: No roster files found.");
		return;
	}

	for roster_file in rosterfiles {
		let roster = Roster::from(&roster_file);

		convert_portraits(&roster_file.team, &roster, folder, &output_folder);
		create_team_file(&roster_file.team, roster, &output_folder, FormatType::TOML);
	}
}

pub fn create_team_file(team: &str, mut roster: Roster, output_folder: &Path, format_type: FormatType) {
	if roster.player_count() < 23 {
		eprintln!(
			"ATTENTION: Creating '{}' team file with fewer than 23 players.",
			team
		)
	}

	let file = match format_type {
		FormatType::TOML => {
			roster.sort();
			toml::to_string(&roster).unwrap()
		}
		FormatType::MSRF => Roster::to_msrf_string(team, &roster),
	};

	if !output_folder.is_dir() {
		if let Err(e) = fs::create_dir(&output_folder) {
			eprintln!("Error: Failed to create output folder: {e}");
			return;
		}
	}
	let output_file = output_folder.join(slugify(team) + ".toml");

	fs::write(output_file, file).unwrap();
}

fn convert_portraits(team: &str, roster: &Roster, folder: &Path, output_folder: &Path) {
	let dds_relative_name = format!("{}_dds", slugify(team));
	let dds_folder = folder.join(&dds_relative_name);
	if !dds_folder.is_dir() {
		eprintln!(
			"Error: Can't rename portraits because '{}' doesn't exist.",
			dds_folder.to_string_lossy()
		);
		return;
	}

	if !output_folder.is_dir() {
		if let Err(e) = fs::create_dir_all(&output_folder) {
			eprintln!("Error: Failed to create portrait folder: {e}");
			return;
		}
	}

	for player in roster.players() {
		// Convert .dds (e.g. "player_XXX03.dds") to .png (e.g. "example-name.png").
		// Converted portraits are placed in a separate folder.
		let default_name = format!("player_XXX{:02}", player.id);
		let dds_path = folder.join(&dds_folder).join(format!("{default_name}.dds"));

		if !dds_path.is_file() {
			eprintln!(
				"Error: Can't rename '{}' because the file doesn't exist.",
				dds_path.to_string_lossy()
			);
			continue;
		}

		// Convert portraits.
		let team_output_folder = output_folder.join(slugify(team));
		let png_path = if let Some(s) = &player.portrait_name {
			team_output_folder.join(format!("{}.png", s))
		} else {
			team_output_folder.join(default_name + ".png")
		};

		if !team_output_folder.is_dir() {
			if let Err(e) = fs::create_dir(team_output_folder) {
				eprintln!("Error: Failed to create output folder for {team}: {e}")
			}
		}

		let img = ImageReader::open(dds_path).unwrap().decode().unwrap();
		img.save(png_path).unwrap();
	}
}
