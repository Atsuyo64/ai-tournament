use std::{
    cmp,
    collections::{HashMap, HashSet},
    sync::Arc,
};

use tracing::info;

use crate::{agent::Agent, match_runner::MatchResult};

pub trait TournamentStrategy {
    fn add_agents(&mut self, agents: Vec<Arc<Agent>>);
    /// get new matches based on current round `scores`.
    ///
    /// If result is empty, the Tournament is considered finished
    fn advance_round(&mut self, scores: Vec<MatchResult>) -> Vec<Vec<Arc<Agent>>>;
    fn players_per_match(&self) -> usize;
    type FinalScore: Ord;
    fn get_final_scores(&self) -> HashMap<Arc<Agent>, Self::FinalScore>;
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Debug, Clone, Copy)]
pub struct TwoPlayersGameScore {
    pub num_win: u32,
    pub num_draw: u32,
    pub num_lose: u32,
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

pub struct SwissTournament {
    agents: Vec<Arc<Agent>>,
    round: usize,
    max_rounds: usize,
    scores: HashMap<Arc<Agent>, (TwoPlayersGameScore, HashSet<Arc<Agent>>)>,
}

impl SwissTournament {
    /// Creates a new Swiss tournament.
    ///
    /// Set `max_rounds` to `0` to automatically determine the number of rounds,
    /// which will be calculated as `ceil(log2(num_players))`.
    pub fn new(max_rounds: usize) -> Self {
        Self {
            agents: vec![],
            round: 0,
            max_rounds,
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

    fn update_scores(&mut self, scores: Vec<MatchResult>) {
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
                    self.scores.get_mut(agent).unwrap().0.num_draw += 1;
                } else if *score == best_score {
                    self.scores.get_mut(agent).unwrap().0.num_win += 1;
                } else
                /* *score != best_score */
                {
                    self.scores.get_mut(agent).unwrap().0.num_lose += 1;
                }
                for (other, _) in &match_result {
                    if other != agent {
                        self.scores.get_mut(agent).unwrap().1.insert(other.clone());
                    }
                }
            }
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
            .filter_map(|chunk| {
                if chunk.len() == 2 {
                    Some(chunk.to_vec())
                } else {
                    //TODO: bye
                    //NOTE: now that scores are handled by Strategy, should be able to just add Bye Match results in internal score
                    // Solution (?): return it anyway and update the scheduler to add +1 when len() < strategy.players_per_match ?
                    // Or return a MatchKind with either Normal(Vec<>) or Bye(Vec<>)
                    None
                }
            })
            .collect::<Vec<_>>();

        self.round += 1;
        pending
    }

    // fn get_pending_tuples(&mut self) -> Vec<Vec<Arc<Agent>>> {
    //     std::mem::take(&mut self.pending)
    // }

    // fn is_complete(&self) -> bool {
    //     self.round >= self.max_rounds && self.pending.is_empty()
    // }

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

pub struct RoundRobinTournament {
    scores: HashMap<Arc<Agent>, TwoPlayersGameScore>,
    agents: Vec<Arc<Agent>>,
    symmetric: bool,
}

impl RoundRobinTournament {
    /// symmetric means `A VS B` should give the same result as `B VS A`
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

    // fn get_pending_tuples(&mut self) -> Vec<Vec<Arc<Agent>>> {
    //     std::mem::take(&mut self.pending)
    // }

    // fn is_complete(&self) -> bool {
    //     self.pending.is_empty()
    // }

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

pub struct SinglePlayerTournament {
    game_per_agent: usize,
    agents: Vec<Arc<Agent>>,
    scores: HashMap<Arc<Agent>, SinglePlayerScore>,
}

impl SinglePlayerTournament {
    /// game_per_agent
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

    // fn get_pending_tuples(&mut self) -> Vec<Vec<Arc<Agent>>> {
    //     std::mem::take(&mut self.pending)
    // }

    // fn is_complete(&self) -> bool {
    //     self.pending.is_empty()
    // }

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
