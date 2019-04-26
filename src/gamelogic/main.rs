mod behaviors;

use structopt::StructOpt;

use rust_ldn_demo::shared::opt::Opt;
use rust_ldn_demo::shared::connection::get_connection;
use crate::behaviors::count_trees::TrackTreesBehaviour;
use spatialos_sdk::worker::view::View;
use spatialos_sdk::worker::op::WorkerOp;
use crate::behaviors::lumberjacks::LumberjackBehavior;
use spatialos_sdk::worker::connection::Connection;

const WORKER_TYPE: &str = "RustWorker";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    let mut connection = get_connection(WORKER_TYPE, &opt)?;
    let mut view = View::new();

    // Behaviours
    let mut trees = TrackTreesBehaviour::new();
    let mut lumberjacks = LumberjackBehavior::new();

    loop {
        view.clear_transient_data();

        let mut in_critical_section = false;

        loop {
            let ops = connection.get_op_list(0);
            view.process_ops(&ops);

            for op in ops.iter() {
                match op {
                    WorkerOp::CriticalSection(_) => in_critical_section = !in_critical_section,
                    _ => {}
                }
            }

            if !in_critical_section {
                break
            }
        }

        trees.tick(&view, &mut connection);
        lumberjacks.tick(&view, &mut connection, &trees);
    }

    Ok(())
}
