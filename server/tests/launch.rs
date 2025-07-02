use crate::games::{DummyFactory, RPSWrapper};
use ::server::{constraints::ConstraintsBuilder, server::Evaluator};
use std::str::FromStr;
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
        .with_max_level(Level::INFO)
        .try_init();
}

#[test]
fn launch_dummy() {
    init_logger();

    let params = ConstraintsBuilder::new().build().unwrap();
    let evaluator = Evaluator::new(DummyFactory {}, params);
    let path = std::env::current_dir().unwrap().join("tests/dummy_agents");
    let _ = evaluator.evaluate(path.as_path()).unwrap();
}

#[test]
fn launch_rock_paper_scissors() {
    init_logger();

    let params = ConstraintsBuilder::new().build().unwrap();
    let evaluator = Evaluator::new(RPSWrapper::default(), params);
    let path = std::env::current_dir()
        .unwrap()
        .join("tests/rock_paper_scissors_agents");
    let _ = evaluator.evaluate(path.as_path()).unwrap();
}

#[test]
fn test_from_str() -> Result<(), String> {
    let _state = crate::games::PlayerState::from_str("0\n")?;
    Ok(())
}
