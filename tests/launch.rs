use crate::games::{DummyFactory, RPSWrapper};

use ai_tournament::prelude::*;
use std::{str::FromStr, time::Duration};
use time::format_description;
use tracing::{Level, Metadata};
use tracing_subscriber::{
    fmt,
    layer::{Context, Filter, SubscriberExt},
    FmtSubscriber, Layer, Registry,
};

mod games;

struct CustomLevelFilter;
impl<S> Filter<S> for CustomLevelFilter {
    fn enabled(&self, meta: &Metadata<'_>, _cx: &Context<'_, S>) -> bool {
        meta.level() == &Level::DEBUG
    }
}

fn init_as_file_logger() {
    let local_offset = time::UtcOffset::current_local_offset().unwrap();
    let timer = tracing_subscriber::fmt::time::OffsetTime::new(
        local_offset,
        format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap(),
    );

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_ansi(false)
        .with_timer(timer)
        // .with_writer(writer)
        .finish();

    let _ = tracing::subscriber::set_global_default(subscriber);
}

#[allow(dead_code)]
fn init_debug_logger() {
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
    let verbose_mode = true;

    if !verbose_mode {
        init_as_file_logger();
    }

    let params = ConstraintsBuilder::new()
        .with_time_budget(Duration::from_secs(10))
        .with_total_cpu_count(2)
        .build()
        .unwrap();

    let config = Configuration::new()
        .with_verbose(verbose_mode)
        // .with_log("/tmp/test_dummy")
        .with_allow_uncontained(true);

    let evaluator = Evaluator::new(DummyFactory {}, config, params);
    let path = "tests/dummy_agents";
    let tournament = SinglePlayerTournament::new(3);
    let scores = evaluator.evaluate(&path, tournament).unwrap();
    dbg!(scores);
}

#[test]
fn launch_rock_paper_scissors() {
    let verbose_mode = true;
    if !verbose_mode {
        init_as_file_logger();
    }

    let params = ConstraintsBuilder::new()
        .with_time_budget(Duration::from_secs(10))
        .build()
        .unwrap();

    let config = Configuration::new()
        .with_test_all_configs(true)
        .with_debug_agent_stderr(false)
        // .with_log("/tmp/test_rps")
        .with_verbose(verbose_mode);

    let evaluator = Evaluator::new(RPSWrapper::default(), config, params);
    let path = "tests/rock_paper_scissors_agents";
    let tournament = SwissTournament::with_auto_rounds(8);
    let (scores, failures) = evaluator.evaluate(&path, tournament).unwrap();
    println!("Working agents:");
    for (name, score) in scores.iter() {
        println!("{name}: {score}");
    }
    println!("Non-compiling agents");
    for (name, error) in failures.iter() {
        println!("{name}: {error}");
    }
    // dbg!(scores);
}

#[test]
fn test_from_str() -> Result<(), String> {
    let _state = crate::games::PlayerState::from_str("0\n")?;
    Ok(())
}
