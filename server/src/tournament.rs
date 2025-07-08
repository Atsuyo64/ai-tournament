use crate::agent::Agent;
use crate::constraints::Constraints;
use crate::match_runner::{MatchResult, MatchSettings};
use crate::tournament_strategy::TournamentStrategy;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScoreKey(i32); //FIXME: two fields

#[derive(Debug, Clone, Default)]
struct AgentScoreEntry {
    all_scores: Vec<f32>,
    sum: f32,
    adversaries: std::collections::HashSet<Arc<Agent>>,
}

pub struct PrettyScore {
    pub win:u32,
    pub draw:u32,
    pub lose:u32,
}

#[derive(Clone,Debug)]
pub struct Scores(HashMap<Arc<Agent>, AgentScoreEntry>);

impl Scores {
    pub fn init() -> Self {
        Scores { 0: HashMap::new() }
    }

    pub fn add_score<I: IntoIterator<Item = (Arc<Agent>, f32)>>(&mut self, score: I) {
        let mut participants = vec![];
        for (agent, score) in score {
            participants.push(agent.clone());
            let e = self.0.entry(agent).or_default();
            e.all_scores.push(score);
            e.sum += score;
        }
        for agent in participants.iter() {
            for participant in participants.iter() {
                if agent != participant {
                    self.0.get_mut(agent).unwrap().adversaries.insert(participant.clone());
                }
            }
        }
    }

    pub fn get_key(&self, agent: &Arc<Agent>) -> ScoreKey {
        if let Some(e) = self.0.get(agent) {
            let adv = e.adversaries.iter().filter_map(|a| self.0.get(a).map(|e|e.sum)).collect::<Vec<_>>();
            let min = adv.iter().fold(f32::MAX, |a, &b| a.min(b));
            let max = adv.iter().fold(f32::MIN, |a, &b| a.max(b));
            let adv_value = adv.iter().sum::<f32>() - min - max;
            let adv_value = adv_value.clamp(0.0, 16384.0);
            let agent_score = (e.sum * 2.0 * 16384.0) as i32;
            let adv_score = (adv_value * 2.0) as i32;
            ScoreKey ( agent_score + adv_score )
        } else {
            ScoreKey ( 0 )
        }
    }

    pub fn get_printable_score(&self, _agent: &Arc<Agent>) -> PrettyScore {
        todo!() //TODO: 
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

    pub fn on_result(&mut self, result: MatchResult) -> Vec<MatchSettings> {
        self.scores.add_score(result.results);
        self.resources.add(result.resources_freed);
        self.running_matches -= 1;
        self.advance()
    }

    pub fn is_finished(&self) -> bool {
        self.strategy.is_complete() && self.pending_matches.is_empty() && self.running_matches == 0
    }

    pub fn final_scores(&self) -> Scores {
        self.scores.clone()
    }
}
