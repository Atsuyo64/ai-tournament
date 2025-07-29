use std::{collections::HashMap, fmt::Display, str::FromStr, sync::Arc, time::Duration};

use agent_interface::Game;
use tracing::{info, instrument, trace, warn};

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
            .map(|a| &a.name[..])
            .collect::<Vec<_>>()
            .join(" VS ");
        write!(f, "[{s}]")
    }
}

pub type MatchResult = Vec<(Arc<Agent>, f32)>;

#[derive(Debug, Clone)]
pub struct RunnerResult {
    pub results: MatchResult,
    pub resources_freed: Constraints,
    pub errors: String,
    // pub duration: Duration,
}

#[instrument(skip_all,fields(VS=settings.to_string()))]
pub fn run_match<G: Game>(settings: MatchSettings, mut game: G) -> RunnerResult
where
    G::Action: FromStr + ToString,
    G::State: ToString,
{
    trace!("game started");
    let MatchSettings {
        ordered_player,
        resources,
    } = settings;
    let mut errors_string = String::new();

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
                    errors_string += &format!("{} startup failed ({e}), ", agent.name);
                    info!("Failed to start client for agent {}: {e}", agent.name);
                }
            }
        }
    }

    // Init clocks (time budget)
    let mut time_budgets = vec![resources.time_budget; ordered_player.len()];

    game.init();
    let mut turn = 0;

    while !game.is_finished() && !clients.is_empty() {
        turn += 1;
        let current = game.get_current_player_number();

        // If player is missing, action is none
        let action = if let Some(client) = clients.get_mut(&current) {
            let state_str = game.get_state().to_string();
            let mut buf = [0; MAX_BUFFER_SIZE];

            let time_budget = time_budgets[current];
            let max_duration = Duration::min(max_turn_duration, time_budget);
            let timer_start = std::time::Instant::now();

            let response = client.send_and_recv(state_str.as_bytes(), &mut buf, max_duration);

            let elapsed = timer_start.elapsed();
            time_budgets[current] = time_budgets[current]
                .checked_sub(elapsed)
                .unwrap_or(Duration::ZERO);

            match response {
                Ok(received) => {
                    let response = std::str::from_utf8(&buf[..received]);
                    match response {
                        Ok(text) => match G::Action::from_str(text.trim()) {
                            Ok(action) => Some(action),
                            Err(_) => {
                                info!(
                                    "Agent {} sent invalid action: '{text}' (len = {received})",
                                    ordered_player[current].name
                                );
                                if received == 0 {
                                    errors_string += &format!(
                                        "{} empty string received (player probably crashed), ",
                                        ordered_player[current].name
                                    );
                                } else {
                                    errors_string += &format!(
                                        "{} not an action: '{text}', ",
                                        ordered_player[current].name
                                    );
                                }
                                clients.remove(&current);
                                None
                            }
                        },
                        Err(_) => {
                            info!(
                                "Agent {} sent non-UTF8 response",
                                ordered_player[current].name
                            );
                            errors_string +=
                                &format!("{} non-utf8 response, ", ordered_player[current].name);
                            clients.remove(&current);
                            None
                        }
                    }
                }
                Err(_e) => {
                    info!(
                        "Agent {} did not respond in time ({}ms)",
                        ordered_player[current].name,
                        max_duration.as_millis()
                    );
                    // timeout is silenced when duration is small (time budget exceeded is normal behaviour (must happen))
                    if max_duration >= resources.action_time
                        || max_duration >= (resources.time_budget / 10)
                    {
                        errors_string += &format!(
                            "{}: {_e} response timeout ({}ms) (turn {turn}), ",
                            ordered_player[current].name,
                            max_duration.as_millis()
                        );
                    }
                    clients.remove(&current);
                    None
                }
            }
        } else {
            // Agent was already eliminated/killed/did not start
            None
        };

        // Apply action (even if it's None, Game is supposed to handle elimination logic)
        // Only warn when a non-None action is rejected
        if let Err(e) = game.apply_action(&action) {
            if action.is_some() {
                info!(
                    "player {current}'s action ({}) rejected by Game",
                    action.as_ref().unwrap().to_string()
                );
                errors_string += &format!(
                    "{}'s action '{}' was rejected: {e}, ",
                    ordered_player[current].name,
                    action.unwrap().to_string()
                );
                clients.remove(&current);
            }
        }
    }
    // Kill remaining processes
    drop(clients);

    // Collect final scores
    let mut results = vec![];
    for (i, agent) in ordered_player.iter().enumerate() {
        let score = game.get_player_score(i as u32);
        results.push((agent.clone(), score));
    }

    trace!("match end");
    RunnerResult {
        results,
        resources_freed: resources,
        errors: errors_string,
    }
}
