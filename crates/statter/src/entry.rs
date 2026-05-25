use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;

use iced::task::{sipper, Straw};
use tokio::fs;

use crate::rankings::Seasons;
use crate::team::Team;
use crate::tournament::{Participation, Tournament, TournamentResult};
use common::{
	errors::{EntryError, ToolError},
	Progress, TeamName,
};

pub fn run_tournaments(
	source: PathBuf,
	destination: PathBuf,
) -> impl Straw<(), Progress, ToolError> {
	sipper(async move |mut progress| {
		let _ = progress.send(Progress { percent: 0.0 });
		let cup_paths = get_cup_paths(&source).await?;
		if cup_paths.is_empty() {
			return Err(EntryError::MissingTournamentFiles.into());
		}

		// Run all tournaments.
		let mut teams_total_stats: HashMap<TeamName, Team> = HashMap::new();
		let mut all_tournament_results: Vec<TournamentResult> = Vec::new();

		let mut percent_done = 1.0; // Getting the paths count as 1%, I guess.
		let fraction_per_cup = 30.0 / cup_paths.len() as f32; // And generating stats is 30%, I guess.
		for cup in cup_paths {
			let _ = progress
				.send(Progress {
					percent: percent_done,
				})
				.await;
			percent_done += fraction_per_cup;

			let tournament: Tournament = toml::from_str(&fs::read_to_string(cup).await?)?;
			let mut teams_results = tournament.run()?;

			// Add tournament team stats to teams_total_stats stats.
			for tp in &mut teams_results {
				// Create participation for this tournament.
				let participation = Participation::new(
					tournament.tournament_name.clone(),
					tp.placement.ok_or(EntryError::MissingTeamPlacement(
						tournament.tournament_name.clone(),
						tp.team.name,
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
		let _ = progress
			.send(Progress {
				percent: percent_done,
			})
			.await; // Need to update final cup processed.

		// Generate the stats folder.
		if !destination.is_dir() {
			fs::create_dir(&destination).await?;
		}
		percent_done += 1.0; // Up to 32%.

		// Generate tournament results.
		all_tournament_results.sort_unstable_by_key(|k| k.date);

		for tournament_results in &mut all_tournament_results {
			let _ = progress
				.send(Progress {
					percent: percent_done,
				})
				.await;
			percent_done += fraction_per_cup; // Another 30% here, up to 62%.

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
			let tournament_results_path = destination.join(format!(
				"{}-results.toml",
				tournament_results
					.tournament_name
					.to_lowercase()
					.replace(' ', "-")
					.replace(|c: char| !c.is_ascii() || c == ':', "")
			));

			fs::write(tournament_results_path, tournament_results_toml).await?;
		}

		// Generate SeasonRankings. NOTE: TournamentResults are already sorted by date.
		let seasons = Seasons::from(all_tournament_results);
		let rankings_toml = toml::to_string(&seasons)?;
		let rankings_path = destination.join("rankings.toml");
		fs::write(rankings_path, rankings_toml).await?;

		percent_done += 2.0; // Up to 64%.

		// Generate team stats.
		let teams = teams_total_stats.values_mut();
		let fraction_per_team = 36.0 / teams.len() as f32;
		for team in teams {
			let _ = progress
				.send(Progress {
					percent: percent_done,
				})
				.await;
			percent_done += fraction_per_team;

			team.participations
				.as_mut()
				.ok_or(EntryError::MissingTeamParticipation(team.name))?
				.sort_unstable_by_key(|p| p.date);
			let team_toml = toml::to_string(&team)?;
			let team_path = destination.join(team.filename());
			fs::write(team_path, team_toml).await?;
		}
		let _ = progress.send(Progress { percent: 100.0 }).await;
		Ok(())
	})
}

async fn get_cup_paths(source: &PathBuf) -> Result<Vec<PathBuf>, ToolError> {
	let mut cup_file_paths = Vec::new();
	let mut entries = fs::read_dir(source).await?;

	let extension = Some(OsStr::new("toml"));

	loop {
		match entries.next_entry().await {
			Ok(None) => break,
			Ok(Some(entry)) => {
				if entry.path().is_file()
					&& entry.file_name().to_string_lossy().contains("bigfunnycup")
					&& entry.path().extension() == extension
				{
					cup_file_paths.push(entry.path());
				}
			}
			Err(_) => {
				return Err(
					EntryError::SourcePathReadError(source.to_string_lossy().to_string()).into(),
				)
			}
		}
	}

	Ok(cup_file_paths)
}
