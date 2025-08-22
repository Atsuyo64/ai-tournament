use tracing::trace;

use crate::agent::Agent;
use crate::constraints::Constraints;
use crate::match_runner::{MatchResult, MatchSettings, RunnerResult};
use crate::tournament_strategy::TournamentStrategy;
use std::collections::HashMap;
use std::mem;
use std::sync::Arc;

pub struct TournamentScheduler<T: TournamentStrategy<S>, S>
where
    S: PartialOrd,
{
    // pub agents: Vec<Arc<Agent>>,
    scores: Vec<MatchResult<S>>,
    resources: Constraints,
    pending_matches: Vec<Vec<Arc<Agent>>>,
    strategy: T,
    running_matches: usize,
    is_finished: bool,
}

impl<T: TournamentStrategy<S>, S: PartialOrd> TournamentScheduler<T, S> {
    pub fn new(
        // agents: Vec<Arc<Agent>>,
        resources: Constraints,
        strategy: T,
    ) -> Self {
        TournamentScheduler {
            // agents,
            scores: vec![],
            resources,
            pending_matches: vec![],
            running_matches: 0,
            strategy,
            is_finished: false,
        }
    }

    pub fn advance(&mut self) -> Vec<MatchSettings> {
        let mut matches_to_run = vec![];

        // Generate new round if needed
        if self.running_matches == 0 && self.pending_matches.is_empty() && !self.is_finished {
            trace!("next round");
            self.pending_matches = self.strategy.advance_round(mem::take(&mut self.scores));

            if self.pending_matches.is_empty() {
                // no more matches from `strategy`
                trace!("no more matches");
                self.is_finished = true;
            }
        }

        let cpu_per_match = self.resources.cpus_per_agent * self.strategy.players_per_match(); //FIXME: can be computed for each match
        let ram_per_match = self.resources.agent_ram * self.strategy.players_per_match();
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

    pub fn on_result(&mut self, result: RunnerResult<S>) -> Vec<MatchSettings> {
        self.scores.push(result.results);
        self.resources.add(result.resources_freed);
        self.running_matches -= 1;
        self.advance()
    }

    /// All tournament matches ran and finished
    pub fn is_finished(&self) -> bool {
        self.is_finished // self.strategy.is_complete() && self.pending_matches.is_empty() && self.running_matches == 0
    }

    pub fn final_scores(&self) -> HashMap<Arc<Agent>, T::FinalScore> {
        self.strategy.get_final_scores()
    }
}
