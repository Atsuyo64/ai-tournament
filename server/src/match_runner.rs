use std::{collections::HashMap, fmt::Display, str::FromStr, sync::Arc, time::Duration};

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
use tracing::{instrument, trace, warn};

use crate::{agent::Agent, client_handler::ClientHandler, constraints::Constraints};

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
    pub results: Vec<(Arc<Agent>, f32)>,
    pub resources_freed: Constraints,
    // pub duration: Duration,
}

#[instrument(skip_all,fields(VS=settings.to_string()))]
pub fn run_match<G: Game>(settings: MatchSettings, mut game: G) -> MatchResult
where
    G::Action: FromStr,
    G::State: ToString,
{
    trace!("game started");
    let MatchSettings {
        ordered_player,
        resources,
    } = settings;
    // let num_players = ordered_player.len();
    let max_turn_duration = resources.action_time;
    const MAX_BUFFER_SIZE: usize = 4096;

    let mut clients: HashMap<usize, ClientHandler> = HashMap::new();
    // Start client processes
    {
        let num_cpus = resources.cpus_per_agent;
        let ram = resources.agent_ram;
        let mut avail_res = resources.clone();
        for (i, agent) in ordered_player.iter().enumerate() {
            match ClientHandler::init(agent.clone(), &avail_res.take(num_cpus, ram)) {
                Ok(client) => {
                    clients.insert(i, client);
                }
                Err(e) => {
                    warn!("Failed to start client for agent {}: {}", agent.name, e);
                }
            }
        }
    }

    // Init clocks (time budget)
    let mut time_budgets = vec![resources.time_budget; ordered_player.len()];

    game.init();

    while !game.is_finished() && clients.len() > 0 {
        let current = game.get_current_player_number();
        let time_budget = time_budgets[current];
        let chrono_start = std::time::Instant::now();

        // If player is missing, action is none
        let action = if let Some(client) = clients.get_mut(&current) {
            let state_str = game.get_state().to_string();
            let mut buf = [0; MAX_BUFFER_SIZE];
            let max_duration = Duration::min(max_turn_duration, time_budget);
            match client.send_and_recv(state_str.as_bytes(), &mut buf, max_duration) {
                Ok(received) => {
                    let response = std::str::from_utf8(&buf[..received]);
                    match response {
                        Ok(text) => match G::Action::from_str(text.trim()) {
                            Ok(action) => Some(action),
                            Err(_) => {
                                warn!(
                                    "Agent {} sent invalid action: {}",
                                    ordered_player[current].name, text
                                );
                                client.kill_child_process().unwrap();
                                clients.remove(&current);
                                None
                            }
                        },
                        Err(_) => {
                            warn!(
                                "Agent {} sent non-UTF8 response",
                                ordered_player[current].name
                            );
                            client.kill_child_process().unwrap();
                            clients.remove(&current);
                            None
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "Agent {} did not respond in time: {}",
                        ordered_player[current].name, e
                    );
                    client.kill_child_process().unwrap();
                    clients.remove(&current);
                    None
                }
            }
        } else {
            // Agent was already eliminated/killed/did not start
            None
        };

        let elapsed = chrono_start.elapsed();
        time_budgets[current] -= elapsed;

        // Apply action (even if it's None, Game is suppposed to handle elimination logic)
        if let Err(_) = game.apply_action(&action) {
            if action.is_some() { //CHECK: print anyway ?
                warn!("player {} 's action rejected by Game", current);
            }
        }
    }

    // Kill remaining processes
    for client in clients.values_mut() {
        client.kill_child_process().unwrap();
    }

    // Collect final scores
    let mut results = vec![];
    for (i, agent) in ordered_player.iter().enumerate() {
        let score = game.get_player_score(i as u32);
        results.push((agent.clone(), score));
    }

    trace!("match end");
    MatchResult {
        results,
        resources_freed: resources,
    }
}
