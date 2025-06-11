use std::time::{Duration, Instant};

use agent_interface;

/// Test server
fn main() {
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
