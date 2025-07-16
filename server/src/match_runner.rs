use std::{collections::HashMap, fmt::Display, str::FromStr, sync::Arc, time::Duration};

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

pub type MatchResult = Vec<(Arc<Agent>,f32)>;

#[derive(Debug, Clone)]
pub struct RunnerResult {
    pub results: MatchResult,
    pub resources_freed: Constraints,
    // pub duration: Duration,
}

#[instrument(skip_all,fields(VS=settings.to_string()))]
pub fn run_match<G: Game>(settings: MatchSettings, mut game: G) -> RunnerResult
where
    G::Action: FromStr,
    G::State: ToString,
{
    trace!("game started");
    let MatchSettings {
        ordered_player,
        resources,
    } = settings;

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

    while !game.is_finished() && !clients.is_empty() { //FIXME: if client is empty from the start (e.g. client does not compile)
        let current = game.get_current_player_number();
        let time_budget = time_budgets[current];
        let timer_start = std::time::Instant::now();

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
                                    "Agent {} sent invalid action: '{}' (len = {received})",
                                    ordered_player[current].name, text
                                );
                                clients.remove(&current);
                                None
                            }
                        },
                        Err(_) => {
                            warn!(
                                "Agent {} sent non-UTF8 response",
                                ordered_player[current].name
                            );
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
                    clients.remove(&current);
                    None
                }
            }
        } else {
            // Agent was already eliminated/killed/did not start
            None
        };

        let elapsed = timer_start.elapsed();
        time_budgets[current]= time_budgets[current].checked_sub(elapsed).unwrap_or(Duration::ZERO);

        // Apply action (even if it's None, Game is supposed to handle elimination logic)
        // Only warn when a non-None action is rejected
        if game.apply_action(&action).is_err() && action.is_some() {
            warn!("player {}'s action rejected by Game", current);
            clients.remove(&current);
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
    }
}
