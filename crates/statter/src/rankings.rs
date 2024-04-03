use serde::Serialize;
use toml::value::Datetime;

use crate::{team::TeamName, tournament::TournamentResult};

#[derive(Serialize)]
pub struct RankedTeam {
	pub name: TeamName,
	pub ranking_points: Vec<u32>,
	pub ranks: Vec<u8>,
}

#[derive(Serialize)]
pub struct SeasonRankings {
	pub date: Datetime,
	pub season_num: u8,
	rankings: Vec<RankedTeam>,
	pub tournaments: Vec<String>,
}

#[derive(Serialize)]
pub struct Seasons {
	pub seasons: Vec<SeasonRankings>,
}

impl Seasons {
	pub fn from(tourny_results: Vec<TournamentResult>) -> Seasons {
		let mut seasons = Seasons::new();
		for tr in tourny_results {
			seasons.push(tr);
		}

		seasons
	}

	fn new() -> Self {
		Self {
			seasons: Vec::new(),
		}
	}

	fn push(&mut self, tourny_result: TournamentResult) {
		let season_num = tourny_result.season_num;

		// Get correct season for the tournament result.
		let season_rankings = match self.seasons.iter_mut().find(|s| s.season_num == season_num) {
			Some(sr) => sr,
			None => {
				let sr = SeasonRankings {
					date: tourny_result.date,
					season_num,
					rankings: Default::default(),
					tournaments: Default::default(),
				};
				self.seasons.push(sr);
				self.seasons.last_mut().unwrap()
			}
		};

		if !season_rankings
			.tournaments
			.contains(&tourny_result.tournament_name)
		{
			season_rankings
				.tournaments
				.push(tourny_result.tournament_name.clone());
		}

		// Populate ranking points.
		for mut new_r in tourny_result.get_teams_ranked() {
			if let Some(old_r) = season_rankings
				.rankings
				.iter_mut()
				.find(|old_r| old_r.name == new_r.name)
			{
				old_r.ranking_points.push(
					old_r.ranking_points.last().unwrap() + new_r.ranking_points.last().unwrap(),
				);
			} else {
				// Fill out rankings before this tournament.
				let n = season_rankings.tournaments.len() - 1;
				new_r.ranking_points.splice(0..0, vec![0; n]);
				new_r.ranks.splice(0..0, vec![0; n]);
				season_rankings.rankings.push(new_r);
			}
		}

		// Sort Rankings in Season – played so far – from highest to lowest points. Needed to generate ranks.
		season_rankings
			.rankings
			.sort_unstable_by(|a, b| b.ranking_points.last().cmp(&a.ranking_points.last()));

		// Populate ranks.
		for (i, r_team) in season_rankings.rankings.iter_mut().enumerate() {
			r_team.ranks.push(i as u8 + 1);
		}
	}
}
