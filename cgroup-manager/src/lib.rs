use std::{
    process::{Child, Stdio},
    time::{Duration, Instant},
};

use anyhow::{self, Context};
use cgroups_rs::Cgroup;

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
        write!(f, "Timeout Error")
    }
}

impl std::error::Error for TimeoutError {}

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

        std::thread::sleep(std::cmp::min(Duration::from_millis(10), max_duration / 10));
    }
    Ok(())
}

fn create_process(command: &str, args: &[String], allow_stderr: bool) -> anyhow::Result<Child> {
    let mut cmd = std::process::Command::new(command);
    cmd.args(args).stdin(Stdio::null()).stdout(Stdio::null());
    if !allow_stderr {
        cmd.stderr(Stdio::null());
    }
    cmd.spawn()
        .with_context(|| format!("command '{command}' not found"))
}

pub fn create_process_in_cgroup(
    command: &str,
    args: &[String],
    group: &cgroups_rs::Cgroup,
    allow_stderr: bool,
) -> anyhow::Result<std::process::Child> {
    let mut child = create_process(command, args, allow_stderr)?;

    let pid = child.id() as u64;
    let addition = group.add_task_by_tgid(cgroups_rs::CgroupPid { pid });
    if addition.is_err() {
        let kill = child.kill();

        addition.with_context(|| {
            if let Err(err) = kill {
                format!(
                    "could not add process to cgroup, and process could not be killed either ({err})"
                )
            } else {
                "could not add process to cgroup".to_string()
            }
        })?;
    }
    Ok(child)
}

#[derive(Debug)]
pub struct LimitedProcess {
    pub child: Child,
    cgroup: Option<Cgroup>,
    cleaned_up: bool,
}

impl LimitedProcess {
    pub fn launch(
        command: &str,
        args: &[String],
        max_memory: i64,
        cpus: &str,
        allow_stderr: bool,
    ) -> anyhow::Result<LimitedProcess> {
        static COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1); // lazy cell ? (if multiple evaluations at the same time !)
        let user_id = get_current_user_id().context("could not get user id")?;
        // generate a new cgroup name for each Limited Process
        let group_name = "CGROUP_MANAGER_".to_owned()
            + &COUNTER
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                .to_string();
        let path = get_cgroup_path(&user_id, &group_name);
        let group =
            create_cgroup(&path, max_memory, 100, cpus).context("could not create cgroup")?;
        let child =
            create_process_in_cgroup(command, args, &group, allow_stderr).with_context(|| {
                let _ = group.delete();
                "could not create process in cgroup"
            })?;

        Ok(LimitedProcess {
            child,
            cgroup: Some(group),
            cleaned_up: false,
        })
    }

    pub fn try_kill(&mut self, max_duration: Duration) -> anyhow::Result<()> {
        match &mut self.cgroup {
            Some(cgroup) => {
                cgroup.kill().context("could not kill process")?;
                wait_for_process_cleanup(cgroup, self.child.id() as u64, max_duration)
                    .context("process cleanup timed out")?;
                // at this point, the process is killed. Even so the cgroup cleanup fail, it is
                // 'safe' (probably) to continue
                self.cleaned_up = true;
                if let Err(e) = cgroup.delete() {
                    // Oh well... Whatever...
                    tracing::warn!("Failed to remove cgroup. If this happens a lot, it may slow down the computer. {e}");
                }
                Ok(())
            }
            None => self.child.kill().context("could not kill process"),
        }
    }

    pub fn launch_without_container(
        command: &str,
        args: &[String],
        allow_stderr: bool,
    ) -> anyhow::Result<LimitedProcess> {
        let child =
            create_process(command, args, allow_stderr).context("could not create process")?;

        Ok(LimitedProcess {
            child,
            cgroup: None,
            cleaned_up: false,
        })
    }

    /// Will print out as much info as possible
    pub fn try_debug_cgroup(&mut self) {
        let pid = self.child.id();
        let mut p = String::new();
        p += "/sys/fs/cgroup/";
        p += self.cgroup.as_ref().unwrap().path();
        println!("Path: {p:?}");
        Self::exec(&format!("lsof +D {p}"));
        Self::exec(&format!("cat {p}/cgroup.procs"));
        Self::exec(&format!("cat {p}/cgroup.stat"));
        Self::exec(&format!("cat {p}/pids.current"));
        Self::exec(&format!("ps -Flww -p {pid}"));
        Self::exec(&format!("cat /proc/{pid}/status"));
        if let Err(e) = self.try_kill(Duration::from_millis(100)) {
            println!("failed to kill again: {e:#}");
        } else {
            println!("successfully killed this time ??");
        }
        Self::exec(&format!("rmdir {p}"));
    }

    fn exec(cmd: &str) {
        let mut iter = cmd.split(" ");
        let program = iter.next().unwrap();
        let args = iter.collect::<Vec<_>>();
        let output = std::process::Command::new(program)
            .args(&args)
            .output()
            .unwrap();
        println!(
            "$ {cmd}\n\x1b[31m{}\x1b[39m{}",
            std::str::from_utf8(&output.stderr).unwrap(),
            std::str::from_utf8(&output.stdout).unwrap()
        );
    }
}

impl Drop for LimitedProcess {
    fn drop(&mut self) {
        static CLEANUP_DURATION: Duration = Duration::from_millis(10);
        if !self.cleaned_up {
            // warn!(
            //     "Process {} was not cleaned up before dropping. Trying to clean up for up to {:?}...",
            //     self.child.id(),
            //     CLEANUP_DURATION
            // );
            self.try_kill(CLEANUP_DURATION)
                .expect("could not kill process/cgroup on LimitedProcess::drop");
        }
    }
}
