use agent_interface::game_info::GameInfo;

use crate::agent::Agent;
use crate::constraints::Constraints;
use crate::match_runner::{MatchResult, MatchSettings};
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScoreKey;

#[derive(Clone)]
pub struct Scores;

impl Scores {
    pub fn init<I: IntoIterator<Item = Arc<Agent>>>(_agents: &I) -> Self {
        Scores
    }

    pub fn add_score<I: IntoIterator<Item = (Arc<Agent>, f32)>>(&mut self, _score: I) {
        //TODO:
    }

    pub fn get_key(&self, _agent: &Arc<Agent>) -> ScoreKey {
        ScoreKey
    }
}

pub struct Tournament {
    pub agents: Vec<Arc<Agent>>,
    pub scores: Scores,
    pub resources: Constraints,
    pub game_info: GameInfo,
    pub pending_matches: Vec<Vec<Arc<Agent>>>,
    pub current_round: usize,
    pub total_rounds: usize,
    running_matches: usize,
}

impl Tournament {
    pub fn new(
        agents: Vec<Arc<Agent>>,
        resources: Constraints,
        game_info: GameInfo,
        total_rounds: usize,
    ) -> Self {
        let scores = Scores::init(&agents);
        Tournament {
            agents,
            scores,
            resources,
            game_info,
            pending_matches: vec![],
            current_round: 0,
            total_rounds,
            running_matches: 0,
        }
    }

    pub fn tick(&mut self) -> Vec<MatchSettings> {
        let mut matches_to_run = vec![];

        // Generate new round if needed
        if self.running_matches == 0 && self.pending_matches.is_empty() && self.current_round < self.total_rounds {
            self.current_round += 1;
            self.generate_pairings();
        }

        let cpu_per_match = self.resources.cpus_per_agent * self.game_info.num_player as usize;
        let ram_per_match =
            self.resources.agent_ram.unwrap_or(0) * self.game_info.num_player as usize;
        // Schedule as many pending matches as we have free CPUs
        let mut remaining = vec![];
        for v in self.pending_matches.drain(..) {
            if let Some(resources) = self.resources.try_take(cpu_per_match, ram_per_match) {
                self.running_matches += 1;
                matches_to_run.push(MatchSettings {
                    ordered_player: v,
                    resources,
                });
            } else {
                remaining.push(v);
            }
        }
        self.pending_matches = remaining;
        matches_to_run
    }

    pub fn on_result(&mut self, result: MatchResult) -> Vec<MatchSettings> {
        // Update scores
        self.running_matches -= 1;
        self.scores.add_score(result.results);
        self.resources.add(result.resources);
        self.tick()
    }

    pub fn is_finished(&self) -> bool {
        self.current_round >= self.total_rounds
            && self.pending_matches.is_empty()
            && self.running_matches == 0
    }

    pub fn final_scores(&self) -> Scores {
        // let mut results: Vec<_> = self.agents
        //     .iter()
        //     .map(|a| (a.clone(), *self.scores.get(&a.id).unwrap()))
        //     .collect();
        // results.sort_by_key(|(_, score)| std::cmp::Reverse(*score));
        // results
        self.scores.clone()
    }

    fn generate_pairings(&mut self) {
        assert!(self.pending_matches.is_empty());
        // Sort agents by score
        let mut sorted = self.agents.clone();
        sorted.sort_by_key(|agent| std::cmp::Reverse(self.scores.get_key(agent)));

        // Pair top with next (simple Swiss pairing)
        self.pending_matches = sorted
            .chunks(2)
            .filter_map(|chunk| {
                if chunk.len() == 2 {
                    Some(chunk.to_vec())
                } else {
                    //TODO: bye
                    None
                }
            })
            .collect();
    }
}
