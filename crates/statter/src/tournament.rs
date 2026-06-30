use core::cmp::min;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};
use toml::value::Datetime;

use crate::fixture::{Fixture, GreatestFixture};
use crate::rankings::RankedTeam;
use crate::team::{MatchupHistory, Team, TeamPlacement};
use common::{
	errors::{ToolError, TournamentError},
	TeamName,
};

#[derive(Deserialize)]
pub struct Brackets {
	pub winners: Vec<Fixture>,
	pub losers: Option<Vec<Fixture>>,
	pub groups: Option<Vec<Fixture>>,
}

#[derive(Deserialize, Serialize, Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum GroupID {
	A,
	B,
	C,
	D,
}

struct GroupTeams {
	teams: Vec<GroupTeam>,
}

impl GroupTeams {
	fn from(teams: Vec<GroupTeam>) -> Self {
		GroupTeams { teams }
	}

	fn sort_teams(&mut self, tournament_name: &str) -> Result<(), ToolError> {
		let mut has_failed_to_order_team = false;
		let mut failed_team1 = TeamName::Unknown;
		let mut failed_team2 = TeamName::Unknown;
		self.teams.sort_unstable_by(|b, a| {
			let order = a.cmp(&b);
			if order == Ordering::Equal {
				has_failed_to_order_team = true;
				failed_team1 = a.team;
				failed_team2 = b.team;
			}
			order
		});

		if has_failed_to_order_team {
			return Err(TournamentError::HeadToHeadError(
				tournament_name.to_string(),
				"Groups".to_string(),
				failed_team1,
				failed_team2,
			)
			.into());
		}
		Ok(())
	}
}

impl Deref for GroupTeams {
	type Target = Vec<GroupTeam>;

	fn deref(&self) -> &Self::Target {
		&self.teams
	}
}

impl DerefMut for GroupTeams {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.teams
	}
}

#[derive(Clone, PartialEq, Eq)]
struct GroupTeam {
	group: GroupID,
	team: TeamName,
	points: u8,
	goals_for: u8,
	goals_against: u8,
	head_to_head: Option<u8>,
}

impl GroupTeam {
	fn from_fixture_result(
		group: GroupID,
		team: TeamName,
		goals_for: u8,
		goals_against: u8,
	) -> Self {
		Self {
			group,
			team,
			points: Self::points_from_fixture_result(goals_for, goals_against),
			goals_for,
			goals_against,
			head_to_head: None,
		}
	}

	fn points_from_fixture_result(goals_for: u8, goals_against: u8) -> u8 {
		if goals_for > goals_against {
			3
		} else if goals_for == goals_against {
			1
		} else {
			0
		}
	}

	fn add_from_fixture_result(&mut self, goals_for: u8, goals_against: u8) {
		self.points += Self::points_from_fixture_result(goals_for, goals_against);
		self.goals_for += goals_for;
		self.goals_against += goals_against;
	}
}

impl Ord for GroupTeam {
	fn cmp(&self, other: &Self) -> Ordering {
		self.points
			.cmp(&other.points)
			.then({
				let goal_diff_self = self.goals_for as i16 - self.goals_against as i16;
				let goal_diff_other = other.goals_for as i16 - other.goals_against as i16;
				goal_diff_self.cmp(&goal_diff_other)
			})
			.then(self.goals_for.cmp(&other.goals_for))
			// Decider (extra) game
			.then_with(|| {
				if let (Some(a), Some(b)) = (self.head_to_head, other.head_to_head) {
					a.cmp(&b)
				} else {
					std::cmp::Ordering::Equal
				}
			})
	}
}

impl PartialOrd for GroupTeam {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

pub struct GroupStage<'a> {
	placements: TournamentPlacements,
	tournament: &'a Tournament,
}

impl<'a> GroupStage<'a> {
	pub fn from(tournament: &'a Tournament) -> Self {
		Self {
			placements: TournamentPlacements::new(),
			tournament,
		}
	}

	fn run(mut self) -> Result<PlayoffStage<'a>, ToolError> {
		let mut groups_seen: HashSet<GroupID> = HashSet::new();
		let mut team_scores: HashMap<TeamName, GroupTeam> = HashMap::new();

