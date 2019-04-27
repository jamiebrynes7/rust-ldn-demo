use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "rust-ldn-demo", about = "A SpatialOS worker written in Rust.")]
pub struct Opt {
    #[structopt(name = "WORKER_ID", long = "worker-id", short = "i")]
    pub worker_id: Option<String>,

    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(name = "receptionist")]
    Receptionist {
        #[structopt(long, short)]
        connect_with_external_ip: bool,

        #[structopt(long, short)]
        host: Option<String>,

        #[structopt(long, short)]
        port: Option<u16>,
    },

    #[structopt(name = "locator")]
    Locator {
        #[structopt(name = "LOCATOR_TOKEN", long = "locator-token", short = "t")]
        token: String,

        #[structopt(name = "PROJECT_NAME", long = "project-name", short = "n")]
        project_name: String,
    },
}
