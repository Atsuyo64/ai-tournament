use crate::client_handler;

use agent_interface::{self, GameFactory};
use std::{
    collections::{HashMap, HashSet}, marker::PhantomData, str::FromStr
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

pub struct Evaluator<G, State, Action>
where
    State: FromStr + ToString,
    Action: FromStr + ToString,
    G: agent_interface::GameFactory<State ,Action> + Sized,
{
    factory: G ,//Box<dyn agent_interface::GameFactory<State, Action>>,
    // game: Box<dyn agent_interface::Game<String, String> + 'static>, //'a instead of static ? ('static <=> not a ref)
    params: SystemParams,
}

impl<State, Action> Evaluator<State, Action>
where
    State: FromStr + ToString,
    Action: FromStr + ToString,
{
    pub fn new(
        factory: Box<dyn agent_interface::GameFactory<State, Action>>,
        params: SystemParams,
    ) -> Evaluator<State, Action> {
        Evaluator { factory, params }
    }

    pub fn evaluate(_directory: &std::path::Path) -> HashMap<String, f32> {
        // 1. get agents name & code in *directory*
        // 2. try to compile each one of them
        // 3. create an tournament of some sort (depending of game_type) for remaining ones
        // 4. run tournament
        if self.factory.new().game_info.num_players() > 1 {}
        todo!()
    }
}