		// First check amount of teams in groups and how many are supposed to go to playoffs.
		// This can be done by checking length of hashmap after all group fixtures are done.
		for fixture in
			self.tournament
				.brackets
				.groups
				.as_ref()
				.ok_or(TournamentError::MissingGroupStage(
					self.tournament.tournament_name.clone(),
				))? {
			match self
				.placements
				.update_teams(fixture, true, &self.tournament.tournament_name)
			{
				Ok(v) => v,
				Err(e) => {
					return Err(TournamentError::UpdateTeamsFailure(
						self.tournament.tournament_name.clone(),
						"Groups".to_string(),
						e.to_string(),
					)
					.into())
				}
			};

			let group = fixture.group.ok_or(TournamentError::MissingGroupID(
				self.tournament.tournament_name.clone(),
				fixture.team1,
				fixture.team2,
			))?;
			groups_seen.insert(group);

			team_scores
				.entry(fixture.team1)
				.and_modify(|gt| gt.add_from_fixture_result(fixture.score1, fixture.score2))
				.or_insert(GroupTeam::from_fixture_result(
					group,
					fixture.team1,
					fixture.score1,
					fixture.score2,
				));

			team_scores
				.entry(fixture.team2)
				.and_modify(|gt| gt.add_from_fixture_result(fixture.score2, fixture.score1))
				.or_insert(GroupTeam::from_fixture_result(
					group,
					fixture.team2,
					fixture.score2,
					fixture.score1,
				));
		}

		// Fill in Head To Head between equal teams.
		if let Some(h2h) = &self.tournament.head_to_head {
			for h2h_decider in h2h {
				let team_score = team_scores
					.get_mut(&h2h_decider.team)
					.expect(&format!("{:?}", h2h_decider.team));
				team_score.head_to_head = Some(h2h_decider.decider_points)
			}
		}

		let team_count = team_scores.len();
		let qualifying_teams_per_group = self.tournament.playoff_teams as usize / groups_seen.len();
		let wildcards_count =
			self.tournament.playoff_teams as usize - qualifying_teams_per_group * groups_seen.len();

		// Wildcards are given to teams that don't automatically qualify for the
		// playoffs by being top of their group. There's a maximum of 1 wildcard
		// per group.
		let mut wildcard_candidates = GroupTeams::from(Vec::with_capacity(groups_seen.len()));
		let mut qualifying_teams =
			GroupTeams::from(Vec::with_capacity(self.tournament.playoff_teams as usize));

		// Add all the qualifying and wildcard candidate teams.
		for group in &groups_seen {
			let mut group_teams = GroupTeams::from(
				team_scores
					.values()
					.filter(|gt| &gt.group == group)
					.map(|gt| gt.clone())
					.collect(),
			);

			group_teams.sort_teams(&self.tournament.tournament_name)?;

			let not_qualified = group_teams.split_off(qualifying_teams_per_group);
			wildcard_candidates.push(
				not_qualified
					.first()
					.ok_or(TournamentError::MissingWildcard(
						self.tournament.tournament_name.clone(),
					))?
					.clone(),
			);
			qualifying_teams.append(&mut group_teams);
		}

		// Sort candidates and add the qualifying wildcard candidates to qualifying teams.
		wildcard_candidates.sort_teams(&self.tournament.tournament_name)?;
		wildcard_candidates.drain(wildcards_count..);
		qualifying_teams.append(&mut wildcard_candidates);

		// Give placements to all of the eliminated teams.
		let mut eliminated_teams: GroupTeams = GroupTeams::from(
			team_scores
				.values()
				.filter(|gt| !qualifying_teams.contains(gt))
				.map(|gt| gt.clone())
				.collect(),
		);
		eliminated_teams.sort_teams(&self.tournament.tournament_name)?;
		for (i, gt) in eliminated_teams.iter().rev().enumerate() {
			let placement = team_count - i;
			self.placements.set_placement(gt.team, placement as u8);
		}

		Ok(PlayoffStage::from_groups(self, qualifying_teams))
	}
}

#[derive(Deserialize)]
pub struct HeadToHead {
	pub team: TeamName,
	pub decider_points: u8,
}

#[derive(Clone, Debug, Serialize)]
pub struct Participation {
	tournament_name: String,
	pub date: Datetime,
	placement: u8,
}

impl Participation {
	pub fn new(tournament_name: String, placement: u8, date: Datetime) -> Self {
		Participation {
			tournament_name,
			date,
			placement,
		}
	}
}

