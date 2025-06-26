use std::{io::Read, process::Stdio, time::Duration};

use cgroup_manager::*;

// use {create_cgroup, get_cgroup_path, get_current_user_id, wait_for_process_cleanup};

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
