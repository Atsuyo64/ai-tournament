use std::time::Instant;

use agent_interface::*;

struct DummyGame {}

impl Game<String, ()> for DummyGame {
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

fn borrow_game<State, Action>(_game: &Box<dyn Game<State, Action>>) {}

#[test]
fn test_dyn_game() {
    let game: Box<dyn Game<String, ()>> = Box::new(DummyGame {});
    borrow_game(&game);
    assert!(game.get_game_info().deterministicness == game_info::Deterministicness::Deterministic);
}

struct DummyAgent {}

impl Agent<String, ()> for DummyAgent {
    fn init(&mut self) {}

    fn select_action(&mut self, _state: String, _deadline: Instant) -> Option<()> {
        Some(())
    }
}

fn get_agent_action<State, Action>(
    state: State,
    _agent: &mut Box<dyn Agent<State, Action>>,
) -> Option<Action> {
    _agent.select_action(state, Instant::now())
}

#[test]
fn test_dyn_agent() {
    let mut game: Box<dyn Game<String, ()>> = Box::new(DummyGame {});
    let mut agent: Box<dyn Agent<String, ()>> = Box::new(DummyAgent {});
    assert!(Some(()) == get_agent_action(game.get_state(), &mut agent));
}

struct DummyFactory {}

impl GameFactory<String, (), DummyGame> for DummyFactory {
    fn new_game(&self) -> DummyGame {
        DummyGame {}
    }
}

// fn game_maker<State,Action,G: Game<State,Action>>(factory:&Box<dyn GameFactory<State,Action,DummyGame>>) -> DummyGame  {
//     factory.new_game()
// }

fn make_game<State, Action, G: Game<State, Action>, F: GameFactory<State, Action, G>>(
    factory: &F,
) -> G {
    factory.new_game()
}

#[test]
fn test_dyn_factory() {
    let factory = DummyFactory {};
    //FIXME: Is it possible to create factory_in_a_box from factory ? (probably not)
    let _factory_in_a_box: Box<dyn GameFactory<String, (), DummyGame>> = Box::new(DummyFactory {});
    assert_eq!(make_game(&factory).get_state(), DummyGame {}.get_state());
}
