use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

use crate::rankings::Seasons;
use crate::team::{Team, TeamName};
use crate::tournament::{Participation, Tournament, TournamentResult};

pub fn run_tournaments(folder: &PathBuf, output_folder: &PathBuf) {
    let cup_paths = get_cup_paths(folder);
    if cup_paths.is_empty() {
        eprintln!("Error: No tournament files have been found.");
        return;
    }

    // Run all tournaments.
    let mut teams_total_stats: HashMap<TeamName, Team> = HashMap::new();
    let mut all_tournament_results: Vec<TournamentResult> = Vec::new();

    for cup in cup_paths {
        let tournament: Tournament = toml::from_str(&fs::read_to_string(cup).unwrap()).unwrap();
        let mut teams_results = tournament.run();

        // Add tournament team stats to teams_total_stats stats.
        for tp in &mut teams_results {
            // Create participation for this tournament.
            let participation = Participation::new(
                tournament.tournament_name.clone(),
                tp.placement.unwrap(),
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
                .add(&mut tp.team);
        }

        // Add TournamentResult to seasons.
        all_tournament_results.push(TournamentResult::from(teams_results, tournament));
    }

    // Generate the stats folder.
    if !output_folder.is_dir() {
        fs::create_dir(&output_folder).unwrap();
    }

    // Generate tournament results.
    all_tournament_results.sort_unstable_by_key(|k| k.date);

    for tournament_results in &mut all_tournament_results {
        tournament_results.team_placements.iter_mut().for_each(|tp| {
            tp.team.matchups = None;
            tp.team.participations = None;
            tp.team.head_to_head = None;
            tp.team.reset_greatest();
        });
        let tournament_results_toml = toml::to_string(&tournament_results).unwrap();
        let tournament_results_path = output_folder.join(
            format!("{}-results.toml", tournament_results
            .tournament_name
            .to_lowercase()
            .replace(' ', "-")
            .replace(|c: char| !c.is_ascii() || c == ':', ""))
        );

        fs::write(tournament_results_path, tournament_results_toml).unwrap();
    }

    // Generate SeasonRankings and sort them.
    let mut seasons = Seasons::from(all_tournament_results);
    seasons
        .seasons
        .sort_by(|a, b| a.season_num.cmp(&b.season_num));
    for s in &mut seasons.seasons {
        // Sort Tournaments in Season from first to last.
        s.tournaments.sort_unstable();
    }
    let rankings_toml = toml::to_string(&seasons).unwrap();
    let rankings_path = output_folder.join("rankings.toml");
    fs::write(rankings_path, rankings_toml).unwrap();

    // Generate team stats.
    for team in teams_total_stats.values_mut() {
        team.participations
            .as_mut()
            .unwrap()
            .sort_unstable_by_key(|p| p.date);
        let team_toml = toml::to_string(&team).unwrap();
        let team_path = output_folder.join(team.filename());
        fs::write(team_path, team_toml).unwrap();
    }
}

fn get_cup_paths(folder: &PathBuf) -> Vec<PathBuf> {
    let mut cup_file_paths = Vec::new();
    let entries = fs::read_dir(folder).unwrap_or_else(|_| panic!("Failed to read directory"));

    let extension = Some(OsStr::new("toml"));
    for entry in entries {
        if let Ok(entry) = entry {
            if entry.path().is_file() && entry.file_name().to_string_lossy().contains("bigfunnycup") && entry.path().extension() == extension
            {
                cup_file_paths.push(entry.path());
            }
        } else {
            panic!("Error: Failed to read file.")
        }
    }

    cup_file_paths
}
