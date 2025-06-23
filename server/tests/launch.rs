use server::server;
use ::server::server::Evaluator;

use crate::games::DummyFactory;

mod games;

#[test]
fn launch_dummy() {
    let params= server::SystemParams::new(server::MaxMemory::Auto, server::AvailableCPUs::Auto);
    let evaluator = Evaluator::new(DummyFactory {}, params);
    let path = std::env::current_dir().unwrap().join("tests/dummy_agents");
    let _ = evaluator.evaluate(path.as_path()).unwrap();
}