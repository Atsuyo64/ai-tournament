use crate::client_handler;

use agent_interface::{Game, GameFactory};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

pub enum MaxMemory {
    Auto,
    MaxMegaBytes(u16),
    MaxGigaBytes(u16),
}

pub enum AvailableCPUs {
    Auto,
    Defined(HashSet<u16>),
}

impl AvailableCPUs {
    pub fn from_string(cpus: &str) -> AvailableCPUs {
        Self::Defined(todo!())
    }
}

pub struct SystemParams {
    max_memory: MaxMemory,
    cpus: AvailableCPUs,
}

impl SystemParams {
    pub fn new(max_memory: MaxMemory, cpus: AvailableCPUs) -> Self {
        Self { max_memory, cpus }
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
            _ff: std::marker::PhantomData::default(),
        }
    }

    pub fn evaluate(_directory: &std::path::Path) -> HashMap<String, f32> {
        // 1. get agents name & code in *directory*
        // 2. try to compile each one of them
        // 3. create an tournament of some sort (depending of game_type) for remaining ones
        // 4. run tournament
        todo!()
    }
}
