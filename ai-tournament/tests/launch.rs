use crate::games::{DummyFactory, RPSWrapper};

use ai_tournament::prelude::*;
use std::{str::FromStr, time::Duration};
use tracing::{Level, Metadata};
use tracing_subscriber::{
    fmt,
    layer::{Context, Filter, SubscriberExt},
    Layer, Registry,
};

mod games;

struct CustomLevelFilter;
impl<S> Filter<S> for CustomLevelFilter {
    fn enabled(&self, meta: &Metadata<'_>, _cx: &Context<'_, S>) -> bool {
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
        .with_target(false);

    let reg = Registry::default().with(
        fmt::layer()
            .event_format(format)
            .with_filter(CustomLevelFilter),
    );

    let _ = tracing::subscriber::set_global_default(reg);
}

#[test]
fn launch_dummy() {
    init_logger();

    let params = ConstraintsBuilder::new()
        .with_time_budget(Duration::from_secs(10))
        .with_total_cpu_count(2)
        .build()
        .unwrap();

    let config = Configuration::new()
        .with_verbose(true)
        .with_allow_uncontained(true);

    let evaluator = Evaluator::new(DummyFactory {}, config, params);
    let path = "tests/dummy_agents";
    let tournament = SinglePlayerTournament::new(3);
    let scores = evaluator.evaluate(&path, tournament).unwrap();
    dbg!(scores);
}

#[test]
fn launch_rock_paper_scissors() {
    init_logger();

    let params = ConstraintsBuilder::new()
        .with_time_budget(Duration::from_secs(10))
        .build()
        .unwrap();

    let config = Configuration::new()
        .with_test_all_configs(true)
        .with_debug_agent_stderr(false);

    let evaluator = Evaluator::new(RPSWrapper::default(), config, params);
    let path = "tests/rock_paper_scissors_agents";
    let tournament = SwissTournament::with_auto_rounds(8);
    let scores = evaluator.evaluate(&path, tournament).unwrap();
    for (name, score) in scores.into_iter() {
        println!("{name}: {score}");
    }
    // dbg!(scores);
}

#[test]
fn test_from_str() -> Result<(), String> {
    let _state = crate::games::PlayerState::from_str("0\n")?;
    Ok(())
}