struct PlayoffStage<'a> {
	placements: TournamentPlacements,
	tournament: &'a Tournament,
	// Only if groups are used.
	qualifying_teams: Option<GroupTeams>,
}

impl<'a> PlayoffStage<'a> {
	fn from(tournament: &'a Tournament) -> Self {
		Self {
			placements: TournamentPlacements::new(),
			tournament,
			qualifying_teams: None,
		}
	}

	fn from_groups(groups: GroupStage<'a>, qualifying_teams: GroupTeams) -> Self {
		Self {
			placements: groups.placements,
			tournament: groups.tournament,
			qualifying_teams: Some(qualifying_teams),
		}
	}

	fn run(&mut self) -> Result<Vec<TeamPlacement>, ToolError> {
		// If teams are unranked, it means they've made it to playoffs from group stage.
		if self.tournament.brackets.groups.is_some() {
			let unranked_teams_count = self
				.placements
				.values()
				.filter(|&tp| tp.placement.is_none())
				.count();

			if unranked_teams_count != self.tournament.playoff_teams as usize {
				return Err(TournamentError::IncorrectTeamsFromGroups(
					self.tournament.tournament_name.clone(),
					self.tournament.playoff_teams as usize,
					unranked_teams_count,
				)
				.into());
			}
		}

		// Run the brackets.
		self.winners_bracket()?;
		if self.tournament.has_losers {
			self.losers_bracket()?;
			self.grand_final()?;
		}

		// Make sure the correct number of teams actually played.
		if self.tournament.brackets.groups.is_none()
			&& self.placements.len() != self.tournament.playoff_teams as usize
		{
			return Err(TournamentError::IncorrectPlayoffTeamsAmount(
				self.tournament.tournament_name.clone(),
				self.tournament.playoff_teams as usize,
				self.placements.len(),
			)
			.into());
		}

		// Fill in Head To Head between equal teams.
		if let Some(h2h) = &self.tournament.head_to_head {
			for h2h_decider in h2h {
				let tp = self
					.placements
					.get_mut(&h2h_decider.team)
					.expect(&format!("{:?}", h2h_decider.team));
				tp.head_to_head = Some(h2h_decider.decider_points);
			}
		}

		// Order the teams after placement, then goal difference, then goals for.
		// If equal, order based on previous fixture in winners bracket,
		// group stage placement, then decider fixture (extra fixture).
		let mut teams_ordered: Vec<TeamPlacement> = self.placements.clone().into_values().collect();
		let mut sort_error = Ok(());
		teams_ordered.sort_unstable_by(|a, b| {
			// Placement
			a.placement
				.cmp(&b.placement)
				// Goal difference
				.then_with(|| {
					let a_goal_difference = a.team.goals_for as i32 - a.team.goals_against as i32;
					let b_goal_difference = b.team.goals_for as i32 - b.team.goals_against as i32;
					b_goal_difference.cmp(&a_goal_difference)
				})
				// Goals for
				.then(b.team.goals_for.cmp(&a.team.goals_for))
				// Previous fixture in winners bracket.
				.then_with(|| {
					if !self.tournament.has_losers {
						return std::cmp::Ordering::Equal;
					}

					// If the teams have played previously in the winners bracket,
					// then base placement based on the outcome of that fixture.
					let prev_fixtures = self
						.tournament
						.brackets
						.winners
						.iter()
						.filter(|&g| {
							(g.team1 == a.team.name && g.team2 == b.team.name)
								|| (g.team1 == b.team.name && g.team2 == a.team.name)
						})
						.collect::<Vec<_>>();

					match prev_fixtures.first() {
						Some(&g) => match g.winner() {
							Ok(Some(winner)) if winner == a.team.name => std::cmp::Ordering::Less,
							Ok(_) => std::cmp::Ordering::Greater,
							Err(_) => {
								if sort_error.is_ok() {
									sort_error =
										Err(TournamentError::SortingPreviousFixtureError.into());
								}
								std::cmp::Ordering::Equal
							}
						},
						None => std::cmp::Ordering::Equal,
					}
				})
				// Group stage placement
				.then_with(|| {
					if self.tournament.brackets.groups.is_none() {
						return std::cmp::Ordering::Equal;
					}

					if let Some(qualifying_teams) = self.qualifying_teams.as_ref() {
						let a_team = qualifying_teams.iter().find(|gt| gt.team == a.team.name);
						let b_team = qualifying_teams.iter().find(|gt| gt.team == b.team.name);

						if sort_error.is_ok() && (a_team.is_none() || b_team.is_none()) {
							sort_error = Err(TournamentError::ComparisonMissingTeam(
								self.tournament.tournament_name.clone(),
								a.team.name,
								b.team.name,
							)
							.into());
						}

						b_team.cmp(&a_team)
					} else {
						if sort_error.is_ok() {
							sort_error = Err(TournamentError::MissingQualifiedTeams(
								self.tournament.tournament_name.clone(),
							));
						}
						std::cmp::Ordering::Equal
					}
				})
				// Decider (extra) game
				.then_with(|| {
					let a_h2h = a.head_to_head;
					let b_h2h = b.head_to_head;

					if let (Some(a), Some(b)) = (a_h2h, b_h2h) {
						b.cmp(&a)
					} else {
						if sort_error.is_ok() {
							sort_error = Err(TournamentError::HeadToHeadError(
								self.tournament.tournament_name.clone(),
								"Post-Playoff".to_string(),
								a.team.name,
								b.team.name,
							));
						}
						std::cmp::Ordering::Equal
					}
				})
		});
		sort_error?;

		// Give the correct rank numbers from 1..n.
		for (i, tp) in teams_ordered.iter_mut().enumerate() {
			tp.placement = Some(1 + i as u8);
		}

		Ok(teams_ordered)
	}

