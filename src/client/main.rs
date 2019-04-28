mod behaviors;

use structopt::StructOpt;

use rust_ldn_demo::shared::connection::get_connection;
use rust_ldn_demo::shared::fps::{FpsTracker, FpsLimiter};
use rust_ldn_demo::shared::opt::Opt;
use spatialos_sdk::worker::connection::Connection;
use spatialos_sdk::worker::op::WorkerOp;
use spatialos_sdk::worker::view::View;
use rust_ldn_demo::shared::templates;
use rust_ldn_demo::shared::generated::improbable::Vector3d;
use rust_ldn_demo::shared::utils::get_random_coords;
use crate::behaviors::trees::TrackTreesBehaviour;
use crate::behaviors::wizards::WizardBehavior;

const WORKER_TYPE: &str = "RustClient";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    let mut connection = get_connection(WORKER_TYPE, &opt)?;
    let mut view = View::new();
    let mut fps_tracker = FpsTracker::new(10);
    let mut fps_limiter = FpsLimiter::new(60.0);

    let center = Vector3d { x: 0.0, y: 0.0 , z: 0.0};
    let mut rng = rand::thread_rng();

    let num_wizards = if opt.evil { 5 } else { 10 };

    for i in 0..num_wizards {
        connection.send_create_entity_request(templates::wizard(&get_random_coords(&center, 500, &mut rng), opt.evil, connection.get_worker_id())?, None, None);
    }

    let mut trees = TrackTreesBehaviour::new();
    let mut wizards = WizardBehavior::new();

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
        wizards.tick(&view, &mut connection, &trees);

        let frame_time = fps_tracker.tick(&mut connection);
        fps_limiter.tick(frame_time);
    }

    Ok(())
}
