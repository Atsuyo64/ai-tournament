use std::sync::Arc;

use crate::{agent::Agent, tournament::Scores};

pub trait TournamentStrategy { //CHECK: template number of player per match ? (probably a bad idea)
    fn advance_round(&mut self, scores: &Scores);
    fn get_pending_tuples(&mut self) -> Vec<Vec<Arc<Agent>>>;
    fn is_complete(&self) -> bool;
    fn players_per_match(&self) -> usize;
}

pub struct SwissStrategy {
    agents: Vec<Arc<Agent>>,
    round: usize,
    max_rounds: usize,
    pending: Vec<Vec<Arc<Agent>>>,
}

impl SwissStrategy {
    pub fn new(agents: Vec<Arc<Agent>>, max_rounds: usize) -> Self {
        Self {
            agents,
            round: 0,
            max_rounds,
            pending: vec![],
        }
    }
}

impl TournamentStrategy for SwissStrategy {
    fn advance_round(&mut self, scores: &Scores) {
        if self.round >= self.max_rounds {
            return;
        }

        let mut sorted = self.agents.clone();
        sorted.sort_by_key(|agent| std::cmp::Reverse(scores.get_key(agent)));

        // Pair per score
        self.pending = sorted
            .chunks(2)
            .filter_map(|chunk| {
                if chunk.len() == 2 {
                    Some(chunk.to_vec())
                } else {
                    //TODO: bye
                    // Solution (?): return it anyway and update the scheduler to add +1 when len() < strat.players_per_match ?
                    // Or return a MatchKind with either Normal(Vec<>) or Bye(Vec<>)
                    None
                }
            })
            .collect();

        self.round += 1;
    }

    fn get_pending_tuples(&mut self) -> Vec<Vec<Arc<Agent>>> {
        std::mem::take(&mut self.pending)
    }

    fn is_complete(&self) -> bool {
        self.round >= self.max_rounds && self.pending.is_empty()
    }
    fn players_per_match(&self) -> usize {
        2
    }
    
}
