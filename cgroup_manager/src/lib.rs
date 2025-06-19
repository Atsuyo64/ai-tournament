use std::{
    process::{Child, Stdio},
    time::{Duration, Instant},
};

use anyhow::{self, Context};
use cgroups_rs::{Cgroup, cgroup};

pub fn get_current_user_id() -> anyhow::Result<String> {
    let output = std::process::Command::new("id")
        .arg("-u")
        .output()
        .context("Could not launch 'id -u'")?;
    let stdout = output.stdout;
    let untrimed_id = std::str::from_utf8(&stdout).context("id is not a valid string")?;
    Ok(untrimed_id.trim().to_string())
}

pub fn get_cgroup_path(user_id: &str, group_name: &str) -> String {
    format!("user.slice/user-{user_id}.slice/user@{user_id}.service/{group_name}")
}

/// Create a cgroup at `path`.
///
/// The cgroup will have the provided limitations.
///
/// * `max_memory` - Maximum available memory in Bytes. Non-positive means no restriction.
/// * `max_pids` - Maximum number of PIDS inside the cgroup at any time. Non-positive means no restriction.
/// * `cpus` - which cpus the members can run one. Uses comma separated cpu ranges ("1-5,7", "1,3,4", ...). Empty string means no restriction.
///
/// # Errors
///
/// This function will return an error if the cgroup could not be created. This can happen if the parameters are incorrect or if cgroup is not available.
pub fn create_cgroup(
    path: &str,
    max_memory: i64,
    max_pids: i64,
    cpus: &str,
) -> anyhow::Result<cgroups_rs::Cgroup> {
    let mut builder = cgroups_rs::cgroup_builder::CgroupBuilder::new(path);
    if max_memory > 0 {
        builder = builder.memory().memory_hard_limit(max_memory).done();
    }
    if max_pids > 0 {
        builder = builder
            .pid()
            .maximum_number_of_processes(cgroups_rs::MaxValue::Value(max_pids))
            .done();
    }
    if !cpus.is_empty() {
        builder = builder.cpu().cpus(cpus.to_string()).done();
    }
    builder
        .build(cgroups_rs::hierarchies::auto())
        .context("could not create cgroup")
}

#[derive(Debug)]
pub struct TimeoutError {}

impl std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Timout Error")
    }
}

impl std::error::Error for TimeoutError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

pub fn wait_for_process_cleanup(
    cgroup: &cgroups_rs::Cgroup,
    pid: u64,
    max_duration: Duration,
) -> Result<(), TimeoutError> {
    let deadline = Instant::now() + max_duration;
    while cgroup.tasks().iter().any(|cpid| cpid.pid == pid) {
        if Instant::now() > deadline {
            return Err(TimeoutError {});
        }

        std::thread::sleep(std::cmp::min(Duration::from_millis(1), max_duration / 10));
    }
    Ok(())
}

pub fn create_process_in_cgroup(
    command: &str,
    args: &Vec<&str>,
    group: &cgroups_rs::Cgroup,
) -> anyhow::Result<std::process::Child> {
    let mut child = std::process::Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("command '{command}' not found"))?;

    let pid = child.id() as u64;
    let addition = group.add_task_by_tgid(cgroups_rs::CgroupPid { pid });
    if addition.is_err() {
        let kill = child.kill();

        addition.with_context(|| {
            if kill.is_ok() {
                format!("could not add process to cgroup")
            } else {
                format!(
                    "could not add process to cgroup, and process could not be killed either ({})",
                    kill.unwrap_err()
                )
            }
        })?;
    }
    Ok(child)
}

pub struct LimitedProcess {
    pub child: Child,
    cgroup: Cgroup,
    cleaned_up: bool,
}

impl LimitedProcess {
    pub fn launch(
        command: &str,
        args: &Vec<&str>,
        max_memory: i64,
        cpus: &str,
    ) -> anyhow::Result<LimitedProcess> {
        static COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
        let user_id = get_current_user_id().context("could not get user id")?;
        //generate a new cgroup name for each Limited Process
        let group_name = COUNTER
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            .to_string();
        let path = get_cgroup_path(&user_id, &group_name);
        let group =
            create_cgroup(&path, max_memory, 10, cpus).context("could not create cgroup")?;
        let child = create_process_in_cgroup(command, args, &group).with_context(|| {
            let _ = group.delete();
            "could not create process in cgroup"
        })?;

        Ok(LimitedProcess {
            child,
            cgroup: group,
            cleaned_up: false,
        })
    }

    pub fn try_kill(&mut self, max_duration: Duration) -> anyhow::Result<()> {
        self.cgroup.kill().context("could not kill process")?;
        wait_for_process_cleanup(&self.cgroup, self.child.id() as u64, max_duration)
            .context("process cleanup timed out")?;
        self.cgroup.delete().context("could not cleanup cgroup")?;
        self.cleaned_up = true;
        Ok(())
    }
}

impl Drop for LimitedProcess {
    fn drop(&mut self) {
        static CLEANUP_DURATION: Duration = Duration::from_millis(10);
        if !self.cleaned_up {
            println!(
                "Process {} was not cleaned up before dropping. Trying to clean up for up to {:?}...",
                self.child.id(),
                CLEANUP_DURATION
            );
            let _ = self.try_kill(CLEANUP_DURATION);
        }
    }
}
