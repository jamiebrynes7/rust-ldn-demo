mod behaviors;

use structopt::StructOpt;

use crate::behaviors::count_trees::TrackTreesBehaviour;
use crate::behaviors::lumberjacks::LumberjackBehavior;
use rust_ldn_demo::shared::connection::get_connection;
use rust_ldn_demo::shared::fps::{FpsTracker, FpsLimiter};
use rust_ldn_demo::shared::opt::Opt;
use spatialos_sdk::worker::connection::Connection;
use spatialos_sdk::worker::op::WorkerOp;
use spatialos_sdk::worker::view::View;
use crate::behaviors::hq::HqBehaviour;

const WORKER_TYPE: &str = "RustWorker";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    let mut connection = get_connection(WORKER_TYPE, &opt)?;
    let mut view = View::new();
    let mut fps_tracker = FpsTracker::new(10);
    let mut fps_limiter = FpsLimiter::new(60.0);

    // Behaviours
    let mut trees = TrackTreesBehaviour::new();
    let mut lumberjacks = LumberjackBehavior::new();
    let mut hqs = HqBehaviour::new();

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
                break;
            }
        }

        trees.tick(&view, &mut connection);
        lumberjacks.tick(&view, &mut connection, &trees);
        hqs.tick(&view, &mut connection);

        let frame_time = fps_tracker.tick(&mut connection);
        fps_limiter.tick(frame_time);
    }

    Ok(())
}
