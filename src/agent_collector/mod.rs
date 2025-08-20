use std::{
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::bail;
use tracing::{error, info, instrument, warn};

use crate::{
    agent::Agent, agent_collector::config_file_utils::check_dir_integrity,
    configuration::Configuration,
};

mod agent_compiler;

mod config_file_utils;

#[instrument(skip(config))]
pub fn collect_agents(
    directory: impl AsRef<Path> + std::fmt::Debug,
    config: Configuration,
) -> anyhow::Result<Vec<Arc<Agent>>> {
    let verbose = config.verbose;
    let compile = config.compile_agents;
    let self_test = config.self_test;
    let all_configs = config.test_all_configs;

    let directory = directory.as_ref();

    if !Path::is_dir(directory) {
        bail!("'{directory:?}' is not a valid directory");
    }

    let mut vec: Vec<Arc<Agent>> = Vec::new();
    const RED: &str = "\x1b[31m";
    const GREEN: &str = "\x1b[32m";
    const YELLOW: &str = "\x1b[33m";
    const RESET: &str = "\x1b[0m";

    let longest_name = std::fs::read_dir(directory)
        .unwrap()
        .filter_map(|res| res.ok())
        .fold(0, |acu, entry| acu.max(entry.file_name().len()))
        + 3; // at least 3 dots

    if verbose {
        if compile {
            println!("Compiling agents...");
        } else {
            println!("Collecting agents...");
        }
    }

    let mut ids = 1;
    let subdirs = if self_test {
        // hacky way of only checking cwd when self_test is set
        vec![std::env::current_dir().unwrap()]
    } else {
        std::fs::read_dir(directory)
            .unwrap()
            .filter_map(|item| {
                if let Ok(item) = item {
                    Some(item.path())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    };
    info!(agent_directories=?subdirs);

    for subdir in subdirs {
        let name = subdir
            .file_name()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();

        if verbose {
            if compile {
                print!("Compiling {name:·<longest_name$} ");
            } else {
                print!("Collecting {name:·<longest_name$} ");
            }
            let _ = std::io::stdout().flush(); // try to flush stdout
        }

        if subdir.metadata().unwrap().is_file() {
            warn!("Not a directory: '{name}'");
            if verbose {
                println!("{RED}Not a directory{RESET}");
            }
            continue;
        }

        let res = if compile {
            agent_compiler::compile_single_agent(&subdir)
        } else {
            collect_binary(&subdir)
        };

        let Ok(res) = res else {
            // compile => already logged
            if !compile {
                error!("agent collection failed: {}", res.as_ref().unwrap_err());
            }
            if verbose {
                println!("{RED}{}{RESET}", res.unwrap_err());
            }
            continue;
        };

        if all_configs {
            let configs = config_file_utils::get_all_configs(&subdir);
            let Ok(configs) = configs else {
                error!("Error getting config: {}", configs.as_ref().unwrap_err());
                if verbose {
                    println!("{RED}{}{RESET}", configs.unwrap_err());
                }
                continue;
            };

            for (config_name, config) in configs {
                let args = config_file_utils::get_args_from_config(&config);
                let Ok(args) = args else {
                    warn!(
                        "Config '{config_name}' error: {}",
                        args.as_ref().unwrap_err()
                    );
                    if verbose {
                        print!(
                            "{YELLOW}Config '{config_name}' error: {}, {RESET}",
                            args.unwrap_err()
                        );
                    }
                    continue;
                };
                vec.push(Arc::new(Agent::new(
                    name.clone() + "-" + &config_name,
                    Some(res.clone()),
                    ids,
                    Some(args),
                )));
                ids += 1;
            }
        } else {
            let config = config_file_utils::get_eval_config(&subdir);
            let Ok(config) = config else {
                error!("No 'eval' config: {}", config.as_ref().unwrap_err());
                if verbose {
                    println!("{RED}No 'eval' config: {}{RESET}", config.unwrap_err());
                }
                continue;
            };
            let args = config_file_utils::get_args_from_config(&config);
            let Ok(args) = args else {
                error!(
                    "Invalid config: '{config}' ({})",
                    args.as_ref().unwrap_err()
                );
                if verbose {
                    println!(
                        "{RED}Invalid config: '{config}' ({}){RESET}",
                        args.unwrap_err()
                    );
                }
                continue;
            };

            vec.push(Arc::new(Agent::new(name, Some(res), ids, Some(args))));
        }

        if verbose {
            println!("{GREEN}Ok{RESET}");
        }

        ids += 1;
    }

    Ok(vec)
}

#[instrument]
fn collect_binary(dir: &Path) -> anyhow::Result<PathBuf> {
    check_dir_integrity(dir)?;

    // Safety: `check_dir_integrity` should have checked that read_dir is ok
    let cnt = std::fs::read_dir(dir).unwrap().count();
    if cnt != 2 {
        bail!("directory contains {cnt} elements instead of 2");
    }
    for entry in std::fs::read_dir(dir).unwrap() {
        let Ok(entry) = entry else {
            bail!("one entry cannot be read in directory");
        };
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if !metadata.is_file() {
            bail!("{:?} is not a file", entry.file_name());
        }
        let Ok(name) = entry.file_name().into_string() else {
            bail!("name error: {:?}", entry.file_name());
        };
        if name.ends_with(".yml") || name.ends_with(".yaml") {
            continue;
        } else {
            return Ok(entry.path());
        }
    }
    bail!("binary not found")
}
