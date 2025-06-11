use std::time::Instant;

mod game_info;

pub trait Game<State = String, Action = String> {
    fn init(&mut self);
    fn apply_action(&mut self, action: &Action) -> Result<(), ()>; //non mutable ? -> Option(Self)
    fn get_state(&mut self) -> State;
    fn is_finished(&self) -> bool;
    fn get_game_info(&self) -> game_info::GameInfo;
}

pub trait Agent<State = String, Action = String> {
    fn init(&mut self);

    //State == String ? (codingame-like)
    //NOTE: deadline : if using VM, make sure clocks are synch (or use Duration)
    fn select_action(&mut self, state: State, deadline: Instant) -> Option<Action>;
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
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
            game_info::GameInfo { num_player: 0, 
                deterministicness: game_info::Deterministicness::Deterministic,
                sequentialness: game_info::Sequentialness::Sequential,
                information: game_info::Information::PerfectInformation }
        }
    }

    #[test]
    fn test_dyn_game() {
        let game: Box<dyn Game<String, ()>> = Box::new(DummyGame {});
        assert!(game.get_game_info().deterministicness == game_info::Deterministicness::Deterministic);
    }

    struct DummyAgent {}

    impl Agent<String, ()> for DummyAgent {
        fn init(&mut self) {}

        fn select_action(&mut self, _state: String, _deadline: Instant) -> Option<()> {
            Some(())
        }
    }

    #[test]
    fn test_dyn_agent() {
        let mut game: Box<dyn Game<String, ()>> = Box::new(DummyGame {});
        let mut agent: Box<dyn Agent<String, ()>> = Box::new(DummyAgent {});
        assert!(
            Some(()) == agent.select_action(game.get_state(), Instant::now() + Duration::from_millis(100))
        );
    }
}
