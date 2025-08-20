//! Module defining traits that need to be implemented to use the evaluator

use std::time::SystemTime;

/// What the game should implement
pub trait Game {
    /// Type representing game state.
    type State;
    /// What should be returned by players to make the game progress.
    type Action;

    /// Apply an optional action to the game.
    ///
    /// `Option<Action>` is necessary because when `num_players >= 3`
    /// a player might be eliminated by not playing,
    /// but the game could still continue with the remaining players.
    ///
    /// # Error
    /// Returned when `action` is not valid (`action` is None, `action` is not allowed, ...).
    ///
    /// Even if `action` is not valid, `current_player` should be updated!
    fn apply_action(&mut self, action: &Option<Self::Action>) -> anyhow::Result<()>;

    /// The current state that will be given to the current player
    ///
    /// Does not returns &State because of annoying lifetime to deal with.
    fn get_state(&self) -> Self::State;

    /// The number of the player that should play now
    fn get_current_player_number(&self) -> usize;

    /// True if game is finished
    fn is_finished(&self) -> bool;

    /// Used at the end of the game to collect players score
    fn get_player_score(&self, player_number: u32) -> f32;
}

/// What the agent should implement. Not used yet, be could allow to launch agent without creating
/// processes
#[allow(dead_code)]
#[doc(hidden)]
pub trait Agent<G: Game> {
    fn init(&mut self);

    fn select_action(&mut self, state: G::State, deadline: SystemTime) -> G::Action;
}

/// What will be given to the evaluator to allow it to create games
pub trait GameFactory<G: Game> {
    /// Returns an initialized game
    fn new_game(&self) -> G;
}

#[cfg(test)]
mod interface_tests {
    use std::time::SystemTime;

    use super::*;

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
}
