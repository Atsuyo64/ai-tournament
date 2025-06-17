use std::time::{Duration, Instant};

use agent_interface;

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

#[cfg(test)]
mod test_rpc {
    use std::{io::Read, process::Stdio};

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

        use cgroups_rs;
        use cgroups_rs::cgroup_builder::CgroupBuilder;

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

        let my_id: Vec<_> = std::process::Command::new("id")
            .arg("-u")
            .output()
            .expect("Could not launch 'id -u'")
            .stdout;
        let my_id =
            std::str::from_utf8(my_id.as_slice()).expect("ID vec<u8> could not be made into &str").trim();

        println!("User id: {my_id}");
        
        let group_name = "my_cgroup";

        let new_group_path = format!("user.slice/user-{my_id}.slice/user@{my_id}.service/{group_name}");

        println!("Future new group path: {new_group_path}");

        //NOTE: name == path !
        let my_group =
            CgroupBuilder::new(&new_group_path)
                .memory()
                .memory_hard_limit(1024 * 1024 * 16) //in bytes ? (to test)
                .done()
                .pid()
                //.maximum_number_of_processes(MaxValue::Value(1)) //FIXME: use cpu().cpus("0-1,4").done() instead ?
                .done()
                .build(my_hierarchy)
                .expect("Cgroug could not be created");
        println!("path: {}", my_group.path());
        // my_group.apply(todo!()).expect("Failed to apply ressouce limit.");

        my_group.delete().expect("Could not delete cgroup")
    }
}
