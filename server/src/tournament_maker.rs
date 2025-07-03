use std::{
    fmt::Display,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    usize,
};

use agent_interface::game_info::GameInfo;

use crate::{agent::Agent, constraints::Constraints};

#[derive(Debug, Clone)]
pub struct Score;

#[derive(Debug, Clone)]
struct Scores;

impl Scores {
    fn add_score(&mut self, _score: Score) {
        todo!()
    }
}

/* https://en.wikipedia.org/wiki/Tournament */
pub struct TournamentMaker {
    agents: Vec<Arc<Agent>>,
    resources: Constraints,
    game_info: GameInfo,
    resource_receiver: Receiver<Constraints>,
    resource_sender: Sender<Constraints>,
    score_receiver: Receiver<Score>,
    score_sender: Sender<Score>,
    prev_confrontations: Vec<Vec<Arc<Agent>>>,
    sched_confrontations: Vec<Vec<Arc<Agent>>>,
    scores: Scores,
    remaining_scheduled_matches: usize,
    remaining_swiss_rounds: usize, /* = ceil(log2(num_players)) ? */
    knockout_stage: usize, /* 3 ? (quarter -> semi -> final) */  /* double elimination tournament ? https://en.wikipedia.org/wiki/Double-elimination_tournament */
}

impl TournamentMaker {
    pub fn new(agents: Vec<Arc<Agent>>, resources: Constraints, game_info: GameInfo) -> Self {
        //TODO: check 'resources VS game_info' to avoid blocking wait
        let (resource_sender, resource_receiver) = mpsc::channel();
        let (score_sender, score_receiver) = mpsc::channel();
        Self {
            agents,
            resources,
            game_info,
            resource_receiver,
            resource_sender,
            score_receiver,
            score_sender,
            prev_confrontations: vec![],
            sched_confrontations: vec![],
            scores: Scores,
            remaining_scheduled_matches: 0,
            remaining_swiss_rounds: 0,//FIXME:
            knockout_stage: 0,
        }
    }

    /// Returns ordered leaderboard (todo)
    pub fn get_final_scores(&self) -> Vec<(String, f32)> {
        todo!()
    }

    fn required_match_ressources(&self) -> (usize, usize) {
        let cpus = self.resources.cpus_per_agent * self.game_info.num_player as usize;
        let ram = self.resources.agent_ram.unwrap_or(0) * self.game_info.num_player as usize;
        (cpus, ram)
    }

    fn all_round_scheduled(&self) -> bool {
        self.remaining_swiss_rounds == 0 && self.knockout_stage == 0
    }

    fn get_next_match(&mut self) -> Option<Vec<Arc<Agent>>> {
        if self.sched_confrontations.is_empty() {
            if self.all_round_scheduled() {
                while self.remaining_scheduled_matches != 0 {
                    let score = self.score_receiver.recv().expect("score deadlock???");
                    self.scores.add_score(score);
                    self.remaining_scheduled_matches -= 1;
                }
                return None;
            }
            while !self.update_schedule() {
                let score = self.score_receiver.recv().expect("score deadlock???");
                self.scores.add_score(score);
                self.remaining_scheduled_matches -= 1;
            }
            /* self.sched_confrontions should now be filled */
        }

        let confrontation = self.sched_confrontations.pop().unwrap();
        self.prev_confrontations.push(confrontation.clone());
        Some(confrontation)
    }

    fn update_schedule(&mut self) -> bool {
        self.remaining_scheduled_matches += 1; // !!!!!!
        todo!()
    }

    fn take_resources(&mut self) -> Constraints {
        let (req_cpus, req_ram) = self.required_match_ressources();
        while self.resources.cpus.len() < req_cpus || self.resources.total_ram < req_ram {
            let res = self
                .resource_receiver
                .recv()
                .expect("resources deadlock (should not be possible)");
            self.resources.add(res);
        }
        self.resources.take(req_cpus, req_ram)
    }
}

impl Iterator for TournamentMaker {
    type Item = MatchSettings;

    fn next(&mut self) -> Option<Self::Item> {
        self.get_next_match().map(|ordered_player| {
            let resources = self.take_resources();
            MatchSettings {
                ordered_player,
                resources,
                on_resource_free: self.resource_sender.clone(),
                on_final_score: self.score_sender.clone(),
            }
        })
    }
}

#[derive(Debug)]
pub struct MatchSettings {
    pub ordered_player: Vec<Arc<Agent>>,
    pub resources: Constraints,
    pub on_resource_free: Sender<Constraints>,
    pub on_final_score: Sender<Score>,
}

impl Display for MatchSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self
            .ordered_player
            .iter()
            .fold(String::new(), |acu, agent| {
                if acu.is_empty() {
                    acu + &agent.name
                } else {
                    acu + " VS " + &agent.name
                }
            });
        write!(f, "[{s}]")
    }
}
