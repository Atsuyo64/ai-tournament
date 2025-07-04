use crate::agent_compiler;
use crate::constraints::Constraints;
use crate::match_runner::run_match;
use crate::tournament::{Scores, TournamentScheduler};
use crate::tournament_strategy::SwissStrategy;

use agent_interface::{Game, GameFactory};
use anyhow::anyhow;
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
    F : Clone + Send + 'static,
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

    pub fn evaluate(&self, directory: &std::path::Path) -> anyhow::Result<Scores> {
        // 1. get agents name & code in *directory*
        if !directory.is_dir() {
            return Err(anyhow!("{directory:?} is not a directory"));
        }

        // 2. try to compile each one of them
        let agents = agent_compiler::compile_all_agents(directory);

        let _game_info = self.factory.new_game().get_game_info();
        //FIXME: deffering choice responsability to caller ?
        // Caller whould choose the tournament strategy ? (and could possibly "pipe" them ?)

        // 3. create an tournament of some sort (depending of game_type) for remaining ones
        let max_rounds = 4; /* ceil(log2(num_players)) ? */
        let strategy = SwissStrategy::new(agents, max_rounds);
        let mut tournament = TournamentScheduler::new(self.params.clone(),strategy);

        let (tx_result, rx_result) = mpsc::channel();
        let (tx_match, rx_match) = mpsc::channel();

        // 4. start match dispatcher (block waiting on match)
        let factory: F = self.factory.clone();
        std::thread::spawn(move || {
            for match_settings in rx_match {
                let new_game = factory.new_game();
                let tx_result = tx_result.clone();
                std::thread::spawn(move || {
                    let result = run_match(match_settings,new_game);
                    tx_result.send(result).unwrap();
                });
            }
        });

        // 5. Init matches
        for m in tournament.tick() {
            tx_match.send(m).unwrap();
        }

        // 6. main loop
        while !tournament.is_finished() {
            let result = rx_result.recv().unwrap();
            for m in tournament.on_result(result) {
                tx_match.send(m).unwrap();
            }
        }

        Ok(tournament.final_scores())

        // let mut tournament_maker = TournamentMaker::new(agents, self.params.clone(), game_info);

        // // 4. run tournament
        // while let Some(match_settings) = tournament_maker.next() {
        //     let game = self.factory.new_game();
        //     std::thread::spawn(move || run_match(match_settings, game));
        //     /* should not have to join threads */
        // }

        // Ok(tournament_maker.get_final_scores())
    }
}
