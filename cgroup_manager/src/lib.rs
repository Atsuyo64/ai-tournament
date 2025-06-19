use std::{
    process::Stdio,
    time::{Duration, Instant},
};

use anyhow::{self, Context};

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

pub fn wait_for_process_cleanup(
    cgroup: &cgroups_rs::Cgroup,
    pid: u64,
    max_duration: Duration,
) -> Result<(), String> {
    let deadline = Instant::now() + max_duration;
    while cgroup.tasks().iter().any(|cpid| cpid.pid == pid) {
        if Instant::now() > deadline {
            return Err("process did not end before timeout".to_string());
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

#[cfg(test)]
mod test_rpc {
    use std::{io::Read, process::Stdio, time::Duration};

    use super::*;

    #[test]
    fn launch_something() {
        use std::process;

        let proc = process::Command::new("echo")
            .args(vec!["Hello", "World"])
            .stdout(Stdio::piped())
            .spawn()
            .expect("Could not spawn child");
        let mut res = proc.stdout.expect("No result ?");

        let mut buffer = String::new();
        let _length = res
            .read_to_string(&mut buffer)
            .expect("Could not make a string ?");

        println!("{buffer}");
    }

    #[test]
    fn test_create_cgroup() {
        //NOTE: futur work: implement the Windows equivalent: "Job Object"
        assert_eq!(
            std::env::consts::OS,
            "linux",
            "Cgroups are only implemented on linux."
        );

        let my_hierarchy = cgroups_rs::hierarchies::auto();
        if my_hierarchy.v2() {
            println!("V2 Hierarchy");
        } else {
            println!("V1 Hierarchy /!\\ THIS CASE IS UNTESTED");
        }

        let my_id = get_current_user_id().expect("Could not get user ID");

        println!("User id: {my_id}");

        let group_name = "my_cgroup";

        let new_group_path = get_cgroup_path(&my_id, &group_name);

        println!("Future new group path: {new_group_path}");

        let my_group = create_cgroup(&new_group_path, 1024 * 1024, 3, "1-3,5")
            .expect("Could not create cgroup...");
        println!("path: {}", my_group.path());
        // my_group.apply(todo!()).expect("Failed to apply ressouce limit.");

        my_group.delete().expect("Could not delete cgroup")
    }

    #[test]
    fn test_create_process_in_cgroup() {
        let id = get_current_user_id().unwrap();
        let path = get_cgroup_path(&id, "rust_group");
        let group = create_cgroup(&path, 1024 * 1024, 0, "").unwrap();
        println!("Cgroup created");
        let process = std::process::Command::new("sleep").arg("10").spawn();
        if let Ok(mut child) = process {
            let pid = child.id() as u64;
            println!("Process {pid} created");
            if let Err(e) = group.add_task_by_tgid(cgroups_rs::CgroupPid { pid }) {
                println!("Could not add task to cgroup: {e}");
            } else {
                println!("Task added to cgroup");
                println!("Waiting for response...");
                //sleep for ...ms and then try get result ?
                //BUT loss time if it finishes "early"
                println!("Finished waiting");
                let result = child.stdout.take(); //FIXME: release ?
                let is_late_or_incorrect = match result {
                    Some(_answer) => {
                        println!("The process responded on time and the response is acceptable");
                        false
                    } // !is_answer_ok(answer)
                    None => {
                        println!("Process is late !");
                        true
                    }
                };
                if is_late_or_incorrect {
                    println!("Attenpting to kill process");
                    //kill
                    group.kill().unwrap_or_else(|e| {
                        println!("Could not kill process. Must wait 10s to let it \"die by itself\", to avoid error in cgroup.delete(). Error: {e}");
                        std::thread::sleep(Duration::from_secs(10));
                    });
                    wait_for_process_cleanup(&group, pid, Duration::from_millis(100))
                        .unwrap_or_else(|e| println!("Process cleanup did not end well: {e}"));
                } else {
                    //release (auto ?)
                }
            }
        } else {
            let error = process.unwrap_err();
            println!("Process creation failed: {}", error);
        }
        println!("Deleting cgroup.");
        group.delete().unwrap_or_else(|e| {
            println!("Could not delete cgroup ! Is there any decendant left ? ({e})");
            let procs = group.tasks();
            println!("PIDS: {:?}", procs);
        });
    }
}