	fn grand_final(&mut self) -> Result<(), ToolError> {
		let gf_fixtures = self.tournament.grand_final.as_ref().unwrap();

		if gf_fixtures.len() == 0 || gf_fixtures.len() > 2 {
			return Err(TournamentError::InvalidGrandFinal(
				self.tournament.tournament_name.clone(),
				format!(
					"Expected 1 or 2 grand final fixtures, found {}.",
					gf_fixtures.len()
				),
			)
			.into());
		}

		let first_fixture = gf_fixtures.first().unwrap();
		let team1 = self.placements.get(&first_fixture.team1).unwrap();
		let team2 = self.placements.get(&first_fixture.team2).unwrap();

		// NOTE: This could break if logic changes in losers_bracket method.
		// Currently ranks are constantly updated in losers_bracket method every fixture,
		// with 3rd place being the highest attainable (of course).
		// FIXME: Check who was in last losers bracket fixture instead. This is stupid.
		let team_from_losers = match team1.placement.unwrap() {
			3 => team1.team.name,
			_ => team2.team.name,
		};

		if let Some(first_fixture_winner) = first_fixture.winner()? {
			if gf_fixtures.len() == 1 && first_fixture_winner == team_from_losers {
				return Err(TournamentError::InvalidGrandFinal(
				    self.tournament.tournament_name.clone(),
					format!("{first_fixture_winner} won grand final in 1 fixture, despite coming from losers bracket."),

				).into());
			}

			if gf_fixtures.len() == 2 && first_fixture_winner != team_from_losers {
				return Err(TournamentError::InvalidGrandFinal(
				self.tournament.tournament_name.clone(),
					format!("{first_fixture_winner} came from winners bracket and won the grand final in the first fixture, but a second fixture was found."),
				).into());
			}
		}

		for fixture in gf_fixtures {
			match self
				.placements
				.update_teams(fixture, false, &self.tournament.tournament_name)
			{
				Ok(v) => v,
				Err(e) => {
					return Err(TournamentError::InvalidGrandFinal(
						format!("{} (GrandFinal)", self.tournament.tournament_name),
						format!("{e}"),
					)
					.into())
				}
			};

			if let (Some(winner), Some(loser)) = (fixture.winner()?, fixture.loser()?) {
				self.placements.set_placement(loser, 2);
				self.placements.set_placement(winner, 1);
			} else {
				return Err(TournamentError::InvalidGrandFinal(
					self.tournament.tournament_name.clone(),
					"1 or more Grand final fixtures were drawn.".to_string(),
				)
				.into());
			}
		}
		Ok(())
	}

