use crate::confrontation::Confrontation;
use crate::{agent::Agent, available_resources::AvailableRessources};
use crate::agent_compiler;

use agent_interface::{Game, GameFactory};
use anyhow::anyhow;
use std::{collections::HashMap, str::FromStr};

#[derive(Default,Clone,Copy)]
pub enum MaxMemory {
    /// Auto = max physical memory minus 1GB
    #[default]
    Auto,
    MaxMegaBytes(u16),
    MaxGigaBytes(u16),
}

/// CPUs used for evaluation. Each CPU can execute only one confrontation simultaneously
#[derive(Default,Clone,Copy)]
pub enum AvailableCPUs {
    /// Auto = all physical cpus
    #[default]
    Auto,
    /// Limited = any cpus, but not more than specified
    Limited(u32),
}

// impl AvailableCPUs {
//     /// create AvailableCPUs from string using unix-like format (eg. "1,2,4,6", "3-7,10-11,13", ...)
//     ///
//     /// returns Auto if the string is empty
//     ///
//     /// # Errors
//     ///
//     /// This function will return an error if the given string is ill-formed
//     pub fn from_string(cpus: &str) -> anyhow::Result<AvailableCPUs> {
//         if cpus.is_empty() {
//             return Ok(AvailableCPUs::Auto);
//         }
//         let mut set: HashSet<u16> = HashSet::new();
//         for item in cpus.split(',') {
//             let mut split = item.split('-');
//             let cnt = split.by_ref().count();
//             if cnt == 1 {
//                 let value: &str = split.nth(0).unwrap();
//                 let value: u16 = value
//                     .parse()
//                     .with_context(|| format!("could not parse {value}"))?;
//                 set.insert(value);
//             } else if cnt == 2 {
//                 let start: &str = split.nth(0).unwrap();
//                 let start: u16 = start
//                     .parse()
//                     .with_context(|| format!("could not parse {start}"))?;
//                 let end: &str = split.nth(0).unwrap();
//                 let end: u16 = end
//                     .parse()
//                     .with_context(|| format!("could not parse {end}"))?;
//                 let range = if start <= end {
//                     start..=end
//                 } else {
//                     end..=start
//                 };
//                 for i in range {
//                     set.insert(i);
//                 }
//             } else {
//                 return Err(anyhow!(
//                     "each comma-separated item must be a number or a range ('a-b'), got '{item}'"
//                 ));
//             }
//         }
//         Ok(AvailableCPUs::Defined(set))
//     }
// }

#[derive(Default,Clone,Copy)]
pub struct SystemParams {
    max_memory: MaxMemory,
    cpus: AvailableCPUs,
}

impl SystemParams {
    pub fn new(max_memory: MaxMemory, cpus: AvailableCPUs) -> Self {
        Self { max_memory, cpus }
    }
    
    pub fn max_memory(&self) -> &MaxMemory {
        &self.max_memory
    }
    
    pub fn cpus(&self) -> &AvailableCPUs {
        &self.cpus
    }
}

pub struct Evaluator<G, F>
where
    G: Game,
    F: GameFactory<G>,
    G::State: FromStr + ToString,
    G::Action: FromStr + ToString,
{
    factory: F,
    // game: Box<dyn agent_interface::Game<String, String> + 'static>, //'a instead of static ? ('static <=> not a ref)
    params: SystemParams,
    _ff: std::marker::PhantomData<G>,
}

impl<G: Game, F: GameFactory<G>> Evaluator<G, F>
where
    G::State: FromStr + ToString,
    G::Action: FromStr + ToString,
{
    pub fn new(factory: F, params: SystemParams) -> Evaluator<G, F> {
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
        let num_remaining = agents
            .iter()
            .fold(0u32, |acu, agent| if agent.compile { acu + 1 } else { acu });

        let game_info = self.factory.new_game().get_game_info();
        // 3. create an tournament of some sort (depending of game_type) for remaining ones
        let tournament = Self::wip_tournament_maker(num_remaining, &game_info);

        let mut _available_resources = AvailableRessources::from(self.params);

        //FIXME: that is a lot of clones
        let filterd_agents: Vec<Agent> = agents
            .clone()
            .into_iter()
            .filter(|agent| agent.compile)
            .collect();

        // 4. run tournament
        //TODO: parrallel for
        for confrontation in tournament {
            let mut contestants = Vec::new();
            for index in confrontation.ordered_player_indexes {
                contestants.push(filterd_agents[index as usize].clone());
            }
            // let _ = run_match(&contestants,)
        }

        Ok(HashMap::new())
    }

    fn wip_tournament_maker(
        num_agents: u32,
        game_info: &agent_interface::game_info::GameInfo,
    ) -> Vec<Confrontation> {
        if game_info.num_player == 1 {
            (0..num_agents)
                .map(|i| Confrontation {
                    ordered_player_indexes: vec![i],
                })
                .collect()
        } else if game_info.num_player == 2 {
            //FIXME: O(n²)
            let mut matches = Vec::new();
            for i in 0..num_agents {
                for j in 0..num_agents {
                    if i != j {
                        matches.push(Confrontation {
                            ordered_player_indexes: vec![i, j],
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
