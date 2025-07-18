use crate::agent_compiler;
use crate::constraints::Constraints;
use crate::match_runner::{run_match, MatchSettings, RunnerResult};
use crate::tournament::TournamentScheduler;
use crate::tournament_strategy::TournamentStrategy;

pub use agent_interface::{anyhow, game_info::GameInfo, Game, GameFactory};
use anyhow::bail;
use std::collections::HashMap;
use std::io::Write;
use std::str::FromStr;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};

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

    pub fn evaluate<T: TournamentStrategy>(
        &self,
        directory: &std::path::Path,
        mut tournament: T,
    ) -> anyhow::Result<HashMap<String, T::FinalScore>>
    where
        T::FinalScore: 'static,
    {
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
        let (tx_match, rx_match) = mpsc::channel::<MatchSettings>();

        // 5. start match dispatcher (block waiting on match)
        let factory: F = self.factory.clone();
        let match_dispatcher =
            std::thread::spawn(move || match_dispatcher(rx_match, tx_result, factory));

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

        let mapped_score = scheduler
            .final_scores()
            .into_iter()
            .map(|(agent, score)| (agent.name.clone(), score))
            .collect::<HashMap<_, _>>();

        Ok(mapped_score)
    }
}

fn match_dispatcher<F, G>(
    rx_match: Receiver<MatchSettings>,
    tx_result: Sender<RunnerResult>,
    factory: F,
) where
    F: GameFactory<G>,
    G: Game,
    F: Clone + Send + 'static,
    G::State: FromStr + ToString,
    G::Action: FromStr + ToString,
    G: 'static + Send,
{
    // only for printing purpose
    let running_matches = Arc::new(Mutex::new(Vec::<String>::new()));

    // hide cursor + disable line wrapping
    print!("\x1b[?25l\x1b[?7l");

    for match_settings in rx_match {
        let string = match_settings.to_string();
        let new_game = factory.new_game();
        let tx_result = tx_result.clone();
        let c_mutex = running_matches.clone();
        add_match(&string, &*running_matches);
        std::thread::spawn(move || {
            let result = run_match(match_settings, new_game);
            remove_match(&string, &*c_mutex, &result);
            tx_result.send(result).unwrap();
        });
    }

    // unhide the cursor at the end + re-unable line wrapping
    print!("\x1b[?25h\x1b[?7h");

    fn add_match(match_string: &String, running_matches: &Mutex<Vec<String>>) {
        let mut running = running_matches.lock().expect("mutex poisoning");
        running.push(match_string.clone());
        print_running_matches(&running);
    }

    fn remove_match(
        match_string: &String,
        running_matches: &Mutex<Vec<String>>,
        result: &RunnerResult,
    ) {
        let mut running = running_matches.lock().expect("mutex poisoning");
        let pos = running
            .iter()
            .position(|s| *s == *match_string)
            .expect("running match not found");
        running.remove(pos);
        print_runner_result(&match_string, &result);
        print_running_matches(&running);
    }
    fn print_runner_result(match_string: &String, result: &RunnerResult) {
        let mut ordered_scores = Vec::new();
        for name in match_string[1..match_string.len() - 1].split(" VS ") {
            let res = result
                .results
                .iter()
                .find_map(|a| if a.0.name == name { Some(a.1) } else { None })
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
            "\x1b[2K\x1b[32m{}: \x1b[39m{} \x1b[31m{}\x1b[39m\x1b[0G",
            match_string, ordered_scores, result.errors
        );
    }
    fn print_running_matches(running: &Vec<String>) {
        // clear, green, default, start of line
        print!(
            "\x1b[2K\x1b[32mRunning...:\x1b[39m {}\x1b[0G",
            running.join(", ")
        );
        let _ = std::io::stdout().flush();
    }
}
