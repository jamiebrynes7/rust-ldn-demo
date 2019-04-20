use std::{
    env,
    path::PathBuf
};
use spatialos_sdk::worker::snapshot::SnapshotOutputStream;
use structopt::StructOpt;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt :Opt = Opt::from_args();

    let mut target_file = env::current_dir()?;
    target_file.push(opt.snapshot_path);

    let _stream = SnapshotOutputStream::new(target_file)?;

    Ok(())
}

#[derive(StructOpt, Debug)]
#[structopt(name = "generate-snapshot")]
struct Opt {

    #[structopt(short = "p", long="snapshot-path")]
    snapshot_path: PathBuf
}
