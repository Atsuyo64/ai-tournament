//! Core evaluation logic for running AI tournaments.
//!
//! This module defines the [`Evaluator`] type, which orchestrates tournament execution.
//! Its responsibilities include:
//!
//! - Compiling or loading agents from a specified directory
//! - Enforcing resource limits via [`Constraints`]
//! - Running matches using a user-defined [`TournamentStrategy`]
//! - Returning final scores per agent
//!
//! # Behavior & Configuration
//!
//! Behavior is controlled by a [`Configuration`] object:
//!
//! - When `config.compile_agents = true`, the evaluator expects agents to be **Rust crates** in the given directory, each containing a YAML config at the root.
//! - When `config.compile_agents = false`, the evaluator expects each subdirectory to contain **only two files**:
//!   - An executable binary (the agent)
//!   - A `.yaml` or `.yml` config file (see below)
//!
//! The YAML config describes agent configurations:
//!
//! ```yaml
//! eval: eval_config_name
//! configs:
//!   - baseline: "--default"
//!   - aggressive: "--mode aggressive"
//!   - eval_config_name: "--args used for evaluation"  # Used if `test_all_configs = false`
//! ```
//!
//! > ⚠️ This file is manually parsed and supports only basic YAML. Comments are supported, but advanced YAML features (anchors, nesting, multi-line strings) may not parse correctly.
//!
//! If `config.test_all_configs = true`, all configs listed under `configs` are tested. Otherwise, only the config named in `eval` is used.
//!
//! ## Self-Test Mode
//!
//! When `config.self_test = true`, the evaluator ignores the directory parameter and runs a match **using the current working directory** as a single agent. This is useful for debugging or development.
//!
//! ## Uncontained Mode
//!
//! If `config.allow_uncontained = true`, the evaluator will run even if Linux cgroups v2 or `taskset` are missing.
//! In this case, **only time constraints are enforced**, and CPU/RAM isolation is skipped.
//!
//! # Example
//!
//! See crate-level documentation for an example on how to use the `Evaluator`.

use crate::agent_collector::collect_agents;
use crate::configuration::Configuration;
use crate::constraints::Constraints;
use crate::game_interface::{Game, GameFactory};
use crate::logger::init_logger;
use crate::match_runner::{run_match, MatchSettings, RunnerResult};
use crate::tournament_scheduler::TournamentScheduler;
use crate::tournament_strategy::TournamentStrategy;

use std::collections::HashMap;
use std::fmt::Display;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, Mutex};
use tracing::{info, instrument, trace};

/// The main type for running AI agent tournaments.
///
/// It compiles agents, schedules matches, applies resource constraints, and collects final scores.
///
/// # Type Parameters
/// - `G`: The game type implementing [`Game`]
/// - `F`: A factory implementing [`GameFactory<G>`]
pub struct Evaluator<G: Game, F>
where
    F: GameFactory<G>,
{
    factory: F,
    constraints: Constraints,
    config: Configuration,
    _ff: std::marker::PhantomData<G>,
}

