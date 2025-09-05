#[cfg(target_os = "linux")]
mod cgroup_manager_linux;

#[cfg(not(target_os = "linux"))]
mod cgroup_manager_stub;

use std::{
    fs::File,
    process::{Child, Stdio},
};

use anyhow::Context;
#[cfg(target_os = "linux")]
pub use cgroup_manager_linux::*;

#[cfg(not(target_os = "linux"))]
pub use cgroup_manager_stub::*;

fn create_process(
    command: &str,
    args: &[String],
    allow_stderr: bool,
    log_file: &Option<File>,
) -> anyhow::Result<Child> {
    let mut cmd = std::process::Command::new(command);
    cmd.args(args).stdin(Stdio::null());

    match (log_file, allow_stderr) {
        (Some(file), false) => {
            let stdout = file.try_clone().context("log file error")?;
            let stderr = file.try_clone().context("log file error")?;
            cmd.stdout(Stdio::from(stdout));
            cmd.stderr(Stdio::from(stderr));
        }
        (Some(file), true) => {
            let stdout = file.try_clone().context("log file error")?;
            cmd.stdout(Stdio::from(stdout));
            cmd.stderr(Stdio::inherit());
        }
        (None, false) => {
            cmd.stdout(Stdio::null());
            cmd.stderr(Stdio::null());
        }
        (None, true) => {
            cmd.stdout(Stdio::null());
            cmd.stderr(Stdio::inherit());
        }
    }

    cmd.spawn()
        .with_context(|| format!("command '{command}' not found"))
}
