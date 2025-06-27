use std::{str::FromStr, time::Duration};

use crate::{
    available_resources::MatchResourceLimit,
    client_handler::ClientHandler,
    confrontation::Confrontation,
};
use agent_interface::Game;
use anyhow::{anyhow, Context};
use log::{debug, error, trace, warn};

pub fn run_match<G: Game>(confrontation: &Confrontation, mut game: G, _megabytes_per_agent: u32)
where
    G::Action: FromStr,
    G::State: ToString,
{
    let mut clients: Vec<_> = confrontation
        .ordered_player
        .iter()
        .map(|agent| ClientHandler::init(agent.clone(), MatchResourceLimit::empty()))
        .collect();

    clients.iter().for_each(|res| {
        if let Err(e) = res {
            warn!("Error creating client: {e} : {:?}",e.chain().nth(1));
            
        }
    });

    //FIXME: game.get_current_player_number or something (for games when it is not a simple round robin)
    //TODO: Logger ? (crate `log`)
    let mut current_player_number = 0usize;
    game.init();
    while !game.is_finished() {
        trace!("player to play: {current_player_number}");
        let state = game.get_state().to_string();
        if let Ok(client) = &mut clients[current_player_number] {
            match try_get_action::<G::Action>(client, state) {
                Err(e) => {
                    error!("{e:?}");
                    todo!("incorrect response")
                } //NOTE: should think about killing agents in case of error
                Ok(a) => {
                    game.apply_action(&a).unwrap() //FIXME: should also fix interface tbh...
                }
            }
        } else {
            debug!("TODO: game.apply_action(None) ?")
        }
        current_player_number = (current_player_number + 1) % clients.len();
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
    let s = str::from_utf8(&buf[..n]).context("response error")?;
    match A::from_str(s) {
        Ok(a) => Ok(a),
        Err(_) => Err(anyhow!("from_str")),
    }
}
