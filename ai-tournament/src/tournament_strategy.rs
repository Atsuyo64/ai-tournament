//! Tournament strategies used by the evaluator to schedule agent matchups.
//!
//! This module defines the [`TournamentStrategy`] trait and several built-in strategies
//! (e.g., Swiss, Round Robin, Single Player) used by the server-side evaluator to structure
//! tournaments and process match results.
//!
//! Although this trait and types are public to allow advanced users to define custom strategies,
//! they are not intended for direct use or manual orchestration of tournaments.
//!
//! # Provided Strategies
//! - [`RoundRobinTournament`]: Every agent plays every other agent. Quite slow.
//! - [`SwissTournament`]: Pairings based on score, with optional tie-breakers. Mush faster than Round Robin
//! - [`SinglePlayerTournament`]: Each agent plays independently multiple times.
//!
//! # Implementing a Custom Strategy
//! To implement a new tournament format, define your own type that implements
//! [`TournamentStrategy`].
//!
//! The server will call `add_agents`, then repeatedly call `advance_round`
//! until it returns an empty list. Once finished, `get_final_scores` is used to produce the ranking.

use std::{
    cmp,
    collections::{HashMap, HashSet},
    sync::Arc,
};

use tracing::info;

use crate::{agent::Agent, match_runner::MatchResult};

/// A trait defining how agents are grouped, matched, and scored in a tournament.
///
/// Implement this trait to define a custom tournament format. The tournament is responsible for:
/// - Receiving a set of agents
/// - Generating matches to run per round
/// - Processing match results
/// - Producing a final score for each agent
pub trait TournamentStrategy {
    /// The score type produced at the end of the tournament.
    type FinalScore: Ord;

    /// Adds a list of agents to the tournament.
    ///
    /// Must be called before advancing rounds. This method may be used to initialize internal
    /// score or pairing state.
    fn add_agents(&mut self, agents: Vec<Arc<Agent>>);

    /// Returns new matchups for the next round, based on the latest match results.
    ///
    /// If the returned list is empty, the tournament is finished.
    ///
    /// Each match is a list of agents (usually 2), and will be scored externally.
    fn advance_round(&mut self, scores: Vec<MatchResult>) -> Vec<Vec<Arc<Agent>>>;

    /// Returns the number of players per match required by this strategy.
    ///
    /// This value must match the length of each sub-`Vec` returned by `advance_round`.
    fn players_per_match(&self) -> usize;

    /// Returns the final scores for all agents once the tournament is complete.
    fn get_final_scores(&self) -> HashMap<Arc<Agent>, Self::FinalScore>;
}

/// Score summary for agents in two-player tournaments.
///
/// Used in `SwissTournament` and `RoundRobinTournament`. This type tracks the total number of wins,
/// draws, losses, and an optional tie-breaker value.
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Debug, Clone, Copy)]
pub struct TwoPlayersGameScore {
    /// Number of wins.
    pub num_win: u32,
    /// Number of draws.
    pub num_draw: u32,
    /// Number of losses.
    pub num_lose: u32,
    /// Additional tie-breaker value.
    pub tie_breaker: u32,
}

impl std::fmt::Display for TwoPlayersGameScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "win: {}, draw: {}, loose: {}, tie-breaker: {}",
            self.num_win, self.num_draw, self.num_lose, self.tie_breaker
        )
    }
}

/// A Swiss-style tournament strategy for two-player games. Does not follow strictly the Swiss
/// tournament rules.
///
/// Agents are paired based on their current score. The number of rounds can be fixed,
/// or automatically determined as `ceil(log2(num_players))`.
pub struct SwissTournament {
    agents: Vec<Arc<Agent>>,
    round: usize,
    max_rounds: usize,
    num_match_per_pair: usize,
    scores: HashMap<Arc<Agent>, (TwoPlayersGameScore, HashSet<Arc<Agent>>)>,
}

impl SwissTournament {
    /// Creates a new Swiss tournament with the number of matches per pair and automatic number of rounds.
    ///
    /// The number of rounds is determined automatically based on the number of agents,
    /// using the formula `ceil(log2(n))`, where `n` is the number of players.
    ///
    /// Each pair of agents will play `num_match_per_pair` games per round. If the game is
    /// asymetric, this number should be even to ensure fairness.
    /// The order of players will alternate between games to account for side asymmetry.
    /// The results of these games are aggregated into a single win/loss/draw outcome
    /// for Swiss pairing and scoring purposes.
    pub fn with_auto_rounds(num_match_per_pair: usize) -> Self {
        Self::new(0, num_match_per_pair)
    }

    /// Creates a new Swiss tournament with a specified number of rounds and matches per pair.
    ///
    /// If `max_rounds` is set to `0`, this is equivalent to using
    /// [`with_auto_rounds`](Self::with_auto_rounds).
    ///
    /// Each pair of agents will play `num_match_per_pair` games per round.
    /// The order of players will alternate between games to account for side asymmetry.
    /// The results of these games are aggregated into a single win/loss/draw outcome
    /// for Swiss pairing and scoring purposes.
    pub fn new(max_rounds: usize, num_match_per_pair: usize) -> Self {
        assert!(
            num_match_per_pair >= 1,
            "Must play at least one match per pairing."
        );
        Self {
            agents: vec![],
            round: 0,
            max_rounds,
            num_match_per_pair,
            scores: HashMap::new(),
        }
    }

