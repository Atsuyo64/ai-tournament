use crate::agent::Agent;
use crate::constraints::Constraints;
use crate::match_runner::{MatchResult, MatchSettings};
use crate::tournament_strategy::TournamentStrategy;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScoreKey;

#[derive(Clone)]
pub struct Scores;

impl Scores {
    pub fn init() -> Self {
        Scores
    }

    pub fn add_score<I: IntoIterator<Item = (Arc<Agent>, f32)>>(&mut self, _score: I) {
        //TODO:
    }

    pub fn get_key(&self, _agent: &Arc<Agent>) -> ScoreKey {
        ScoreKey
    }
}

pub struct TournamentScheduler<S: TournamentStrategy> {
    // pub agents: Vec<Arc<Agent>>,
    pub scores: Scores,
    pub resources: Constraints,
    pub pending_matches: Vec<Vec<Arc<Agent>>>,
    running_matches: usize,
    strategy: S,
}

impl<S: TournamentStrategy> TournamentScheduler<S> {
    pub fn new(
        // agents: Vec<Arc<Agent>>,
        resources: Constraints,
        strategy: S,
    ) -> Self {
        let scores = Scores::init(); //&agents);
        TournamentScheduler {
            // agents,
            scores,
            resources,
            pending_matches: vec![],
            running_matches: 0,
            strategy,
        }
    }

    pub fn advance(&mut self) -> Vec<MatchSettings> {
        let mut matches_to_run = vec![];

        // Generate new round if needed
        if self.running_matches == 0
            && self.pending_matches.is_empty()
            && !self.strategy.is_complete()
        {
            self.strategy.advance_round(&self.scores);
            self.pending_matches = self.strategy.get_pending_tuples();
        }

        let cpu_per_match = self.resources.cpus_per_agent * self.strategy.players_per_match();
        let ram_per_match =
            self.resources.agent_ram * self.strategy.players_per_match();
        // Schedule as many pending matches as long as there is enough resources
        let mut remaining = vec![];
        for v in self.pending_matches.drain(..) {
            if let Some(resources) = self.resources.try_take(cpu_per_match, ram_per_match) {
                matches_to_run.push(MatchSettings {
                    ordered_player: v,
                    resources,
                });
            } else {
                remaining.push(v);
            }
        }
        self.pending_matches = remaining;
        self.running_matches += matches_to_run.len();
        matches_to_run
    }

    pub fn on_result(&mut self, result: MatchResult) -> Vec<MatchSettings> {
        self.scores.add_score(result.results);
        self.resources.add(result.resources_freed);
        self.running_matches -= 1;
        self.advance()
    }

    pub fn is_finished(&self) -> bool {
        self.strategy.is_complete()
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
}