	fn losers_bracket(&mut self) -> Result<(), ToolError> {
		let theoretical_fixtures_played = (self.tournament.playoff_teams - 2) as usize;
		let fixtures = self.tournament.brackets.losers.as_ref().unwrap();
		let actual_fixtures_played = fixtures.len();

		if theoretical_fixtures_played != actual_fixtures_played {
			return Err(TournamentError::IncorrectBracketFixtureCount(
				self.tournament.tournament_name.clone(),
				theoretical_fixtures_played,
				actual_fixtures_played,
				self.tournament.playoff_teams as usize,
			)
			.into());
		}

		// Number of fixtures in a given losers stage, where n is the stage.
		// Reverse from losers final to round 1: n=1 means losers final.
		// stage_fixture_count = 0.5 * 2^ceil(stage / 2) = 2^ceil(stage / 2 - 1)
		let stage_fixture_count =
			|stage: u8| 2usize.pow(f32::ceil(stage as f32 / 2.0 - 1.0) as u32);
		// Almost same thing, but have to accumulate. Since the stage_fixture_count is a power of 2,
		// we can use almost the same formula, although we don't have to subtract one from the power,
		// and we also need to multiply by two, considering we're doing the accumulate of stages/2.
		// This works because every even&odd pair of stages have the same amount of fixtures.
		// If the stages supplied is odd, we have to subtract the even pair for that stage, so we don't overcount.
		let stages_fixtures_accum = |stages: u8| {
			(2usize.pow(f32::ceil(stages as f32 / 2.0) as u32) - 1) * 2
				- stages as usize % 2 * stage_fixture_count(stages)
		};
		// We still have to account for the team going through the winners bracket (not -1).
		let mut teams_left = self.tournament.playoff_teams;
		let mut stages_left = 0;
		// Figure out the stages left and the amount of fixtures in the outer stage.
		while stages_fixtures_accum(stages_left) < actual_fixtures_played {
			stages_left += 1;
		}
		let mut stage_fixtures = actual_fixtures_played - stages_fixtures_accum(stages_left - 1);

		let mut teams_to_subtract = 0;
		for fixture in fixtures {
			match self
				.placements
				.update_teams(fixture, false, &self.tournament.tournament_name)
			{
				Ok(_) => (),
				Err(e) => {
					return Err(TournamentError::UpdateTeamsFailure(
						self.tournament.tournament_name.clone(),
						"Losers".to_string(),
						e.to_string(),
					)
					.into())
				}
			};

			// FIXME: Winner shouldn't need to have placement set now, as they aren't out,
			// but grand_final depends on the winner of the losers bracket to have a placemetn.
			// Fix in grand_final method, then remove here.
			self.placements
				.set_placement(fixture.loser()?.unwrap(), teams_left);
			self.placements
				.set_placement(fixture.winner()?.unwrap(), teams_left);

			stage_fixtures -= 1;
			teams_to_subtract += 1; // We need to use teams_left unchanged for placements...
			if stage_fixtures == 0 {
				stages_left = stages_left - 1;
				stage_fixtures = stage_fixture_count(stages_left);
				teams_left -= teams_to_subtract; // ...so we only subtract when changing stages.
				teams_to_subtract = 0;
			}
		}
		Ok(())
	}

