use std::{fmt::Display, str::FromStr, sync::Arc, time::Duration};

// use crate::{
//     client_handler::ClientHandler, tournament_maker::MatchSettings,
// };
// use agent_interface::Game;
// use anyhow::{anyhow, Context};
// use tracing::{error, info, instrument, trace, warn};

// #[instrument(skip_all,fields(VS=match_settings.to_string()))]
// pub fn run_match_old<G: Game>(match_settings: MatchSettings, mut game: G)
// where
//     G::Action: FromStr,
//     G::State: ToString,
// {
//     let MatchSettings {
//         ordered_player,
//         mut resources,
//         on_resource_free,
//         on_final_score,
//     } = match_settings;
//     assert_eq!(
//         resources.cpus.len(),
//         resources.cpus_per_agent * ordered_player.len()
//     );
//     info!("new match started");

//     let cpu_per_agent = resources.cpus_per_agent;
//     let ram_per_agent = resources.agent_ram;
//     let num_players = ordered_player.len();

//     let mut clients = Vec::with_capacity(num_players);
//     let mut agents_resources = Vec::with_capacity(num_players);

//     for i in 0..ordered_player.len() {
//         let res = resources.take(cpu_per_agent, ram_per_agent);
//         clients.push(ClientHandler::init(ordered_player[i].clone(), &res));
//         agents_resources.push(res);
//         if let Err(e) = &clients[i] {
//             warn!("Error creating client: {e} : {:?}", e.chain().nth(1));
//             on_resource_free.send(agents_resources[i].clone()).unwrap();
//         }
//     }

//     let mut current_player_number;
//     game.init();
//     while !game.is_finished() {
//         let state = game.get_state().to_string();
//         current_player_number = game.get_current_player_number();
//         trace!("player to play: {current_player_number}");

//         if clients[current_player_number].is_err() {
//             let _ = game.apply_action(&None);
//             continue;
//         }
//         let action = match try_get_action(clients[current_player_number].as_mut().unwrap(), state) {
//             Ok(a) => a,
//             Err(e) => {
//                 warn!(
//                     "no response from agent {:?} : {e}",
//                     ordered_player[current_player_number]
//                         .path_to_exe
//                         .as_ref()
//                         .unwrap()
//                 );
//                 let _ = game.apply_action(&None);
//                 let _ = clients[current_player_number]
//                     .as_mut()
//                     .unwrap()
//                     .kill_child()
//                     .map_err(|e| error!("could not kill client {e}"));
//                 clients[current_player_number] = Err(anyhow!("stopped"));
//                 on_resource_free
//                     .send(agents_resources[current_player_number].clone())
//                     .unwrap();
//                 continue;
//             }
//         };
//         if game.apply_action(&Some(action)).is_err() {
//             warn!(
//                 "invalid action from agent {:?}",
//                 ordered_player[current_player_number]
//                     .path_to_exe
//                     .as_ref()
//                     .unwrap()
//             );
//             let _ = clients[current_player_number]
//                 .as_mut()
//                 .unwrap()
//                 .kill_child()
//                 .map_err(|e| error!("could not kill client {e}"));
//             on_resource_free
//                 .send(agents_resources[current_player_number].clone())
//                 .unwrap();
//             clients[current_player_number] = Err(anyhow!("stopped"));
//         }
//     }
    
//     clients.iter_mut().enumerate().for_each(|(i,c)| {
//         if let Ok(c) = c {
//             c.kill_child().unwrap();
//             on_resource_free.send(agents_resources[i].clone()).unwrap();
//         }
//     });
    
//     let scores = (0..num_players as u32).map(|i| game.get_player_score(i)).collect::<Vec<_>>();
//     //TODO: update scores
//     //TODO: on_score_update
// }

// fn try_get_action<A>(client: &mut ClientHandler, state: String) -> anyhow::Result<A>
// where
//     A: FromStr,
// {
//     let mut buf = [0; 4096];
//     let n = client
//         .send_and_recv(state.as_bytes(), &mut buf, Duration::from_secs(1))
//         .context("message error")?;
//     let s = str::from_utf8(&buf[..n]).context("invalid utf8 response")?;
//     match A::from_str(s) {
//         Ok(a) => Ok(a),
//         Err(_) => Err(anyhow!("from_str")),
//     }
// }


use agent_interface::Game;

use crate::{agent::Agent, constraints::Constraints};

#[derive(Debug, Clone)]
pub struct MatchSettings {
    pub ordered_player: Vec<Arc<Agent>>,
    pub resources: Constraints,
}

impl Display for MatchSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self
            .ordered_player
            .iter()
            .fold(String::new(), |acu, agent| {
                if acu.is_empty() {
                    acu + &agent.name
                } else {
                    acu + " VS " + &agent.name
                }
            });
        write!(f, "[{s}]")
    }
}

#[derive(Debug, Clone)]
pub struct MatchResult {
    pub results:Vec<(Arc<Agent>,f32)>,
    pub resources: Constraints,
    // pub duration: Duration,
}

pub fn run_match<G:Game>(settings: MatchSettings,mut _game:G) -> MatchResult
where
    G::Action: FromStr,
    G::State: ToString,
{
    //FIXME: that is very much a placeholder. Please don't try this at home
    std::thread::sleep(Duration::from_millis(100));

    // Random winner for now
    MatchResult {
        results: vec![(settings.ordered_player[0].clone(),1.0),(settings.ordered_player[1].clone(),0.0),],
        resources: settings.resources,
    }
}
