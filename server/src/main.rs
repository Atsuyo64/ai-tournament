use std::time::{Duration, Instant};

use agent_interface;

fn get_current_user_id() -> Result<String, String> {
    let output = match std::process::Command::new("id").arg("-u").output() {
        Ok(output) => output,
        Err(_) => return Err("Could not launch command 'id'".to_string()),
    };

    let stdout = output.stdout;
    let untrimed_id = match std::str::from_utf8(&stdout) {
        Ok(str) => str,
        Err(_) => return Err("ID is not a valid string".to_string()),
    };
    Ok(untrimed_id.trim().to_string())
}

/// Test server
fn main() {
    #![allow(unreachable_code)]
    #![allow(unused)]

    let mut game: Box<dyn agent_interface::Game> = todo!();
    let num_players = game.get_game_info().num_player as usize;
    let mut agents: Vec<Box<dyn agent_interface::Agent>> = Vec::with_capacity(num_players);
    for _ in 0..num_players {
        let mut agent = todo!();
        agents.push(agent);
    }

    //TODO: different tournament depending of game info
    let mut player_index = 0;
    game.init();
    while !game.is_finished() {
        let state = game.get_state();
        let action =
            agents[player_index].select_action(state, Instant::now() + Duration::from_millis(100));
        if action.is_none() || game.apply_action(&action.unwrap()).is_err() {
            break;
        }
        player_index = (player_index + 1) % num_players;
    }
    println!("Looser is agent number {player_index}"); //FIXME: work only for 2 players games, with no score
}

fn get_cgroup_path(user_id: &str, group_name: &str) -> String {
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
fn create_cgroup(
    path: &str,
    max_memory: i64,
    max_pids: i64,
    cpus: &str,
) -> Result<cgroups_rs::Cgroup, String> {
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
        .map_err(|e| format!("Could not create cgroup: {e}").to_string())
}

#[cfg(test)]
mod test_rpc {
    use std::{io::Read, process::Stdio};

    use crate::{create_cgroup, get_cgroup_path, get_current_user_id};

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
        //NOTE: futur work: implement equivalent for Windows : "Job Object"
        assert_eq!(
            std::env::consts::OS,
            "linux",
            "Cgroups are only implemented on linux."
        );

        /*         let my_mount = cgroups_rs::hierarchies::mountinfo_self();
               println!("Mount info: {:?}", my_mount); // == [] ?? (does not parse cgroup2 elements)
        */

        /* match std::fs::File::open("/proc/self/mountinfo") {
            Ok(mut f) => {
                let mut s: String = String::new();
                f.read_to_string(&mut s).unwrap();
                println!("Content: {s}")
            }
            Err(e) => {
                println!("Erreur: {e}")
            }
        } */

        let my_hierarchy = cgroups_rs::hierarchies::auto();
        if my_hierarchy.v2() {
            println!("V2 Hierarchy");
        } else {
            println!("V1 Hierarchy /!\\ THIS CASE IS UNTESTED");
        }

        // println!("Hierarchy subsystems: {:?}", my_hierarchy.subsystems());

        let my_id = get_current_user_id().expect("Could not get user ID");

        println!("User id: {my_id}");

        let group_name = "my_cgroup";

        let new_group_path = get_cgroup_path(&my_id, &group_name);

        println!("Future new group path: {new_group_path}");

        //NOTE: name == path !
        let my_group =
            create_cgroup(&new_group_path, 1024 * 1024, 3, "1-3,5").expect("c'est non...");
        println!("path: {}", my_group.path());
        // my_group.apply(todo!()).expect("Failed to apply ressouce limit.");

        my_group.delete().expect("Could not delete cgroup")
    }
}