	// NOTE: The following text is incorrect if losers bracket is used, but losers bracket
	// will fix it for us.
	// NOTE: There'll be n-1 winners fixtures, where n is the number of teams, since you'll
	// need 2 teams for 1 fixture, 3 for 2 fixtures, etc.
	// NOTE: ceiling(log2(n)) is the number of stages in winners. Think of winners bracket
	// as a branching tree from finals and backwards: Finals have 2 teams, semis
	// has 4, then 8, 16, etc. If stages had decimals, then teams = 2^stages, and
	// stages = log2(teams). So, without decimals, stages = ceil(log2(n)).
	// NOTE: First stage has n-2^floor(log2(n)) fixtures. Same principle as above, but we use
	// floor instead of ceil. That way providing it as exponent to 2 gives us the
	// first power lower/equal to the number of teams. Subtracting this power from
	// teams will give the number of fixtures left in the stage, except if the number
	// of teams is a power of 2. In that case, divide number of teams by 2.
	fn winners_bracket(&mut self) -> Result<(), ToolError> {
		let theoretical_fixtures_in_bracket = (self.tournament.playoff_teams - 1) as usize;
		let actual_fixtures_in_bracket = self.tournament.brackets.winners.len();

		if theoretical_fixtures_in_bracket != actual_fixtures_in_bracket {
			return Err(TournamentError::IncorrectBracketFixtureCount(
				self.tournament.tournament_name.clone(),
				theoretical_fixtures_in_bracket,
				actual_fixtures_in_bracket,
				self.tournament.playoff_teams as usize,
			)
			.into());
		}

		let mut teams_left = self.tournament.playoff_teams;
		let mut stages_left = f32::ceil(f32::log2(teams_left as f32)) as u8;
		let mut stage_fixture_count = teams_left - 2u8.pow(f32::log2(teams_left as f32) as u32);

		if stage_fixture_count == 0 {
			stage_fixture_count = self.tournament.playoff_teams / 2;
		}

		for fixture in &self.tournament.brackets.winners {
			match self
				.placements
				.update_teams(fixture, false, &self.tournament.tournament_name)
			{
				Ok(_) => (),
				Err(e) => {
					return Err(TournamentError::UpdateTeamsFailure(
						self.tournament.tournament_name.clone(),
						"Winners".to_string(),
						e.to_string(),
					)
					.into())
				}
			};
			self.placements
				.set_placement(fixture.loser()?.unwrap(), teams_left);
			self.placements
				.set_placement(fixture.winner()?.unwrap(), teams_left);

			stage_fixture_count -= 1;

			if stage_fixture_count == 0 {
				stages_left = stages_left - 1;
				teams_left = 2u8.pow(stages_left as u32);
				stage_fixture_count = teams_left / 2;
			}
		}

		// Without a grand final we need to fix the winner of the last winners game.
		if self.tournament.grand_final.is_none() {
			let last_game = self.tournament.brackets.winners.last().unwrap();
			if let (Some(winner), Some(loser)) = (last_game.winner()?, last_game.loser()?) {
				self.placements.set_placement(winner, 1);
				self.placements.set_placement(loser, 2);
			}
		}

		Ok(())
	}
}

#[derive(
	Debug, Clone, Copy, Deserialize, Serialize, strum_macros::Display,
)]
pub enum PointSystem {
    TennisV1,
    LinearV1,
}

struct Points {
    system: PointSystem,
}

impl Points {
    const TENNIS_V1: [u32; 17] = [
   		1500, 1100, 900, 750, 600, 500, 400, 300, 200, 100, 50, 25, 12, 6, 3, 1, 0,
   	];
    const LINEAR_V1: [u32; 17] = [
        10_000, 9000, 8500, 7500, 7000, 6500, 6000, 5500, 5000, 4500, 4000, 3500, 3000, 2000, 1000, 100, 0
   	];

    fn new(point_system: PointSystem) -> Self {
        Points { system: point_system}
    }

    fn get(&self, idx: usize) -> u32 {
        let idx = match self.system {
            PointSystem::TennisV1 => min(Self::TENNIS_V1.len() - 1, idx),
            PointSystem::LinearV1 => min(Self::LINEAR_V1.len() - 1, idx),
        };

        match self.system {
            PointSystem::TennisV1 => Self::TENNIS_V1[idx],
            PointSystem::LinearV1 => Self::LINEAR_V1[idx],
        }
    }
}

#[derive(Deserialize)]
pub struct Tournament {
	pub tournament_name: String,
	pub season_num: u8,
	pub date: Datetime,
	pub has_losers: bool, // Losers bracket.
	pub playoff_teams: u8,
	pub point_system: PointSystem,
	pub brackets: Brackets,
	pub grand_final: Option<Vec<Fixture>>,
	pub head_to_head: Option<Vec<HeadToHead>>,
}

impl Tournament {
	pub fn run(&self) -> Result<Vec<TeamPlacement>, ToolError> {
		let mut playoffs = match self.brackets.groups {
			Some(_) => GroupStage::from(self).run()?,
			None => PlayoffStage::from(self),
		};

		playoffs.run()
	}
}

#[derive(Serialize)]
pub struct TournamentResult {
	pub tournament_name: String,
	pub season_num: u8,
	pub date: Datetime,
	point_system: PointSystem,
	pub team_placements: Vec<TeamPlacement>,
}

impl TournamentResult {
	pub fn from(team_placements: Vec<TeamPlacement>, tourny: Tournament) -> Self {
		Self {
			tournament_name: tourny.tournament_name,
			season_num: tourny.season_num,
			date: tourny.date,
			point_system: tourny.point_system,
			team_placements,
		}
	}

