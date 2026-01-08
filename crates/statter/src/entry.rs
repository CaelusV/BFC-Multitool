use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Result};

use crate::rankings::Seasons;
use crate::team::{Team, TeamName};
use crate::tournament::{Participation, Tournament, TournamentResult};

pub fn run_tournaments(folder: &PathBuf, output_folder: &PathBuf) -> Result<()> {
	let cup_paths = get_cup_paths(folder)?;
	if cup_paths.is_empty() {
		return Err(anyhow!("Error: No tournament files have been found."));
	}

	// Run all tournaments.
	let mut teams_total_stats: HashMap<TeamName, Team> = HashMap::new();
	let mut all_tournament_results: Vec<TournamentResult> = Vec::new();

	for cup in cup_paths {
		let tournament: Tournament = toml::from_str(&fs::read_to_string(cup)?)?;
		let mut teams_results = tournament.run()?;

		// Add tournament team stats to teams_total_stats stats.
		for tp in &mut teams_results {
			// Create participation for this tournament.
			let participation = Participation::new(
				tournament.tournament_name.clone(),
				tp.placement.ok_or(anyhow!(
					"No placement was found for {} in {}, even though they participated",
					tp.team.name,
					tournament.tournament_name
				))?,
				tournament.date,
			);
			// Add the tournament to the team.
			if let Some(p) = &mut tp.team.participations {
				p.push(participation);
			} else {
				tp.team.participations = Some(vec![participation]);
			}

			teams_total_stats
				.entry(tp.team.name)
				.or_insert(Team::from(tp.team.name))
				.add(&mut tp.team)?;
		}

		// Add TournamentResult to seasons.
		all_tournament_results.push(TournamentResult::from(teams_results, tournament));
	}

	// Generate the stats folder.
	if !output_folder.is_dir() {
		fs::create_dir(&output_folder)?;
	}

	// Generate tournament results.
	all_tournament_results.sort_unstable_by_key(|k| k.date);

	for tournament_results in &mut all_tournament_results {
		// Don't clutter tournament results with historic team data.
		tournament_results
			.team_placements
			.iter_mut()
			.for_each(|tp| {
				tp.team.matchups = None;
				tp.team.participations = None;
				// tp.head_to_head = None; // This shouldn't be necessary anymore.
				tp.team.reset_greatest();
			});
		let tournament_results_toml = toml::to_string(&tournament_results)?;
		let tournament_results_path = output_folder.join(format!(
			"{}-results.toml",
			tournament_results
				.tournament_name
				.to_lowercase()
				.replace(' ', "-")
				.replace(|c: char| !c.is_ascii() || c == ':', "")
		));

		fs::write(tournament_results_path, tournament_results_toml)?;
	}

	// Generate SeasonRankings. NOTE: TournamentResults are already sorted by date.
	let seasons = Seasons::from(all_tournament_results);
	let rankings_toml = toml::to_string(&seasons)?;
	let rankings_path = output_folder.join("rankings.toml");
	fs::write(rankings_path, rankings_toml)?;

	// Generate team stats.
	for team in teams_total_stats.values_mut() {
		team.participations
			.as_mut()
			.ok_or(anyhow!(
				"Failed to retrieve participations from {}.",
				team.name
			))?
			.sort_unstable_by_key(|p| p.date);
		let team_toml = toml::to_string(&team)?;
		let team_path = output_folder.join(team.filename());
		fs::write(team_path, team_toml)?;
	}

	Ok(())
}

fn get_cup_paths(folder: &PathBuf) -> Result<Vec<PathBuf>> {
	let mut cup_file_paths = Vec::new();
	let entries = fs::read_dir(folder)?;

	let extension = Some(OsStr::new("toml"));
	for entry in entries {
		if let Ok(entry) = entry {
			if entry.path().is_file()
				&& entry.file_name().to_string_lossy().contains("bigfunnycup")
				&& entry.path().extension() == extension
			{
				cup_file_paths.push(entry.path());
			}
		} else {
			return Err(anyhow!(
				"Failed to read file in: {}.",
				folder.to_string_lossy()
			));
		}
	}

	Ok(cup_file_paths)
}
