use crate::games::{DummyFactory, RPSWrapper};
use server::tournament_strategy::{SinglePlayerTournament, SwissTournament};
use ::server::{constraints::ConstraintsBuilder, server::Evaluator};
use tracing_subscriber::{fmt, layer::{Context, Filter, SubscriberExt}, Layer, Registry};
use std::{str::FromStr, time::Duration};
use tracing::{Level, Metadata};

mod games;

struct CustomLevelFilter;
impl<S> Filter<S> for CustomLevelFilter {
fn enabled(&self,meta: &Metadata<'_>,_cx: &Context<'_,S>) -> bool {
        meta.level() == &Level::DEBUG
    }
}

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

    let reg = Registry::default().with(
        fmt::layer().event_format(format).with_filter(CustomLevelFilter)
    );

    let _ = tracing::subscriber::set_global_default(reg);
}

#[test]
fn launch_dummy() {
    init_logger();

    let params = ConstraintsBuilder::new().with_time_budget(Duration::from_secs(10)).with_total_cpu_count(2).build().unwrap();
    let evaluator = Evaluator::new(DummyFactory {}, params);
    let path = std::env::current_dir().unwrap().join("tests/dummy_agents");
    let tournament = SinglePlayerTournament::new(3);
    let scores = evaluator.evaluate(path.as_path(),tournament).unwrap();
    dbg!(scores);
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
    let scores = evaluator.evaluate(path.as_path(),tournament).unwrap();
    dbg!(scores);
}

#[test]
fn test_from_str() -> Result<(), String> {
    let _state = crate::games::PlayerState::from_str("0\n")?;
    Ok(())
}