	pub fn get_teams_ranked(&self) -> Vec<RankedTeam> {
	    let points = Points::new(self.point_system);
		self.team_placements
			.iter()
			.map(|tp| {
				let placement = tp.placement.unwrap();
				RankedTeam {
					name: tp.team.name.clone(),
					ranking_points: vec![points.get(placement as usize - 1)],
					ranks: Vec::new(),
				}
			})
			.collect()
	}
}

struct TournamentPlacements {
	placements: HashMap<TeamName, TeamPlacement>,
}

impl TournamentPlacements {
	fn new() -> Self {
		Self {
			placements: HashMap::default(),
		}
	}

	fn set_placement(&mut self, team: TeamName, placement: u8) {
		self.entry(team)
			.and_modify(|tp| tp.placement = Some(placement));
	}

	pub fn update_teams(
		&mut self,
		fixture: &Fixture,
		is_groups: bool,
		tournament_name: &str,
	) -> Result<(), ToolError> {
		self.update_team(fixture, true, is_groups, tournament_name)?;
		self.update_team(fixture, false, is_groups, tournament_name)?;
		Ok(())
	}

	fn update_team(
		&mut self,
		fixture: &Fixture,
		is_team1: bool,
		is_groups: bool,
		tournament_name: &str,
	) -> Result<(), ToolError> {
		let (team_name, opponent_name, goals_for, goals_against, pen_goals_for, pen_goals_against) =
			match is_team1 {
				true => (
					fixture.team1,
					fixture.team2,
					fixture.score1,
					fixture.score2,
					fixture.pen1,
					fixture.pen2,
				),
				false => (
					fixture.team2,
					fixture.team1,
					fixture.score2,
					fixture.score1,
					fixture.pen2,
					fixture.pen1,
				),
			};

		let team_entry = self
			.entry(team_name)
			.or_insert(TeamPlacement::from(None, Team::from(team_name)));

		team_entry.team.goals_for += goals_for as u32;
		team_entry.team.goals_against += goals_against as u32;

		// Add penalties_played, penalties_goals_against, penalties_goals_for.
		let (penalties_played, penalties_goals_against, penalties_goals_for) =
			match (pen_goals_for, pen_goals_against) {
				(Some(pgf), Some(pga)) => {
					team_entry.team.penalties_goals_for += pgf as u32;
					team_entry.team.penalties_goals_against += pga as u32;
					team_entry.team.penalties_played += 1;
					(1, pga, pgf)
				}
				_ => (0, 0, 0),
			};

		// Add wins, draws, losses.
		let winner = fixture.winner()?;
		let (wins, draws, losses) = match winner {
			Some(t) if t == team_name => {
				team_entry.team.wins += 1;
				(1, 0, 0)
			}
			Some(_) => {
				team_entry.team.losses += 1;
				(0, 0, 1)
			}
			None => {
				if !is_groups {
					return Err(TournamentError::PlayoffFixtureDraw(
						tournament_name.to_string(),
						team_name,
						opponent_name,
					)
					.into());
				}
				team_entry.team.draws += 1;
				(0, 1, 0)
			}
		};

		// Add greatest_{win/loss}.
		let maybe_greatest = GreatestFixture::from(&fixture, tournament_name);
		team_entry.team.try_add_greatest_win(&maybe_greatest)?;
		team_entry.team.try_add_greatest_loss(&maybe_greatest)?;

		// Add this matchup to the matchup history.
		let this_matchup = MatchupHistory::from(
			opponent_name,
			goals_against as u32,
			goals_for as u32,
			penalties_played,
			penalties_goals_against as u32,
			penalties_goals_for as u32,
			wins,
			draws,
			losses,
		);

		// FIXME: Create helper method to add/insert matchup.
		match team_entry.team.matchups.as_mut() {
			Some(matchups) => {
				if let Some(matchup) = matchups
					.iter_mut()
					.find(|m| m.opponent_name == opponent_name)
				{
					matchup.add(&this_matchup)?;
				} else {
					matchups.push(this_matchup);
				}
			}
			None => team_entry.team.matchups = Some(vec![this_matchup]),
		}

		Ok(())
	}
}

impl Deref for TournamentPlacements {
	type Target = HashMap<TeamName, TeamPlacement>;

	fn deref(&self) -> &Self::Target {
		&self.placements
	}
}

impl DerefMut for TournamentPlacements {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.placements
	}
}