    fn update_tie_breakers(&mut self) {
        // Median tie-breaker: for each player, tie-breaker is the sum of player's adversaries, minus extrema
        // https://en.wikipedia.org/wiki/Tie-breaking_in_Swiss-system_tournaments#Median_/_Buchholz_/_Solkoff
        for agent in &self.agents {
            let mut adv_scores = vec![];
            for adv in &self.scores[agent].1 {
                let adv_score = &self.scores[adv].0;
                let adv_score = adv_score.num_win * 2 + adv_score.num_draw;
                adv_scores.push(adv_score);
            }
            let min = *adv_scores.iter().min().unwrap_or(&0);
            let max = *adv_scores.iter().max().unwrap_or(&0);
            self.scores.get_mut(agent).unwrap().0.tie_breaker = if adv_scores.len() <= 1 {
                0
            } else {
                adv_scores.iter().sum::<u32>() - min - max
            };
        }
    }

    fn update_scores(&mut self, match_results: Vec<MatchResult>) {
        let mut pair_results: HashMap<(Arc<Agent>, Arc<Agent>), Vec<(f32, f32)>> =
            HashMap::with_capacity(match_results.len() / self.num_match_per_pair);

        // 1. aggregate score per pair
        for result in match_results {
            assert!(result.len() == 2, "not two players match ??");

            let (a, score_a) = &result[0];
            let (b, score_b) = &result[1];

            assert!(
                !Arc::ptr_eq(a, b) && a.id != b.id,
                "should not be able to play against yourself"
            );

            let key = if a.id < b.id {
                (a.clone(), b.clone())
            } else {
                (b.clone(), a.clone())
            };

            let score = if a.id < b.id {
                (*score_a, *score_b)
            } else {
                (*score_b, *score_a)
            };

            pair_results.entry(key).or_default().push(score);
        }

        // 2. update swiss score
        for ((a, b), scores) in pair_results.into_iter() {
            let (score_a, score_b) = scores
                .into_iter()
                .fold((0.0, 0.0), |acu, (score_a, score_b)| {
                    (acu.0 + score_a, acu.1 + score_b)
                });
            let is_draw = (score_a - score_b).abs() < f32::EPSILON;
            if is_draw {
                self.scores.get_mut(&a).unwrap().0.num_draw += 1;
                self.scores.get_mut(&b).unwrap().0.num_draw += 1;
            } else if score_a > score_b {
                self.scores.get_mut(&a).unwrap().0.num_win += 1;
                self.scores.get_mut(&b).unwrap().0.num_lose += 1;
            } else {
                self.scores.get_mut(&a).unwrap().0.num_lose += 1;
                self.scores.get_mut(&b).unwrap().0.num_win += 1;
            }

            self.scores.get_mut(&a).unwrap().1.insert(b.clone());
            self.scores.get_mut(&b).unwrap().1.insert(a.clone());
        }
    }
}

impl TournamentStrategy for SwissTournament {
    fn advance_round(&mut self, scores: Vec<MatchResult>) -> Vec<Vec<Arc<Agent>>> {
        self.update_scores(scores);
        self.update_tie_breakers();

        if self.round >= self.max_rounds {
            return vec![];
        }

        let mut sorted = self.agents.clone();
        sorted.sort_by_key(|agent| cmp::Reverse(self.scores[agent].0));

        //FIXME: prevent already-played match pairing
        // Pair per score
        let pending = sorted
            .chunks(2)
            .flat_map(|chunk| {
                if chunk.len() == 2 {
                    let a = &chunk[0];
                    let b = &chunk[1];
                    (0..self.num_match_per_pair)
                        .map(|i| {
                            // permute order for each match
                            if i % 2 == 0 {
                                vec![a.clone(), b.clone()]
                            } else {
                                vec![b.clone(), a.clone()]
                            }
                        })
                        .collect::<Vec<_>>()
                    // Some(chunk.to_vec())
                } else {
                    // NOTE: According to standard Swiss rules, a player should not receive more than one bye.
                    // We don’t enforce that here for simplicity, but it can be added by tracking byes per player.
                    let a = &chunk[0];
                    // Player has no opponent this round (bye). Award a free win but no tie-breaker.
                    println!("BYE FOR {}", &a.name);
                    self.scores.get_mut(a).unwrap().0.num_win += 1;
                    vec![] //no match for the last one (if odd num of players)
                }
            })
            .collect::<Vec<_>>();

        self.round += 1;
        pending
    }

    fn players_per_match(&self) -> usize {
        2
    }

