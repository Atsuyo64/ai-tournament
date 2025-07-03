use std::{str::FromStr, time::Duration};

use crate::{
    client_handler::ClientHandler,
    tournament_maker::MatchSettings,
};
use agent_interface::Game;
use anyhow::{anyhow, Context};
use tracing::{error, info, instrument, trace, warn};

#[instrument(skip_all,fields(VS=match_settings.to_string()))]
pub fn run_match<G: Game>(match_settings: MatchSettings, mut game: G)
where
    G::Action: FromStr,
    G::State: ToString,
{
    let MatchSettings {
        ordered_player,
        resources,
        on_resource_free,
        on_final_score,
    } = match_settings;
    assert_eq!(
        resources.cpus.len(),
        resources.cpus_per_agent * ordered_player.len()
    );
    info!("new match started");
    let mut clients: Vec<_> = ordered_player
        .iter()
        .map(|agent| ClientHandler::init(agent.clone(), &resources)) //FIXME: resources limit
        .collect();

    clients.iter().for_each(|res| {
        if let Err(e) = res {
            warn!("Error creating client: {e} : {:?}", e.chain().nth(1));
        }
    });

    let mut current_player_number;
    game.init();
    while !game.is_finished() {
        let state = game.get_state().to_string();
        current_player_number = game.get_current_player_number();
        trace!("player to play: {current_player_number}");

        if clients[current_player_number].is_err() {
            let _ = game.apply_action(&None);
            continue;
        }
        let action = match try_get_action(clients[current_player_number].as_mut().unwrap(), state) {
            Ok(a) => a,
            Err(e) => {
                warn!(
                    "no response from agent {:?} : {e}",
                    ordered_player[current_player_number]
                        .path_to_exe
                        .as_ref()
                        .unwrap()
                );
                let _ = game.apply_action(&None);
                let _ = clients[current_player_number]
                    .as_mut()
                    .unwrap()
                    .kill_child()
                    .map_err(|e| error!("could not kill client {e}"));
                clients[current_player_number] = Err(anyhow!("stopped"));
                continue;
            }
        };
        if game.apply_action(&Some(action)).is_err() {
            warn!(
                "invalid action from agent {:?}",
                ordered_player[current_player_number]
                    .path_to_exe
                    .as_ref()
                    .unwrap()
            );
            let _ = clients[current_player_number]
                .as_mut()
                .unwrap()
                .kill_child()
                .map_err(|e| error!("could not kill client {e}"));
            clients[current_player_number] = Err(anyhow!("stopped"));
        }
    }

    //TODO: update scores

    clients.iter_mut().for_each(|c| {
        if let Ok(c) = c {
            c.kill_child().unwrap()
        }
    });
}

fn try_get_action<A>(client: &mut ClientHandler, state: String) -> anyhow::Result<A>
where
    A: FromStr,
{
    let mut buf = [0; 4096];
    let n = client
        .send_and_recv(state.as_bytes(), &mut buf, Duration::from_secs(1))
        .context("message error")?;
    let s = str::from_utf8(&buf[..n]).context("invalid utf8 response")?;
    match A::from_str(s) {
        Ok(a) => Ok(a),
        Err(_) => Err(anyhow!("from_str")),
    }
}
