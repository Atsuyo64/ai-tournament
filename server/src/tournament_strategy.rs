use std::sync::Arc;

use tracing::trace;

use crate::{agent::Agent, tournament::Scores};

pub trait TournamentStrategy {
    fn add_agents(&mut self, agents:Vec<Arc<Agent>>);
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
    /// Creates a new Swiss tournament.
    ///
    /// Set `max_rounds` to `0` to automatically determine the number of rounds,
    /// which will be calculated as `ceil(log2(num_players))` based on the number of players.
    pub fn new(max_rounds: usize) -> Self {
        Self {
            agents: vec![],
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

        //FIXME: prevent already-played match pairing
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
    
    fn add_agents(&mut self, agents:Vec<Arc<Agent>>) {
        self.agents = agents;
        if self.max_rounds == 0 {
            let n = self.agents.len();
            self.max_rounds = f32::log2(n as f32).ceil() as usize;
            trace!("Max number of rounds: {}",self.max_rounds);
        }
    }
}

pub struct RoundRobinTournament {
    symmetric:bool ,
    pending: Vec<Vec<Arc<Agent>>>,
}

impl RoundRobinTournament {
    /// symmetric means `A VS B` should give the same result as `B VS A`
    pub fn new(symmetric: bool) -> Self {
        Self { symmetric, pending: vec![] }
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
    
    fn add_agents(&mut self, agents:Vec<Arc<Agent>>) {
        let n = agents.len();
        let mut pending = vec![];
        for i in 0..n {
            for j in i..n {
                pending.push(vec![agents[i].clone(), agents[j].clone()]);
                if !self.symmetric {
                    pending.push(vec![agents[j].clone(), agents[i].clone()]);
                }
            }
        }
        self.pending = pending;
    }
}

pub struct SingleplayerTournament {
    game_per_agent: usize, 
    pending: Vec<Vec<Arc<Agent>>>,
}

impl SingleplayerTournament {
    /// game_per_agent 
    pub fn new(game_per_agent : usize) -> Self {
        Self { game_per_agent, pending: vec![] }
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
    
    fn add_agents(&mut self, agents:Vec<Arc<Agent>>) {
        
        let mut pending = vec![];
        for agent in agents {
            pending.append(&mut vec![vec![agent];self.game_per_agent]);
        }
        self.pending = pending
    }
}

//TODO: knockout AKA single elimination tournament