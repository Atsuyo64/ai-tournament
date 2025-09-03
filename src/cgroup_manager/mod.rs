#[cfg(target_os = "linux")]
mod cgroup_manager_linux;

#[cfg(target_os = "linux")]
pub use cgroup_manager_linux::*;

#[cfg(not(target_os = "linux"))]
mod cgroup_manager_stub;

use std::process::{Child, Stdio};

use anyhow::Context;
#[cfg(not(target_os = "linux"))]
pub use cgroup_manager_stub::*;

pub(self) fn create_process(
    command: &str,
    args: &[String],
    allow_stderr: bool,
) -> anyhow::Result<Child> {
    let mut cmd = std::process::Command::new(command);
    cmd.args(args).stdin(Stdio::null()).stdout(Stdio::null());
    if !allow_stderr {
        cmd.stderr(Stdio::null());
    }
    cmd.spawn()
        .with_context(|| format!("command '{command}' not found"))
}
