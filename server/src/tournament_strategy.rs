use std::sync::Arc;

use crate::{agent::Agent, tournament::Scores};

pub trait TournamentStrategy {
    //TODO: if deffering responsability to choose tournament to caller, add `init` parameter-only constructor (actually not here: parameters are strategy-specific)
    fn advance_round(&mut self, scores: &Scores);
    fn get_pending_tuples(&mut self) -> Vec<Vec<Arc<Agent>>>;
    fn is_complete(&self) -> bool;
    fn players_per_match(&self) -> usize;
}

pub struct SwissTournament {
    agents: Vec<Arc<Agent>>,
    round: usize,
    max_rounds: usize,
    pending: Vec<Vec<Arc<Agent>>>,
}

impl SwissTournament {
    pub fn new(agents: Vec<Arc<Agent>>, max_rounds: usize) -> Self {
        Self {
            agents,
            round: 0,
            max_rounds,
            pending: vec![],
        }
    }
}

impl TournamentStrategy for SwissTournament {
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

pub struct RoundRobinTournament {
    pending: Vec<Vec<Arc<Agent>>>,
}

impl RoundRobinTournament {
    /// symmetric means `A VS B` should give the same result as `B VS A`
    pub fn new(agents: Vec<Arc<Agent>>, symmetric: bool) -> Self {
        let n = agents.len();
        let mut pending = vec![];
        for i in 0..n {
            for j in i..n {
                pending.push(vec![agents[i].clone(), agents[j].clone()]);
                if !symmetric {
                    pending.push(vec![agents[j].clone(), agents[i].clone()]);
                }
            }
        }
        Self { pending }
    }
}

impl TournamentStrategy for RoundRobinTournament {
    fn advance_round(&mut self, _scores: &Scores) {}

    fn get_pending_tuples(&mut self) -> Vec<Vec<Arc<Agent>>> {
        std::mem::take(&mut self.pending)
    }

    fn is_complete(&self) -> bool {
        self.pending.is_empty()
    }
    fn players_per_match(&self) -> usize {
        2
    }
}

pub struct SingleplayerTournament {
    pending: Vec<Vec<Arc<Agent>>>,
}

impl SingleplayerTournament {
    /// game_per_agent 
    pub fn new(agents: Vec<Arc<Agent>>, game_per_agent : usize) -> Self {
        let mut pending = vec![];
        for agent in agents {
            pending.append(&mut vec![vec![agent];game_per_agent]);
        }
        Self { pending }
    }
}

impl TournamentStrategy for SingleplayerTournament {
    fn advance_round(&mut self, _scores: &Scores) {}

    fn get_pending_tuples(&mut self) -> Vec<Vec<Arc<Agent>>> {
        std::mem::take(&mut self.pending)
    }

    fn is_complete(&self) -> bool {
        self.pending.is_empty()
    }
    fn players_per_match(&self) -> usize {
        1
    }
}