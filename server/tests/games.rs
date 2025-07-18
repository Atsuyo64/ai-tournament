use std::fmt::Display;
use std::str::FromStr;
use std::vec;

use agent_interface::game_info::Deterministicness::*;
use agent_interface::game_info::GameInfo;
use agent_interface::game_info::Information::*;
use agent_interface::game_info::Sequentialness::*;
use agent_interface::*;

pub struct DummyGame {
    counter: u32,
    got_none: bool,
}

impl Game for DummyGame {
    type State = u32;

    type Action = u32;

    fn init(&mut self) {}

    fn apply_action(&mut self, _action: &Option<Self::Action>) -> anyhow::Result<()> {
        self.got_none |= _action.is_none();
        Ok(())
    }

    fn get_state(&mut self) -> Self::State {
        self.counter -= 1;
        self.counter
    }

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
        if self.got_none { 0.0 } else { 1.0 }
    }

    fn get_current_player_number(&self) -> usize {
        0
    }
}

#[derive(Clone)]
pub struct DummyFactory {}

impl GameFactory<DummyGame> for DummyFactory {
    fn new_game(&self) -> DummyGame {
        DummyGame { counter: 10, got_none: false }
    }
}

// Unused
pub struct DummyAgent;

impl Agent<DummyGame> for DummyAgent {
    fn init(&mut self) {}

    fn select_action(
        &mut self,
        state: <DummyGame as Game>::State,
        _deadline: std::time::Instant,
    ) -> Option<<DummyGame as Game>::Action> {
        Some(state + 1)
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum RpsAction {
    Rock,
    Paper,
    Scissors,
}

#[allow(dead_code)]
impl RpsAction {
    /// Any number of players supported (however, more players = more ties)
    pub fn get_winners(actions: &[Option<RpsAction>]) -> Option<RpsAction> {
        let contains_rock = actions.contains(&Some(RpsAction::Rock));
        let contains_paper = actions.contains(&Some(RpsAction::Paper));
        let contains_scissors = actions.contains(&Some(RpsAction::Scissors));
        let at_least_one = contains_rock || contains_paper || contains_scissors;
        let at_least_two = (contains_rock && contains_paper)
            || (contains_paper && contains_scissors)
            || (contains_scissors && contains_rock);
        let tie = contains_rock && contains_paper && contains_scissors;

        if !at_least_one || tie || !at_least_two {
            return None;
        }

        // if !at_least_two {
        //     // only one type
        //     if contains_rock {
        //         return Some(Self::Rock);
        //     } else if contains_paper {
        //         return Some(Self::Paper);
        //     } else {
        //         return Some(Self::Scissors);
        //     }
        // }

        let winners = if contains_rock && contains_paper {
            RpsAction::Paper
        } else if contains_paper && contains_scissors {
            RpsAction::Scissors
        } else {
            RpsAction::Rock
        };
        Some(winners)
    }

    pub fn lose_against(self) -> Self {
        match self {
            RpsAction::Rock => RpsAction::Paper,
            RpsAction::Paper => RpsAction::Scissors,
            RpsAction::Scissors => RpsAction::Rock,
        }
    }

    pub fn win_against(self) -> Self {
        match self {
            RpsAction::Rock => RpsAction::Scissors,
            RpsAction::Paper => RpsAction::Rock,
            RpsAction::Scissors => RpsAction::Paper,
        }
    }
}

impl FromStr for RpsAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Rock" => Ok(Self::Rock),
            "Paper" => Ok(Self::Paper),
            "Scissors" => Ok(Self::Scissors),
            _ => Err(()),
        }
    }
}

impl Display for RpsAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rock => write!(f, "Rock"),
            Self::Paper => write!(f, "Paper"),
            Self::Scissors => write!(f, "Scissors"),
        }
    }
}

#[derive(Clone)]
pub struct RockPaperScissors {
    num_players: usize,
    scores: Vec<i32>,
}

impl RockPaperScissors {
    pub const SCORE_TO_WIN: i32 = 10;

    pub fn new(num_players: usize) -> Self {
        RockPaperScissors {
            num_players,
            scores: vec![0; num_players],
        }
    }

    pub fn finished(&self) -> bool {
        self.scores.iter().any(|score| *score >= Self::SCORE_TO_WIN)
    }

