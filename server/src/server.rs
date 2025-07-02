use crate::agent_compiler;
use crate::confrontation::Confrontation;
use crate::constraints::Constraints;
use crate::match_runner::run_match;
use crate::agent::Agent;

use agent_interface::{Game, GameFactory};
use anyhow::anyhow;
use std::sync::Arc;
use std::{collections::HashMap, str::FromStr};

pub struct Evaluator<G, F>
where
    G: Game,
    F: GameFactory<G>,
    G::State: FromStr + ToString,
    G::Action: FromStr + ToString,
{
    factory: F,
    params: Constraints,
    _ff: std::marker::PhantomData<G>,
}

impl<G: Game, F: GameFactory<G>> Evaluator<G, F>
where
    G::State: FromStr + ToString,
    G::Action: FromStr + ToString,
    G : 'static + Send
{
    pub fn new(factory: F, params: Constraints) -> Evaluator<G, F> {
        Evaluator {
            factory,
            params,
            _ff: std::marker::PhantomData,
        }
    }

    pub fn evaluate(&self, directory: &std::path::Path) -> anyhow::Result<HashMap<String, f32>> {
        // 1. get agents name & code in *directory*
        if !directory.is_dir() {
            return Err(anyhow!("{directory:?} is not a directory"));
        }

        // 2. try to compile each one of them
        let agents = agent_compiler::compile_all_agents(directory);

        let game_info = self.factory.new_game().get_game_info();
        // 3. create an tournament of some sort (depending of game_type) for remaining ones
        let tournament = Self::wip_tournament_maker(&agents, &game_info);

        // 4. run tournament
        let confrontations = tournament.into_iter().map(|confrontation| {
            let game = self.factory.new_game();
            let cons = self.params.clone();
            std::thread::spawn(move || run_match(&confrontation, game, cons))
        }).collect::<Vec<_>>();

        for confrontation in confrontations {
            confrontation.join().unwrap();
        }

        Ok(HashMap::new())
    }

    fn wip_tournament_maker(
        agents: &Vec<Arc<Agent>>,
        game_info: &agent_interface::game_info::GameInfo,
    ) -> Vec<Confrontation> {
        //NOTE: unlike humans, bots can participate in several confrontations concurrently! (potentially more exotic tournaments available)
        //NOTE: when this will be an iterator, should create channel to 'wake him up' in case of event (end of match + killed agent (=> more resources available))

        if game_info.num_player == 1 {
            agents
                .iter()
                .filter_map(|agent| {
                    if agent.compile {
                        Some(Confrontation {
                            ordered_player: vec![agent.clone()],
                        })
                    } else {
                        None
                    }
                })
                .collect()
        } else if game_info.num_player == 2 {
            //FIXME: O(n²)
            let mut matches = Vec::new();
            for (i,a) in agents.iter().enumerate() {
                for (j,b) in agents.iter().enumerate() {
                    if i != j && a.compile && b.compile {
                        matches.push(Confrontation {
                            ordered_player: vec![a.clone(), b.clone()],
                        });
                    }
                }
            }
            matches
        } else {
            todo!()
        }
    }
}
