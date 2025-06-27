use crate::games::DummyFactory;
use ::server::server::Evaluator;
use server::server;
use std::io::Write;

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
        .try_init(); //NOTE: migh change is_test to false
}

//[WARN  server::match_runner] Error c

#[test]
fn launch_dummy() {
    init_logger();

    let params = server::SystemParams::new(server::MaxMemory::Auto, server::AvailableCPUs::Auto);
    let evaluator = Evaluator::new(DummyFactory {}, params);
    let path = std::env::current_dir().unwrap().join("tests/dummy_agents");
    let _ = evaluator.evaluate(path.as_path()).unwrap();
}
