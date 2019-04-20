use std::{
    env,
    iter::from_fn,
    path::PathBuf
};
use spatialos_sdk::worker::snapshot::SnapshotOutputStream;
use structopt::StructOpt;

use rust_ldn_demo::shared::{
    generated::improbable::Vector3d,
    templates
};
use rand::Rng;
use rand::prelude::ThreadRng;
use spatialos_sdk::worker::EntityId;

const NUM_TREES: i32 = 2000;
const NUM_CLUSTERS: i32 = 20;
const CLUSTER_RADIUS: i32 = 150;
const WORLD_RADIUS: i32 = 500;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt :Opt = Opt::from_args();
    let mut rng = rand::thread_rng();

    let mut target_file = env::current_dir()?;
    target_file.push(opt.snapshot_path);

    let mut stream = SnapshotOutputStream::new(target_file)?;

    let clusters = generate_clusters(&mut rng);

    let mut entity_count = 1;

    for cluster in &clusters {
        for _i in 0..(NUM_TREES / NUM_CLUSTERS) {
            add_tree(&mut stream, entity_count, cluster, &mut rng)?;
            entity_count += 1;
        }
    }

    Ok(())
}

// TODO: Fix error where write_entity isn't a &mut method.
fn add_tree(stream: &mut SnapshotOutputStream, id: i64, center: &Vector3d, rng: &mut ThreadRng) -> Result<(), Box<dyn std::error::Error>> {
    let mut position = center.clone();

    let angle_adjustment = rng.gen_range(0.0, 2.0 * std::f64::consts::PI);
    let (z_component, x_component) = angle_adjustment.sin_cos();

    position.x += x_component * rng.gen_range(0.0, CLUSTER_RADIUS as f64);
    position.z += z_component * rng.gen_range(0.0, CLUSTER_RADIUS as f64);

    stream.write_entity(EntityId::new(id), &templates::tree(&position)?)?;

    Ok(())
}

fn generate_clusters(rng: &mut ThreadRng) -> Vec<Vector3d> {
    let mut count = 0;

    let max = (WORLD_RADIUS - CLUSTER_RADIUS) as f64;
    let min = -max;

    from_fn(move || {
        if count == NUM_CLUSTERS {
            return None;
        }

        count += 1;

        Some(Vector3d {
            x: rng.gen_range(min, max),
            y: 0.0,
            z: rng.gen_range(min, max)
        })
    }).collect()
}

#[derive(StructOpt, Debug)]
#[structopt(name = "generate-snapshot")]
struct Opt {

    #[structopt(short = "p", long="snapshot-path")]
    snapshot_path: PathBuf
}
