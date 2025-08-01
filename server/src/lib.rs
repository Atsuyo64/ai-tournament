//! # AI Agent Evaluator – Server Crate
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
//! # impl server::Game for YourGame {
//! #     type State = u32;
//! #     type Action = u32;
//! #     fn apply_action(&mut self, _action: &Option<Self::Action>) -> anyhow::Result<()> { Ok(()) }
//! #     fn get_state(&self) -> Self::State { 0 }
//! #     fn get_current_player_number(&self) -> usize { 0 }
//! #     fn is_finished(&self) -> bool { true }
//! #     fn get_player_score(&self, _player_number: u32) -> f32 { 0.0 }
//! # }
//! # impl server::GameFactory<YourGame> for YourGame {
//! #     fn new_game(&self) -> YourGame { YourGame }
//! # }
//! use std::{collections::HashMap, time::Duration};
//! use anyhow;
//! use server::{
//!     constraints::ConstraintsBuilder,
//!     server::Evaluator,
//!     tournament_strategy::{SinglePlayerScore, SinglePlayerTournament},
//! };
//!
//! // Your custom game implementing the Game + GameFactory traits
//!
//! fn main() -> anyhow::Result<()> {
//!     // Define per-agent constraints
//!     let constraints = ConstraintsBuilder::new()
//!         .with_ram_per_agent(1000) // in MB
//!         .with_action_timeout(Duration::from_millis(100))
//!         .build()?;
//!
//!     let factory = YourGame::new();
//!     let evaluator = Evaluator::new(factory, constraints);
//!
//!     let tournament = SinglePlayerTournament::new(10); // Run 10 games per agent
//!     let results: HashMap<String, SinglePlayerScore> =
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
//! # impl server::Game for YourGame {
//! #     type State = u32;
//! #     type Action = u32;
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
//! };
//!
//! use anyhow;
//!
//! use server::Game;
//!
//! fn main() -> anyhow::Result<()> {
//!     let mut args = env::args();
//!     let _ = args.next(); // Skip binary name
//!
//!     // Read the port number to connect to
//!     let port = args.next().unwrap().parse().unwrap();
//!     let addr = SocketAddrV4::new(Ipv4Addr::from_str("127.0.0.1")?, port);
//!     let mut stream = TcpStream::connect(addr)?;
//!
//!     let mut agent = YourAgent::new();
//!
//!     // Interaction loop
//!     loop {
//!         let mut buf = [0; 4096];
//!         let n = stream.read(&mut buf)?;
//!         let string = str::from_utf8(&buf[..n]).unwrap();
//!
//!         // Parse game state, compute action, send it back
//!         let game_state = string.parse::<<YourGame as Game>::State>().unwrap();
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

pub use agent_interface::{anyhow, Game, GameFactory};
mod agent;
mod agent_compiler;
mod client_handler;
mod confrontation;
pub mod constraints;
mod match_runner;
pub mod server;
mod tournament;
pub mod tournament_strategy;
