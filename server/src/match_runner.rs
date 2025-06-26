use crate::confrontation::Confrontation;
use agent_interface::Game;

pub fn run_match<G: Game>(
    confrontation: &Confrontation,
    game: G,
    megabytes_per_agent: u32,
) {
    todo!()
    //let mut clients = confrontation.ordered_player.iter().flat_map(f);
}
