use crate::agent::Agent;
use crate::constraints::Constraints;
use crate::match_runner::{MatchResult, MatchSettings, RunnerResult};
use crate::tournament_strategy::TournamentStrategy;
use std::collections::HashMap;
use std::mem;
use std::sync::Arc;

// #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
// pub struct ScoreKey(i32,i32); //score, tie breaker //NOTE: might change to u32

// #[derive(Debug, Clone, Default)]
// pub struct AgentScoreEntry {
//     pub all_scores: Vec<f32>, // Win,Draw,Lose Enum ? Rank uint ?
//     pub sum: f32,
//     pub adversaries: std::collections::HashSet<Arc<Agent>>,
// }

// pub struct PrettyScore {
//     pub win:u32,
//     pub draw:u32,
//     pub lose:u32,
// }

// impl Scores {
//     pub fn init() -> Self {
//         Scores(HashMap::new())
//     }

//     pub fn add_match_result<I: IntoIterator<Item = (Arc<Agent>, f32)>>(&mut self, score: I) {
//         let mut participants = vec![];
//         for (agent, score) in score {
//             participants.push(agent.clone());
//             let e = self.0.entry(agent).or_default();
//             e.all_scores.push(score);
//             e.sum += score;
//         }
//         for agent in participants.iter() {
//             for participant in participants.iter() {
//                 if agent != participant {
//                     self.0.get_mut(agent).unwrap().adversaries.insert(participant.clone());
//                 }
//             }
//         }
//     }

//     pub fn get_key(&self, agent: &Arc<Agent>) -> ScoreKey {
//         if let Some(e) = self.0.get(agent) {
//             let adv = e.adversaries.iter().filter_map(|a| self.0.get(a).map(|e|e.sum)).collect::<Vec<_>>();
//             let min = adv.iter().fold(f32::MAX, |a, &b| a.min(b));
//             let max = adv.iter().fold(f32::MIN, |a, &b| a.max(b));
//             let adv_value = if adv.len() <= 1 { 0.0 } else { adv.iter().sum::<f32>() - min - max };
//             let adv_score = (adv_value * 2.0)  as i32;
//             let agent_score = (e.sum * 2.0) as i32;
//             ScoreKey ( agent_score, adv_score )
//         } else {
//             ScoreKey ( 0, 0 )
//         }
//     }

//     pub fn get_printable_score(&self, _agent: &Arc<Agent>) -> PrettyScore {
//         todo!() //TODO:
//     }
// }

pub struct TournamentScheduler<S: TournamentStrategy> {
    // pub agents: Vec<Arc<Agent>>,
    scores: Vec<MatchResult>,
    resources: Constraints,
    pending_matches: Vec<Vec<Arc<Agent>>>,
    strategy: S,
    running_matches: usize,
    is_finished: bool,
}

impl<S: TournamentStrategy> TournamentScheduler<S> {
    pub fn new(
        // agents: Vec<Arc<Agent>>,
        resources: Constraints,
        strategy: S,
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
            self.pending_matches = self.strategy.advance_round(mem::take(&mut self.scores));

            if self.pending_matches.is_empty() {
                // no more matches from `strategy`
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

    pub fn on_result(&mut self, result: RunnerResult) -> Vec<MatchSettings> {
        self.scores.push(result.results);
        self.resources.add(result.resources_freed);
        self.running_matches -= 1;
        self.advance()
    }

    pub fn is_finished(&self) -> bool {
        self.is_finished // self.strategy.is_complete() && self.pending_matches.is_empty() && self.running_matches == 0
    }

    pub fn final_scores(&self) -> HashMap<Arc<Agent>, S::FinalScore> {
        self.strategy.get_final_scores()
    }
}
