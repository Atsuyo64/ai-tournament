use crate::games::{DummyFactory, RPSWrapper};
use ::server::server::Evaluator;
use server::server;
use std::{io::Write, str::FromStr};

mod games;

fn init_logger() {
    let env = env_logger::Env::default()
        .filter_or("RUST_LOG_LEVEL", "debug")
        .write_style_or("RUST_LOG_STYLE", "auto");

    let _ = env_logger::Builder::from_env(env)
        .is_test(false)
        .format_timestamp(None)
        .format(|buf, record| {
            let style = buf.default_level_style(record.level());
            let tid= std::thread::current().id();
            writeln!(
                buf,
                "[{style}{:<5}{style:#} {:?} {}] {}",
                record.level(),
                tid,
                record.module_path().unwrap_or(""),
                record.args()
            )
        })
        .try_init(); //NOTE: might change is_test to false
}

#[test]
fn launch_dummy() {
    init_logger();

    let params = server::SystemParams::new(server::MaxMemory::Auto, server::AvailableCPUs::Auto);
    let evaluator = Evaluator::new(DummyFactory {}, params);
    let path = std::env::current_dir().unwrap().join("tests/dummy_agents");
    let _ = evaluator.evaluate(path.as_path()).unwrap();
}

#[test]
fn launch_rock_paper_scissors() {
    init_logger();

    let params  = server::SystemParams::new(server::MaxMemory::Auto, server::AvailableCPUs::Auto);
    let evaluator = Evaluator::new(RPSWrapper::default(), params);
    let path = std::env::current_dir().unwrap().join("tests/rock_paper_scissors_agents");
    let _ = evaluator.evaluate(path.as_path()).unwrap();
}

#[test]
fn test_from_str() -> Result<(),String> {
    let _state = crate::games::PlayerState::from_str("0\n")?;
    Ok(())
}