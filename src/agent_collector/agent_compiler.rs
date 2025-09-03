// use crate::agent::Agent;

use std::path::{Path, PathBuf};

use tracing::{error, instrument};

#[instrument(parent = None)]
pub fn compile_single_agent(dir: &Path) -> (anyhow::Result<PathBuf>, String) {
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
        .current_dir(dir.canonicalize().unwrap())
        .stderr(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("could not launch command 'cargo'");

    let output = proc
        .wait_with_output()
        .expect("failed to wait for end of compilation");
    let result = std::str::from_utf8(&output.stdout).unwrap().to_owned()
        + "\n"
        + std::str::from_utf8(&output.stderr).unwrap();
    if output.status.success() {
        let path = dir.join("target/release/").join(BIN_NAME);
        //FIXME: on Windows: BIN_NAME.join(".exe") or something link that
        (Ok(path), result)
    } else {
        let output = &output.stderr;
        let output = std::str::from_utf8(output).unwrap().trim();
        error!("compilation error: {output}");

        (
            Err(anyhow::anyhow!(
                "Compilation error: {}",
                output.trim().split("\n").next().unwrap_or_default(),
            )),
            result,
        )
    }
}
