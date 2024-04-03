use std::{fs, path::Path};

use image::io::Reader as ImageReader;

use crate::{
	roster::{Roster, RosterFile},
	slugify,
};

pub enum FormatType {
	TOML,
	MSRF,
}

pub fn create_team_and_portraits() {
	let rosterfiles = RosterFile::get_rosterfiles();
	if rosterfiles.is_empty() {
		eprintln!("Error: No roster files found.");
		return;
	}

	for roster_file in rosterfiles {
		let roster = Roster::from(&roster_file);

		let output = Path::new("output").join(slugify(&roster_file.team) + ".toml");
		convert_portraits(&roster_file, &roster, &output);
		create_team_file(&roster_file.team, roster, &output, FormatType::TOML);
	}
}

pub fn create_team_file(team: &str, mut roster: Roster, output: &Path, format_type: FormatType) {
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

	let parent = output.parent().unwrap();
	if !parent.is_dir() {
		if let Err(e) = fs::create_dir(&parent) {
			eprintln!("Error: Failed to create output folder: {e}");
			return;
		}
	}

	fs::write(output, file).unwrap();
}

fn convert_portraits(roster_file: &RosterFile, roster: &Roster, output: &Path) {
	let dir_string = format!("{}_dds", slugify(&roster_file.team));
	let dir = Path::new(&dir_string);
	if !dir.is_dir() {
		eprintln!(
			"Error: Can't rename portraits because '{}' doesn't exist.",
			dir_string
		);
		return;
	}

	let parent = output.parent().unwrap();
	if !parent.is_dir() {
		if let Err(e) = fs::create_dir_all(&parent) {
			eprintln!("Error: Failed to create portrait folder: {e}");
			return;
		}
	}

	for player in roster.players() {
		// Convert .dds (e.g. "player_XXX03.dds") to .png (e.g. "example-name.png").
		// Converted portraits are placed in a separate folder.
		let default_name = format!("player_XXX{:02}", player.id);
		let dds_path = dir.join(format!("{default_name}.dds"));

		if !dds_path.is_file() {
			eprintln!(
				"Error: Can't rename '{}' because the file doesn't exist.",
				dir_string
			);
			continue;
		}

		// Convert portraits.
		let png_path = if let Some(s) = &player.portrait_name {
			parent.join(format!("{}.png", s))
		} else {
			parent.join(default_name + ".png")
		};

		let img = ImageReader::open(dds_path).unwrap().decode().unwrap();
		img.save(png_path).unwrap();
	}
}
