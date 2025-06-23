use agent_interface::game_info::Deterministicness::*;
use agent_interface::game_info::GameInfo;
use agent_interface::game_info::Information::*;
use agent_interface::game_info::Sequentialness::*;
use agent_interface::*;

pub struct DummyGame { counter : u32}

impl Game for DummyGame {
    type State = u32;

    type Action = u32;

    fn init(&mut self) {}

    fn apply_action(&mut self, _action: &Self::Action) -> Result<(), ()> {
        Ok(())
    }

    fn get_state(&mut self) -> Self::State { self.counter -= 1; self.counter}

    fn is_finished(&self) -> bool {
        self.counter <= 0
    }

    fn get_game_info(&self) -> game_info::GameInfo {
        GameInfo {
            num_player: 1,
            deterministicness: Deterministic,
            sequentialness: Sequential,
            information: PerfectInformation,
        }
    }

    fn get_player_score(&self, _player_number: u32) -> f32 {
        1.0
    }
}

pub struct DummyFactory {}

impl GameFactory<DummyGame> for DummyFactory {
    fn new_game(&self) -> DummyGame {
        DummyGame { counter: 10 }
    }
}

pub struct DummyAgent;

impl Agent<DummyGame> for DummyAgent {
    fn init(&mut self) {}

    fn select_action(
        &mut self,
        state: <DummyGame as Game>::State,
        _deadline: std::time::Instant,
    ) -> Option<<DummyGame as Game>::Action> {
        Some(state+1)
    }
}
