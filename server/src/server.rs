use crate::{agent::Agent, available_resources::AvailableRessources};
use crate::confrontation::Confrontation;

use agent_interface::{Game, GameFactory};
use anyhow::anyhow;
use std::{
    collections::HashMap,
    fs::DirEntry,
    path::PathBuf,
    str::FromStr,
};
use sysinfo;

#[derive(Default)]
pub enum MaxMemory {
    /// Auto = max physical memory minus 1GB
    #[default]
    Auto,
    MaxMegaBytes(u16),
    MaxGigaBytes(u16),
}

/// CPUs used for evaluation. Each CPU can execute only one confrontation simultaneously
#[derive(Default)]
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

#[derive(Default)]
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
            _ff: std::marker::PhantomData,
        }
    }

    pub fn evaluate(&self, directory: &std::path::Path) -> anyhow::Result<HashMap<String, f32>> {
        // 1. get agents name & code in *directory*
        // 2. try to compile each one of them
        // 3. create an tournament of some sort (depending of game_type) for remaining ones
        // 4. run tournament

        if !directory.is_dir() {
            return Err(anyhow!("{directory:?} is not a directory"));
        }
        let agents = self.compile_agents(directory);
        let num_remaining = agents
            .iter()
            .fold(0u32, |acu, agent| if agent.compile { acu + 1 } else { acu });
        let game_info = self.factory.new_game().get_game_info();

        let tournament = Self::wip_tournament_maker(num_remaining, &game_info);

        let mut available_resources = self.compute_available_resources();

        //TODO: parrallel for
        for confrontation in tournament {

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
        } else {
            todo!()
        }
    }

    fn compute_available_resources(&self) -> AvailableRessources {
        let mut sys = sysinfo::System::new();
        
        let available_cpus = match self.params.cpus {
            AvailableCPUs::Auto => {
                sys.refresh_cpu_all();
                sys.cpus().len() as i32
            },
            AvailableCPUs::Limited(limit) => {
                limit as i32
            },
        };
        
        let available_megabytes = match self.params.max_memory {
            MaxMemory::Auto => {
                sys.refresh_memory();
                //Auto => use all memory except for 1GB //REVIEW: use 90% ?
                (sys.available_memory() / 1_000_000 ) as i32 - 1_000
            },
            MaxMemory::MaxMegaBytes(max) => max as i32,
            MaxMemory::MaxGigaBytes(max) => (max * 1_000) as i32,
        };

        assert!(available_cpus > 0,"Not enough CPUs to process");
        assert!(available_megabytes > 0,"Not enough memory to process");

        AvailableRessources { available_cpus, available_megabytes }
    }

    fn compile_agents(&self, directory: &std::path::Path) -> Vec<Agent> {
        let mut vec: Vec<Agent> = Vec::new();
        const RED: &str = "\x1b[31m";
        const GREEN: &str = "\x1b[32m";
        const RESET: &str = "\x1b[0m";

        let longest_name = std::fs::read_dir(directory)
            .unwrap()
            .filter_map(|res| res.ok())
            .fold(0, |acu, entry| acu.max(entry.file_name().len()))
            + 3; //at least 3 dots

        println!("Compiling agents...");

        for subdir in std::fs::read_dir(directory).unwrap() {
            let Ok(subdir) = subdir else {
                continue;
            };
            let name = subdir.file_name().into_string().unwrap();

            print!("Compiling {name:·<longest_name$} ");

            if subdir.metadata().unwrap().is_file() {
                println!("{RED}Not a directory{RESET}");
                continue;
            }

            let res = self.compile_agent(&subdir);
            if let Ok(res) = res {
                println!("{GREEN}Ok{RESET}");
                vec.push(Agent::new(name, Some(res)));
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
            .stderr(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("could not launch command 'cargo'");

        let ouput = proc.wait_with_output().expect("failed to wait on child");
        if ouput.status.success() {
            let path = dir.path().join("target/release/{}").join(Self::BIN_NAME);
            Ok(path)
        } else {
            Err(format!(
                "Compilation error: {}",
                // ouput.status.code().unwrap(),
                std::str::from_utf8(&ouput.stderr)
                    .unwrap()
                    .trim()
                    .split("\n")
                    .nth(0)
                    .unwrap_or_default(),
                // std::str::from_utf8(&ouput.stdout).unwrap().trim(),
            ))
        }
    }
}
