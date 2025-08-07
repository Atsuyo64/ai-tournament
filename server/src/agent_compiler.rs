use std::{fs::DirEntry, io::Write, path::PathBuf, sync::Arc};

use crate::agent::Agent;

pub fn compile_all_agents(directory: &std::path::Path, verbose: bool) -> Vec<Arc<Agent>> {
    let mut vec: Vec<Arc<Agent>> = Vec::new();
    const RED: &str = "\x1b[31m";
    const GREEN: &str = "\x1b[32m";
    const RESET: &str = "\x1b[0m";

    let longest_name = std::fs::read_dir(directory)
        .unwrap()
        .filter_map(|res| res.ok())
        .fold(0, |acu, entry| acu.max(entry.file_name().len()))
        + 3; // at least 3 dots

    if verbose {
        println!("Compiling agents...");
    }
    let mut ids = 1;
    for subdir in std::fs::read_dir(directory).unwrap() {
        let Ok(subdir) = subdir else {
            continue;
        };
        let name = subdir.file_name().into_string().unwrap();

        if verbose {
            print!("Compiling {name:·<longest_name$} ");
            let _ = std::io::stdout().flush(); // try to flush stdout
        }

        if subdir.metadata().unwrap().is_file() {
            if verbose {
                println!("{RED}Not a directory{RESET}");
            }
            continue;
        }

        let res = compile_single_agent(&subdir);
        if let Ok(res) = res {
            if verbose {
                println!("{GREEN}Ok{RESET}");
            }
            vec.push(Arc::new(Agent::new(name, Some(res), ids)));
        } else {
            if verbose {
                println!("{RED}{}{RESET}", res.unwrap_err());
            }
            vec.push(Arc::new(Agent::new(name, None, ids)));
        }
        ids += 1;
    }
    vec
}

pub fn compile_single_agent(dir: &DirEntry) -> Result<PathBuf, String> {
    const BIN_NAME: &str = "eval";
    //TODO: check crates used ? (list "abnormal" crates)
    //TODO: --offline to prevent using other crates than expected ?
    let args = vec![
        "build",
        "--release",
        "--bin",
        BIN_NAME,
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

    let output = proc.wait_with_output().expect("failed to wait on child");
    if output.status.success() {
        let path = dir.path().join("target/release/").join(BIN_NAME);
        Ok(path)
    } else {
        Err(format!(
            "Compilation error: {}",
            // output.status.code().unwrap(),
            std::str::from_utf8(&output.stderr)
                .unwrap()
                .trim()
                .split("\n")
                .next()
                .unwrap_or_default(),
            // std::str::from_utf8(&output.stdout).unwrap().trim(),
        ))
    }
}
