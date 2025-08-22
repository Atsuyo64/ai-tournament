//! # Ai Tournament
//!
//! A modular Rust crate system for evaluating AI agents via customizable tournaments, supporting sandboxed execution and flexible constraints.
//!
//! It provides:
//! - Match scheduling and execution (`Evaluator`)
//! - Tournament logic via the `TournamentStrategy` trait
//! - Built-in strategies like `SinglePlayerTournament`, `SwissTournament` and `RoundRobin`
//! - Resource constraints enforced through Linux cgroups v2 and `taskset`
//!
//! Each match consists of one or more agents, each running as a separate OS process.
//! Process-level isolation applies constraints such as CPU affinity, memory limits, and timeouts.
//!
//! # Documentation Overview
//!
//! - For details about the core tournament execution and agent lifecycle, see the [`server`] module.
//! - For configuring evaluation behavior, resource limits, and execution environment,
//! see [`Configuration`](crate::configuration::Configuration) and [`constraints`].
//! - To understand tournament formats and match scheduling, see the [`TournamentStrategy`](crate::tournament_strategy::TournamentStrategy) trait and its implementations.
//! - For implementing custom games and agents, check out the [`Game`] and [`GameFactory`] traits.
//!
//! This crate is designed to be modular and extensible, allowing you to customize agent compilation, match execution, and resource management.
//!
//! # Usage Example
//!
//! Below is a minimal example of using the evaluator with a custom game and built-in tournament:
//!
//! ```no_run
//! # struct YourAgent;
//! # impl YourAgent {
//! #     pub fn new() -> Self { YourAgent }
//! #     pub fn select_action(&mut self, _action: u32) -> u32 { 0 }
//! # }
//! # #[derive(Clone)]
//! # struct YourGame;
//! # impl YourGame {
//! #     pub fn new() -> YourGame { YourGame }
//! # }
//! # impl ai_tournament::game_interface::Game for YourGame {
//! #     type State = u32;
//! #     type Action = u32;
//! #     type Score = f32;
//! #     fn apply_action(&mut self, _action: &Option<Self::Action>) -> anyhow::Result<()> { Ok(()) }
//! #     fn get_state(&self) -> Self::State { 0 }
//! #     fn get_current_player_number(&self) -> usize { 0 }
//! #     fn is_finished(&self) -> bool { true }
//! #     fn get_player_score(&self, _player_number: u32) -> f32 { 0.0 }
//! # }
//! # impl ai_tournament::game_interface::GameFactory<YourGame> for YourGame {
//! #     fn new_game(&self) -> YourGame { YourGame }
//! # }
//! use std::{collections::HashMap, time::Duration};
//! use anyhow;
//! use ai_tournament::prelude::*;
//!
//! fn main() -> anyhow::Result<()> {
//!     // Define per-agent constraints
//!     let constraints = ConstraintsBuilder::new()
//!         .with_ram_per_agent(1000) // in MB
//!         .with_action_timeout(Duration::from_millis(100))
//!         .build()?;
//!
//!     // Create a configuration allowing uncontained execution if cgroup v2 or taskset are not
//!     // available
//!     let config = Configuration::new().with_allow_uncontained(true);
//!
//!     // Your custom game implementing the Game + GameFactory traits
//!     let factory = YourGame::new();
//!     let evaluator = Evaluator::new(factory, config, constraints);
//!
//!     let mut tournament = SinglePlayerTournament::new(10); // Run 10 games per agent
//!     let results: HashMap<String, SinglePlayerScore<_>> =
//!         evaluator.evaluate("path_to_agents_directory", tournament)?;
//!
//!     // Sort and display scores
//!     let mut sorted = results.iter().collect::<Vec<_>>();
//!     sorted.sort_by(|a, b| b.1.cmp(a.1));
//!     for (agent_name, score) in sorted {
//!         println!("{agent_name}: {score:?}");
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! # Example Agent
//!
//! Here’s a minimal agent implementation that communicates over TCP:
//!
//! ```no_run
//! # struct YourAgent;
//! # impl YourAgent {
//! #     pub fn new() -> Self { YourAgent }
//! #     pub fn select_action(&mut self, _action: u32) -> u32 { 0 }
//! # }
//! # struct YourGame;
//! # impl ai_tournament::game_interface::Game for YourGame {
//! #     type State = u32;
//! #     type Action = u32;
//! #     type Score = f32;
//! #     fn apply_action(&mut self, _action: &Option<Self::Action>) -> anyhow::Result<()> { Ok(()) }
//! #     fn get_state(&self) -> Self::State { 0 }
//! #     fn get_current_player_number(&self) -> usize { 0 }
//! #     fn is_finished(&self) -> bool { true }
//! #     fn get_player_score(&self, _player_number: u32) -> f32 { 0.0 }
//! # }
//! use std::{
//!     env,
//!     io::{Read, Write},
//!     net::{Ipv4Addr, SocketAddrV4, TcpStream},
//!     str::{self, FromStr},
//!     time::Duration,
//! };
//!
//! use anyhow;
//!
//! use ai_tournament::game_interface::Game;
//!
//! fn main() -> anyhow::Result<()> {
//!     let mut args = env::args();
//!     let _ = args.next(); // Skip binary name
//!
//!     // Read the port number to connect to
//!     let port = args.next().unwrap().parse()?;
//!     let addr = SocketAddrV4::new(Ipv4Addr::from_str("127.0.0.1")?, port);
//!     let mut stream = TcpStream::connect(addr)?;
//!
//!     // Optionally, reading time_budget and action_timeout from next args
//!     let total_time_budget = Duration::from_micros(args.next().unwrap().parse()?);
//!     let action_timeout = Duration::from_micros(args.next().unwrap().parse()?);
//!     // After the four first arguments (binary name, port number, time budget, and action
//!     // timeout) will follow your arguments defined in your config file
//!
//!     let mut agent = YourAgent::new();
//!
//!     // Interaction loop
//!     loop {
//!         let mut buf = [0; 4096];
//!         let n = stream.read(&mut buf)?;
//!         let string = str::from_utf8(&buf[..n])?;
//!
//!         // Parse game state, compute action, send it back
//!         let game_state = string.parse::<<YourGame as Game>::State>()?;
//!         let action = agent.select_action(game_state);
//!         stream.write_all(action.to_string().as_bytes())?;
//!     }
//! }
//! ```
//!
//! ## Agent Requirements
//!
//! - `Game::State` and `Game::Action` must implement `ToString` and `FromStr`
//! - Agent logic must terminate within the configured timeout
//! - Communication is done over TCP using a basic protocol:
//!  * Server -> Agent : string of Game::State
//!  * Agent -> Server : string of Game::Action
#![warn(missing_docs)]

mod cgroup_manager;
pub mod game_interface;
pub use anyhow;
mod agent;
mod agent_collector;
mod client_handler;
pub mod configuration;
mod confrontation;
pub mod constraints;
mod logger;
mod match_runner;
pub mod server;
mod tournament_scheduler;
pub mod tournament_strategy;

/// Commonly used types and traits for quick access.
///
/// Import this prelude to get started easily:
/// ```rust
/// use ai_tournament::prelude::*;
/// ```
///
/// Includes:
/// - [`Configuration`](crate::configuration::Configuration)
/// - [`ConstraintsBuilder`](crate::constraints::ConstraintsBuilder)
/// - [`Evaluator`](crate::server::Evaluator)
/// - all built-in [`Tournament strategies`](crate::tournament_strategy)
pub mod prelude {
    pub use crate::configuration::Configuration;
    pub use crate::constraints::ConstraintsBuilder;
    pub use crate::game_interface::Game;
    pub use crate::game_interface::GameFactory;
    pub use crate::server::Evaluator;
    pub use crate::tournament_strategy::*;
}