impl<G: Game + Send + 'static, F: GameFactory<G>> Evaluator<G, F> {
    #[instrument(skip_all)]
    /// Create an [`Evaluator`] with given [`Constraints`] and [`GameFactory`]
    pub fn new(factory: F, config: Configuration, constraints: Constraints) -> Evaluator<G, F> {
        if config.log {
            init_logger();
        }

        // trace!("config: {:?}\nconstraints: {:?}", &config, &constraints);
        trace!(?config, ?constraints);

        Evaluator {
            factory,
            config,
            constraints,
            _ff: std::marker::PhantomData,
        }
    }

    /// Executes a tournament between agents found in the specified directory.
    ///
    /// # Parameters
    /// - `directory`: Path to the directory containing agent crates
    /// - `tournament`: Tournament strategy to run
    ///
    /// # Returns
    /// A `HashMap` of agent names to final scores, based on the selected tournament strategy.
    ///
    /// # Errors
    /// Returns an error if the directory is invalid.
    pub fn evaluate<T: TournamentStrategy<G::Score>>(
        &self,
        directory: impl AsRef<std::path::Path>,
        mut tournament: T,
    ) -> anyhow::Result<HashMap<String, T::FinalScore>>
    where
        T::FinalScore: 'static,
    {
        // 1. Exit on panic otherwise the program would be in a deadlock
        Self::setup_panic_hook(self.config.verbose);
        if self.config.verbose {
            disable_line_wrap();
        }

        // 2. get agents name & code in *directory*
        let agents = collect_agents(directory.as_ref(), self.config)?;
        info!(?agents);

        // 3. add agents to tournament
        tournament.add_agents(agents);

        // 4. create scheduler and communication channels
        let mut scheduler = TournamentScheduler::new(self.constraints.clone(), tournament);
        let (tx_result, rx_result) = mpsc::channel();

        // 5. create running matches shared vector (for printing purpose)
        let running = Arc::new(Mutex::new(vec![]));

        // 6. Init matches
        self.launch_initial_matches(&mut scheduler, &tx_result, &running);

        // 7. main loop
        while !scheduler.is_finished() {
            // not finished <=> match running <=> result to receive
            let result = rx_result.recv().unwrap();
            for new_match in scheduler.on_result(result) {
                self.launch_match(new_match, tx_result.clone(), &running);
            }
        }

        if self.config.verbose {
            enable_line_wrap();
        }
        Ok(Self::collect_final_scores(&scheduler))
    }

    fn setup_panic_hook(verbose: bool) {
        let orig_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            if verbose {
                enable_line_wrap();
            }
            orig_hook(panic_info);
            std::process::exit(1);
        }));
    }

    fn launch_initial_matches<T: TournamentStrategy<G::Score>>(
        &self,
        scheduler: &mut TournamentScheduler<T, G::Score>,
        tx_result: &Sender<RunnerResult<G::Score>>,
        running: &Arc<Mutex<Vec<MatchSettings>>>,
    ) {
        for m in scheduler.advance() {
            self.launch_match(m, tx_result.clone(), running);
        }
    }

    fn collect_final_scores<T: TournamentStrategy<G::Score>>(
        scheduler: &TournamentScheduler<T, G::Score>,
    ) -> HashMap<String, T::FinalScore> {
        scheduler
            .final_scores()
            .into_iter()
            .map(|(agent, score)| (agent.name.clone(), score))
            .collect::<HashMap<_, _>>()
    }

    fn launch_match(
        &self,
        match_settings: MatchSettings,
        tx_result: Sender<RunnerResult<G::Score>>,
        running: &Arc<Mutex<Vec<MatchSettings>>>,
    ) {
        let game = self.factory.new_game();
        let mutex = running.clone();

        let mut guard = mutex.lock().expect("poisoned");
        guard.push(match_settings.clone());
        if self.config.verbose {
            print_running_matches(&guard);
        }
        drop(guard);

        let config = self.config;
        std::thread::spawn(move || {
            let result = run_match(match_settings.clone(), config, game);

            if config.verbose {
                print_runner_result(&match_settings, &result);
            }
            Self::remove_running_match(&mutex, &match_settings);

            tx_result.send(result).unwrap();
        });
    }

    fn remove_running_match(mutex: &Mutex<Vec<MatchSettings>>, running: &MatchSettings) {
        let mut guard = mutex.lock().expect("poisoned");
        let pos = guard
            .iter()
            .position(|s| s == running)
            .expect("error: got result of a match that was not started (or got result twice)");
        guard.remove(pos);
    }
}

fn print_runner_result<S: Display + PartialOrd>(
    match_settings: &MatchSettings,
    result: &RunnerResult<S>,
) {
    let mut ordered_scores = Vec::new();
    for player in &match_settings.ordered_player {
        let name = &player.name;
        let res = result
            .results
            .iter()
            .find_map(|a| if &a.0.name == name { Some(&a.1) } else { None })
            .expect("agent not found");
        ordered_scores.push(res);
    }
    let ordered_scores = ordered_scores
        .into_iter()
        .map(|f| format!("{f}"))
        .collect::<Vec<_>>()
        .join("-");

    // clear line, green match, results, red errors, start of line
    println!(
        "\x1b[2K\x1b[32m{match_settings}: \x1b[39m{ordered_scores} \x1b[31m{}\x1b[39m\x1b[0G",
        result.errors
    );
}

fn print_running_matches(running: &[MatchSettings]) {
    // clear, green, default, start of line
    print!(
        "\x1b[2K\x1b[32mRunning...:\x1b[39m {}\x1b[0G",
        running
            .iter()
            .map(MatchSettings::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    );
    let _ = std::io::Write::flush(&mut std::io::stdout());
}

fn disable_line_wrap() {
    print!("\x1b[?7l");
}

fn enable_line_wrap() {
    print!("\x1b[?7h");
}
