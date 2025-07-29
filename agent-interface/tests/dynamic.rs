use std::time::SystemTime;

use agent_interface::*;

struct DummyGame {}

impl Game for DummyGame {
    type State = String;
    type Action = ();

    fn apply_action(&mut self, _action: &Option<()>) -> anyhow::Result<()> {
        Ok(())
    }

    fn is_finished(&self) -> bool {
        false
    }

    fn get_state(&self) -> String {
        "".to_owned()
    }

    fn get_player_score(&self, _player_number: u32) -> f32 {
        0.0
    }

    fn get_current_player_number(&self) -> usize {
        0
    }
}

fn borrow_game<G: Game>(_game: &G) {}

#[test]
fn test_dyn_game() {
    let game = DummyGame {};
    borrow_game(&game);
    assert_eq!(game.get_state(), "");
}

struct DummyAgent {}

impl Agent<DummyGame> for DummyAgent {
    fn init(&mut self) {}

    fn select_action(
        &mut self,
        _state: <DummyGame as Game>::State,
        _deadline: SystemTime,
    ) -> <DummyGame as Game>::Action {
    }
}

fn get_agent_action<G: Game, A: Agent<G>>(state: G::State, _agent: &mut A) -> G::Action {
    _agent.select_action(state, SystemTime::now())
}

#[test]
fn test_dyn_agent() {
    let game = DummyGame {};
    let mut agent = DummyAgent {};
    assert_eq!((), get_agent_action(game.get_state(), &mut agent));
}

struct DummyFactory {}

impl GameFactory<DummyGame> for DummyFactory {
    fn new_game(&self) -> DummyGame {
        DummyGame {}
    }
}

fn make_game<G: Game, F: GameFactory<G>>(factory: &F) -> G {
    factory.new_game()
}

#[test]
fn test_dyn_factory() {
    let factory = DummyFactory {};
    assert_eq!(make_game(&factory).get_state(), DummyGame {}.get_state());
}
