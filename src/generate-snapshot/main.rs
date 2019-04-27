use spatialos_sdk::worker::snapshot::SnapshotOutputStream;
use std::{env, iter::from_fn, path::PathBuf};
use structopt::StructOpt;

use rand::prelude::ThreadRng;
use rand::Rng;
use rust_ldn_demo::shared::{generated::improbable::Vector3d, templates};
use spatialos_sdk::worker::entity::Entity;
use spatialos_sdk::worker::EntityId;
use std::env::current_dir;
use std::path::Path;

const NUM_TREES: i32 = 2000;
const NUM_CLUSTERS: i32 = 20;
const TREE_CLUSTER_RADIUS: i32 = 150;
const WORLD_RADIUS: i32 = 500;

const NUM_LUMBERJACKS: i32 = 10;
const LUMBERJACK_CLUSTER_RADIUS: i32 = 15;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt: Opt = Opt::from_args();
    let mut rng = rand::thread_rng();

    let mut target_file = env::current_dir()?;
    target_file.push(opt.snapshot_path);

    let mut snapshot = Snapshot::new(target_file)?;

    let clusters = generate_clusters(&mut rng);

    for cluster in &clusters {
        for _i in 0..(NUM_TREES / NUM_CLUSTERS) {
            snapshot.write(&templates::tree(&get_random_coords(
                cluster,
                TREE_CLUSTER_RADIUS,
                &mut rng,
            ))?)?;
        }
    }

    let hq_coord = (WORLD_RADIUS - 100) as f64;
    generate_hq(&mut snapshot, hq_coord, &mut rng)?;
    let hq_coord = -hq_coord;
    generate_hq(&mut snapshot, hq_coord, &mut rng)?;

    Ok(())
}

fn generate_hq(
    snapshot: &mut Snapshot,
    coord: f64,
    rng: &mut ThreadRng,
) -> Result<(), Box<dyn std::error::Error>> {
    let hq_position = Vector3d {
        x: coord,
        y: 0.0,
        z: coord,
    };
    snapshot.write(&templates::headquarters(&hq_position)?)?;

    for _i in 0..NUM_LUMBERJACKS {
        snapshot.write(&templates::lumberjack(&get_random_coords(
            &hq_position,
            LUMBERJACK_CLUSTER_RADIUS,
            rng,
        ))?)?;
    }

    Ok(())
}

fn generate_clusters(rng: &mut ThreadRng) -> Vec<Vector3d> {
    let mut count = 0;

    let max = (WORLD_RADIUS - TREE_CLUSTER_RADIUS) as f64;
    let min = -max;

    from_fn(move || {
        if count == NUM_CLUSTERS {
            return None;
        }

        count += 1;

        Some(Vector3d {
            x: rng.gen_range(min, max),
            y: 0.0,
            z: rng.gen_range(min, max),
        })
    })
    .collect()
}

fn get_random_coords(center: &Vector3d, radius: i32, rng: &mut ThreadRng) -> Vector3d {
    let mut position = center.clone();

    let angle_adjustment = rng.gen_range(0.0, 2.0 * std::f64::consts::PI);
    let (z_component, x_component) = angle_adjustment.sin_cos();

    position.x += x_component * rng.gen_range(0.0, radius as f64);
    position.z += z_component * rng.gen_range(0.0, radius as f64);

    position
}

#[derive(StructOpt, Debug)]
#[structopt(name = "generate-snapshot")]
struct Opt {
    #[structopt(short = "p", long = "snapshot-path")]
    snapshot_path: PathBuf,
}

struct Snapshot {
    stream: SnapshotOutputStream,
    current_id: i64,
}

impl Snapshot {
    pub fn new<P: AsRef<Path>>(filename: P) -> Result<Self, String> {
        Ok(Snapshot {
            stream: SnapshotOutputStream::new(filename)?,
            current_id: 1,
        })
    }

    pub fn write(&mut self, entity: &Entity) -> Result<(), Box<dyn std::error::Error>> {
        self.stream
            .write_entity(EntityId::new(self.current_id), entity)?;
        self.current_id += 1;

        Ok(())
    }
}
