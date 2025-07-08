use crate::games::{DummyFactory, RPSWrapper};
use server::tournament_strategy::{SingleplayerTournament, SwissTournament};
use ::server::{constraints::ConstraintsBuilder, server::Evaluator};
use std::{str::FromStr, time::Duration};
use tracing::Level;

mod games;

fn init_logger() {
    let format = tracing_subscriber::fmt::format()
        .without_time()
        .with_ansi(true)
        .with_level(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
    ;

    let _ = tracing_subscriber::fmt()
        .event_format(format)
        .with_max_level(Level::TRACE)
        .try_init();
}

#[test]
fn launch_dummy() {
    init_logger();

    let params = ConstraintsBuilder::new().with_time_budget(Duration::from_secs(10)).with_total_cpu_count(2).build().unwrap();
    let evaluator = Evaluator::new(DummyFactory {}, params);
    let path = std::env::current_dir().unwrap().join("tests/dummy_agents");
    let tournament = SingleplayerTournament::new(3);
    let _ = evaluator.evaluate(path.as_path(),tournament).unwrap();
}

#[test]
fn launch_rock_paper_scissors() {
    init_logger();

    let params = ConstraintsBuilder::new().with_time_budget(Duration::from_secs(10)).build().unwrap();
    let evaluator = Evaluator::new(RPSWrapper::default(), params);
    let path = std::env::current_dir()
        .unwrap()
        .join("tests/rock_paper_scissors_agents");
    let tournament = SwissTournament::new(0);
    let _ = evaluator.evaluate(path.as_path(),tournament).unwrap();
}

#[test]
fn test_from_str() -> Result<(), String> {
    let _state = crate::games::PlayerState::from_str("0\n")?;
    Ok(())
}