    pub fn update_scores(&mut self, actions: &[Option<RpsAction>]) {
        assert_eq!(self.num_players, actions.len());

        if let Some(winners) = RpsAction::get_winners(actions) {
            for i in 0..self.num_players {
                if actions[i] == Some(winners) {
                    self.scores[i] += 1;
                } else if actions[i] == None {
                    self.scores[i] -= 1;
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct RPSWrapper {
    rps: RockPaperScissors,
    actions_buffer: Vec<Option<RpsAction>>,
    finished: bool,
    current_player: usize,
    state: RpsState,
}

#[derive(Clone)]
pub struct RpsState {
    pub previous_actions: Vec<Option<RpsAction>>,
}

impl FromStr for RpsState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut v = Vec::new();
        for slice in s.split(",") {
            if slice.is_empty() {
                continue;
            } else if slice == "None" {
                v.push(None);
            } else {
                let value = Some(slice.parse()?);
                v.push(value);
            }
        }
        Ok(RpsState {
            previous_actions: v,
        })
    }
}

pub struct PlayerState {
    pub player_number: usize,
    pub state: RpsState,
}

impl Display for PlayerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.player_number)?;
        let s = self
            .state
            .previous_actions
            .iter()
            .map(|opt| {
                if let Some(action) = opt {
                    action.to_string()
                } else {
                    "None".to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(",");
        write!(f, "{s}")
    }
}

impl FromStr for PlayerState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines = s.lines().collect::<Vec<_>>();
        if lines.len() == 1 {
            Ok(Self {
                player_number: lines[0]
                    .parse()
                    .map_err(|_| format!("not a usize: '{}'", lines[0]))?,
                state: RpsState::from_str("").map_err(|_| format!("not a RpsState: '{}'", ""))?,
            })
        } else if lines.len() == 2 {
            let player_number = lines[0]
                .parse()
                .map_err(|_| format!("not a usize: '{}'", lines[0]))?;
            let state = lines[1]
                .parse()
                .map_err(|_| format!("not a RpsState: '{}'", lines[1]))?;
            Ok(Self {
                player_number,
                state,
            })
        } else {
            Err(format!(
                "not a PlayerState ({} lines): '{}'",
                lines.len(),
                s
            ))
        }
    }
}

impl Default for RPSWrapper {
    fn default() -> Self {
        Self {
            rps: RockPaperScissors::new(2),
            actions_buffer: Default::default(),
            finished: Default::default(),
            current_player: Default::default(),
            state: RpsState {
                previous_actions: vec![],
            },
        }
    }
}

impl Game for RPSWrapper {
    type State = PlayerState;

    type Action = RpsAction;

    fn init(&mut self) {
        *self = RPSWrapper::default();
    }

    fn apply_action(&mut self, action: &Option<Self::Action>) -> anyhow::Result<()> {
        self.actions_buffer.push(*action);
        self.current_player += 1;
        if self.current_player == self.rps.num_players {
            self.rps.update_scores(&self.actions_buffer);
            self.finished = self.finished
                || self.rps.finished()
                || self.actions_buffer.iter().all(Option::is_none);
            self.current_player = 0;
            self.state = RpsState {
                previous_actions: self.actions_buffer.clone(),
            };
            self.actions_buffer = vec![];
        }
        if action.is_none() {
            Err(anyhow::anyhow!("action is None"))
        } else {
            Ok(())
        }
    }

    fn get_state(&mut self) -> Self::State {
        PlayerState {
            player_number: self.current_player,
            state: RpsState {
                previous_actions: self.actions_buffer.clone(),
            },
        }
    }

    fn get_current_player_number(&self) -> usize {
        self.current_player
    }

    fn is_finished(&self) -> bool {
        self.finished || self.rps.finished()
    }

    fn get_game_info(&self) -> game_info::GameInfo {
        GameInfo {
            num_player: self.rps.num_players as u32,
            deterministicness: Deterministic,
            sequentialness: Simultaneous,
            information: PerfectInformation,
        }
    }

    fn get_player_score(&self, player_number: u32) -> f32 {
        self.rps.scores[player_number as usize] as f32
    }
}

impl GameFactory<RPSWrapper> for RPSWrapper {
    fn new_game(&self) -> RPSWrapper {
        RPSWrapper::default()
    }
}
