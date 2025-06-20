use std::time::Instant;

use agent_interface::*;

struct DummyGame {}

impl Game for DummyGame {
    type State = String;
    type Action = ();

    fn init(&mut self) {}

    fn apply_action(&mut self, _action: &()) -> Result<(), ()> {
        Ok(())
    }

    fn is_finished(&self) -> bool {
        false
    }

    fn get_state(&mut self) -> String {
        "".to_string()
    }

    fn get_game_info(&self) -> game_info::GameInfo {
        game_info::GameInfo {
            num_player: 0,
            deterministicness: game_info::Deterministicness::Deterministic,
            sequentialness: game_info::Sequentialness::Sequential,
            information: game_info::Information::PerfectInformation,
        }
    }

    fn get_player_score(&self, _player_number: u32) -> f32 {
        0.0
    }
}

fn borrow_game<G: Game>(_game: &G) {}

#[test]
fn test_dyn_game() {
    let game = DummyGame {};
    borrow_game(&game);
    assert!(game.get_game_info().deterministicness == game_info::Deterministicness::Deterministic);
}

struct DummyAgent {}

impl Agent<DummyGame> for DummyAgent {
    fn init(&mut self) {}

    fn select_action(
        &mut self,
        _state: <DummyGame as Game>::State,
        _deadline: Instant,
    ) -> Option<<DummyGame as Game>::Action> {
        Some(())
    }
}

fn get_agent_action<G: Game, A: Agent<G>>(state: G::State, _agent: &mut A) -> Option<G::Action> {
    _agent.select_action(state, Instant::now())
}

#[test]
fn test_dyn_agent() {
    let mut game = DummyGame {};
    let mut agent = DummyAgent {};
    assert!(Some(()) == get_agent_action(game.get_state(), &mut agent));
}

struct DummyFactory {}

impl GameFactory<DummyGame> for DummyFactory {
    fn new_game(&self) -> DummyGame {
        DummyGame {}
    }
}

// Legacy code: I will not miss you.
// fn game_maker<State,Action,G: Game<State,Action>>(factory:&Box<dyn GameFactory<State,Action,DummyGame>>) -> DummyGame  {
//     factory.new_game()
// }

fn make_game<G: Game, F: GameFactory<G>>(factory: &F) -> G {
    factory.new_game()
}

#[test]
fn test_dyn_factory() {
    let factory = DummyFactory {};
    assert_eq!(make_game(&factory).get_state(), DummyGame {}.get_state());
}
