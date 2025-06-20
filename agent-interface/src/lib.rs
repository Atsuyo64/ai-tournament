use std::time::Instant;

pub mod game_info;

/// What the game should imlement
pub trait Game<State, Action> {
    fn init(&mut self);
    fn apply_action(&mut self, action: &Action) -> Result<(), ()>; //non mutable ? -> Option(Self)
    /// The state that will be given to the current player(s)
    fn get_state(&mut self) -> State;
    fn is_finished(&self) -> bool;
    fn get_game_info(&self) -> game_info::GameInfo;
    fn get_player_score(&self,player_number:u32) -> f32;
}

/// What the agent should implement
pub trait Agent<State, Action> {
    fn init(&mut self);

    //State == String ? (codingame-like)
    //NOTE: deadline : if using VM, make sure clocks are synch (or use Duration)
    fn select_action(&mut self, state: State, deadline: Instant) -> Option<Action>;
}

/// What will be given to the evaluator to allow it to create games
pub trait GameFactory<State,Action, G: Game<State,Action>> {
    fn new_game(&self) -> G;
} 



// pub struct TcpAgentServer {
//     command: String,
//     args: Vec<String>,
//     max_agent_memory: i64,
//     agent_cpus: String,
// }

// impl TcpAgentServer {
//     pub fn new(
//         command: String,
//         args: Vec<String>,
//         max_agent_memory: i64,
//         agent_cpus: String,
//     ) -> TcpAgentServer {
//         TcpAgentServer {
//             command,
//             args,
//             max_agent_memory,
//             agent_cpus,
//         }
//     }
//     pub fn get_command(&self) -> &String {
//         &self.command
//     }
//     pub fn get_args(&self) -> &Vec<String> {
//         &self.args
//     }
// }

// impl<S: ToString + FromStr, A: ToString + FromStr> Agent<S, A> for TcpAgentServer {
//     fn init(&mut self) {
//         //cgroup_manager::LimitedProcess::launch(&self.command, &self.args, self.max_agent_memory, &self.agent_cpus);
//     }

//     fn select_action(&mut self, _state: S, _deadline: Instant) -> Option<A> {
//         todo!()
//         // socket.send(state.to_string())
//         // let result = wait_with_deadline(deadline)
//         // return result.from_string()
//     }
// }

// trait TcpAgentServer<State, Action>  where State: ToString + FromString, Action: ToString + FromString {

//     fn select_Action(&mut self, state: &s)
// }
