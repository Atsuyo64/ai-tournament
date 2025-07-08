use crate::agent_compiler;
use crate::constraints::Constraints;
use crate::match_runner::run_match;
use crate::tournament::{Scores, TournamentScheduler};
use crate::tournament_strategy::TournamentStrategy;

pub use agent_interface::{Game, GameFactory};
use anyhow::bail;
use std::str::FromStr;
use std::sync::mpsc;

pub struct Evaluator<G, F>
where
    G: Game,
    F: GameFactory<G>,
    G::State: FromStr + ToString,
    G::Action: FromStr + ToString,
{
    factory: F,
    params: Constraints,
    _ff: std::marker::PhantomData<G>,
}

impl<G: Game, F: GameFactory<G>> Evaluator<G, F>
where
    F: Clone + Send + 'static,
    G::State: FromStr + ToString,
    G::Action: FromStr + ToString,
    G: 'static + Send,
{
    pub fn new(factory: F, params: Constraints) -> Evaluator<G, F> {
        Evaluator {
            factory,
            params,
            _ff: std::marker::PhantomData,
        }
    }

    pub fn evaluate<T:TournamentStrategy>(&self, directory: &std::path::Path, mut tournament: T) -> anyhow::Result<Scores> {
        // 1. get agents name & code in *directory*
        if !directory.is_dir() {
            bail!("{directory:?} is not a directory");
        }

        // 2. try to compile each one of them
        let agents = agent_compiler::compile_all_agents(directory);

        // 3. add agents to tournament
        tournament.add_agents(agents);

        // 4. create scheduler and communication channels
        let mut scheduler = TournamentScheduler::new(self.params.clone(), tournament);
        let (tx_result, rx_result) = mpsc::channel();
        let (tx_match, rx_match) = mpsc::channel();

        // 5. start match dispatcher (block waiting on match)
        let factory: F = self.factory.clone();
        let match_dispatcher = std::thread::spawn(move || {
            for match_settings in rx_match {
                let new_game = factory.new_game();
                let tx_result = tx_result.clone();
                std::thread::spawn(move || {
                    let result = run_match(match_settings, new_game);
                    tx_result.send(result).unwrap();
                });
            }
        });

        // 6. Init matches
        for m in scheduler.advance() {
            tx_match.send(m).unwrap();
        }

        // 7. main loop
        while !scheduler.is_finished() {
            let result = rx_result.recv().unwrap();
            for m in scheduler.on_result(result) {
                tx_match.send(m).unwrap();
            }
        }

        drop(tx_match); // should end match dispatcher
        match_dispatcher.join().unwrap();

        Ok(scheduler.final_scores())
    }
}