    fn add_agents(&mut self, agents: Vec<Arc<Agent>>) {
        self.agents = agents;
        if self.max_rounds == 0 {
            let n = self.agents.len();
            self.max_rounds = f32::log2(n as f32).ceil() as usize;
            info!("Max number of rounds: {}", self.max_rounds);
        }
        for agent in &self.agents {
            self.scores.insert(
                agent.clone(),
                (TwoPlayersGameScore::default(), HashSet::new()),
            );
        }
    }

    type FinalScore = TwoPlayersGameScore;

    fn get_final_scores(&self) -> HashMap<Arc<Agent>, Self::FinalScore> {
        //NOTE: Tie-breakers should already be up-to-date
        self.scores
            .iter()
            .map(|(agent, (score, _adv))| (agent.clone(), *score))
            .collect()
    }
}

/// A round-robin tournament where each agent plays against every other agent.
///
/// If `symmetric` is false, each pair is evaluated in both directions (A vs B and B vs A).
pub struct RoundRobinTournament {
    scores: HashMap<Arc<Agent>, TwoPlayersGameScore>,
    agents: Vec<Arc<Agent>>,
    symmetric: bool,
}

impl RoundRobinTournament {
    /// Creates a new Round Robin tournament.
    ///
    /// Set `symmetric = true` if A vs B is equivalent to B vs A.
    pub fn new(symmetric: bool) -> Self {
        Self {
            symmetric,
            agents: vec![],
            scores: HashMap::new(),
        }
    }
}

impl TournamentStrategy for RoundRobinTournament {
    fn advance_round(&mut self, scores: Vec<MatchResult>) -> Vec<Vec<Arc<Agent>>> {
        for match_result in scores {
            let best_score =
                match_result.iter().fold(
                    -f32::INFINITY,
                    |acu, (_agent, score)| if acu < *score { *score } else { acu },
                );
            let is_draw = match_result
                .iter()
                .all(|(_agent, score)| *score == best_score);
            for (agent, score) in &match_result {
                if is_draw {
                    self.scores.entry(agent.clone()).or_default().num_draw += 1;
                } else if *score == best_score {
                    self.scores.entry(agent.clone()).or_default().num_win += 1;
                } else
                /* *score != best_score */
                {
                    self.scores.entry(agent.clone()).or_default().num_lose += 1;
                }
            }
        }
        //TODO: tie-breakers
        // Not quite an official source, but that will do: https://mtgoldframe.com/the-round-robin-tournament-system-rules-scoring-and-tiebreakers/

        if !self.scores.is_empty() {
            // first (and only) round was already ran
            return vec![];
        }

        let n = self.agents.len();
        let mut pending = vec![];
        for i in 0..n {
            for j in i..n {
                pending.push(vec![self.agents[i].clone(), self.agents[j].clone()]);
                if !self.symmetric {
                    pending.push(vec![self.agents[j].clone(), self.agents[i].clone()]);
                }
            }
        }

        pending
    }

    fn players_per_match(&self) -> usize {
        2
    }

    fn add_agents(&mut self, agents: Vec<Arc<Agent>>) {
        self.agents = agents;
    }

    type FinalScore = TwoPlayersGameScore;

    fn get_final_scores(&self) -> HashMap<Arc<Agent>, Self::FinalScore> {
        self.scores.clone()
    }
}

/// Holds a list of scores for an agent in a single-player tournament.
///
/// Implements ordering by comparison.
#[derive(PartialEq, Debug, Clone, Default)]
pub struct SinglePlayerScore(pub Vec<f32>);

impl Eq for SinglePlayerScore {} // That's it ??

impl PartialOrd for SinglePlayerScore {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SinglePlayerScore {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}

/// A tournament where each agent plays independently across multiple games.
///
/// Each agent is evaluated in isolation, and scores are stored as lists of `f32`.
pub struct SinglePlayerTournament {
    game_per_agent: usize,
    agents: Vec<Arc<Agent>>,
    scores: HashMap<Arc<Agent>, SinglePlayerScore>,
}

impl SinglePlayerTournament {
    /// Creates a new single-player tournament.
    ///
    /// `game_per_agent` determines how many games each agent will play.
    pub fn new(game_per_agent: usize) -> Self {
        Self {
            game_per_agent,
            agents: vec![],
            scores: HashMap::new(),
        }
    }
}

impl TournamentStrategy for SinglePlayerTournament {
    fn advance_round(&mut self, match_results: Vec<MatchResult>) -> Vec<Vec<Arc<Agent>>> {
        for match_result in match_results {
            for (agent, score) in match_result {
                self.scores.entry(agent).or_default().0.push(score);
            }
        }

        // the first and only round
        let mut pending = vec![];
        for agent in self.agents.drain(..) {
            // drain so that no more matches are returned after first round
            pending.append(&mut vec![vec![agent.clone()]; self.game_per_agent]);
        }
        pending
    }

    fn players_per_match(&self) -> usize {
        1
    }

    fn add_agents(&mut self, agents: Vec<Arc<Agent>>) {
        self.agents = agents;
    }

    type FinalScore = SinglePlayerScore;

    fn get_final_scores(&self) -> HashMap<Arc<Agent>, Self::FinalScore> {
        self.scores.clone()
    }
}

//TODO: knockout AKA single elimination tournament
