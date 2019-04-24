mod behaviors;

use structopt::StructOpt;

use rust_ldn_demo::shared::opt::Opt;
use rust_ldn_demo::shared::connection::get_connection;
use rust_ldn_demo::shared::behavior::BehaviorManager;
use crate::behaviors::count_trees::CountTreesBehaviour;

const WORKER_TYPE: &str = "RustWorker";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    let mut worker_connection = get_connection(WORKER_TYPE, &opt)?;

    let mut manager = BehaviorManager::new();
    manager.register_behaviour(CountTreesBehaviour {});

    loop {
        manager.tick(&mut worker_connection);
    }

    Ok(())
}
