#![allow(unused)]

use crate::client_handler;

use agent_interface::{Game, GameFactory};
use anyhow::{anyhow, Context};
use std::{
    collections::{HashMap, HashSet},
    slice::SplitMut,
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
    pub fn from_string(cpus: &str) -> anyhow::Result<AvailableCPUs> {
        if cpus.is_empty() {
            return Ok(AvailableCPUs::Auto);
        }
        let mut set: HashSet<u16> = HashSet::new();
        for item in cpus.split(',') {
            let mut split = item.split('-');
            let cnt = split.by_ref().count();
            if cnt == 1 {
                let value: &str = split.nth(0).unwrap();
                let value: u16 = value
                    .parse()
                    .with_context(|| format!("could not parse {value}"))?;
                set.insert(value);
            } else if cnt == 2 {
                let start: &str = split.nth(0).unwrap();
                let start: u16 = start
                    .parse()
                    .with_context(|| format!("could not parse {start}"))?;
                let end: &str = split.nth(0).unwrap();
                let end: u16 = end
                    .parse()
                    .with_context(|| format!("could not parse {end}"))?;
                let range = if start <= end {
                    start..=end
                } else {
                    end..=start
                };
                for i in range {
                    set.insert(i);
                }
            } else {
                return Err(anyhow!(
                    "each comma-separated item must be a number or a range ('a-b'), got '{item}'"
                ));
            }
        }
        Ok(AvailableCPUs::Defined(set))
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
