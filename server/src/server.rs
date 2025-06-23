#![allow(unused)]

use crate::{agent::Agent, client_handler};

use agent_interface::{Game, GameFactory};
use anyhow::{anyhow, Context};
use std::{
    collections::{HashMap, HashSet}, fs::{self, DirEntry}, os::unix::process::CommandExt, path::PathBuf, slice::SplitMut, str::FromStr
};

use crate::agent;

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
    // Parameter in the future ?
    const BIN_NAME: &'static str = "eval";

    pub fn new(factory: F, params: SystemParams) -> Evaluator<G, F> {
        Evaluator {
            factory,
            params,
            _ff: std::marker::PhantomData::default(),
        }
    }

    pub fn evaluate(&self, directory: &std::path::Path) -> anyhow::Result<HashMap<String, f32>> {
        // 1. get agents name & code in *directory*
        // 2. try to compile each one of them
        // 3. create an tournament of some sort (depending of game_type) for remaining ones
        // 4. run tournament

        if (!directory.is_dir()) {
            return Err(anyhow!("{directory:?} is not a directory"));
        }
        self.compile_agents(directory);

        Ok(HashMap::new())
        // todo!()
    }

    fn compile_agents(&self, directory: &std::path::Path) -> Vec<Agent> {
        let mut vec: Vec<Agent> = Vec::new();
        const RED: &'static str = "\x1b[31m";
        const GREEN: &'static str = "\x1b[32m";
        const RESET: &'static str = "\x1b[0m";

        let longest_name = std::fs::read_dir(directory)
            .unwrap()
            .into_iter()
            .filter_map(|res| res.ok())
            .fold(0, |acu, entry| acu.max(entry.file_name().len()))
            + 3; //at least 3 dots

        println!("Compiling agents...");

        for subdir in std::fs::read_dir(directory).unwrap() {
            let Ok(subdir) = subdir else {
                continue;
            };
            let name = subdir.file_name().into_string().unwrap();

            print!("Compiling {name:.<longest_name$} ");

            if subdir.metadata().unwrap().is_file() {
                println!("{RED}Not a directory{RESET}");
                continue;
            }

            let res = self.compile_agent(&subdir);
            if res.is_ok() {
                println!("{GREEN}Ok{RESET}");
                vec.push(Agent::new(name, Some(res.unwrap())));
            } else {
                println!("{RED}{}{RESET}", res.unwrap_err());
                vec.push(Agent::new(name, None));
            }
        }
        vec
    }

    fn compile_agent(&self, dir: &DirEntry) -> Result<PathBuf, String> {
        //TODO: check crates used ? (list "abnormal" crates)
        //TODO: --offline to prevent using other crates than expected ?
        let args = vec![
            "build",
            "--release",
            "--bin",
            Self::BIN_NAME,
            "--message-format",
            "short",
        ];
        let proc = std::process::Command::new("cargo")
            .args(args)
            .current_dir(dir.path().canonicalize().unwrap())
            //.env_clear() // maybe ?
            .stderr(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn().unwrap();

        let ouput = proc.wait_with_output().expect("failed to wait on child");
        if ouput.status.success() {
            let path = dir.path().join(format!("target/release/{}",Self::BIN_NAME));
            Ok(path)
        } else {
            Err(format!("Compilation error ({}): {}",ouput.status.code().unwrap(),std::str::from_utf8(&ouput.stderr).unwrap().trim()))
        }
    }
}
