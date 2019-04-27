use crate::shared::opt::{Command, Opt};
use futures::future::Future;
use spatialos_sdk::worker::connection::WorkerConnection;
use spatialos_sdk::worker::constants::LOCATOR_HOSTNAME;
use spatialos_sdk::worker::constants::RECEPTIONIST_PORT;
use spatialos_sdk::worker::locator::Locator;
use spatialos_sdk::worker::locator::{LocatorCredentials, LocatorParameters};
use spatialos_sdk::worker::parameters::ConnectionParameters;

pub fn get_connection(worker_type: &str, options: &Opt) -> Result<WorkerConnection, String> {
    let worker_id = match options.worker_id {
        Some(ref id) => id.clone(),
        None => format!("{}-{}", worker_type, "TODO"),
    };

    let connection_future = match &options.command {
        Command::Receptionist {
            host,
            port,
            connect_with_external_ip,
        } => {
            let params = ConnectionParameters::new(worker_type)
                .using_tcp()
                .using_external_ip(*connect_with_external_ip)
                .enable_internal_serialization();

            let host = match host {
                Some(ref h) => h.clone(),
                None => "127.0.0.1".to_owned(),
            };

            let port = port.unwrap_or(RECEPTIONIST_PORT);

            WorkerConnection::connect_receptionist_async(&worker_id, &host, port, &params)
        }
        Command::Locator {
            token,
            project_name,
        } => {
            let params = ConnectionParameters::new(worker_type)
                .using_raknet()
                .using_external_ip(true)
                .enable_internal_serialization();

            let locator_params =
                LocatorParameters::new(project_name, LocatorCredentials::login_token(token));
            let locator = Locator::new(LOCATOR_HOSTNAME, &locator_params);

            let deployment_list_future = locator.get_deployment_list_async();
            let deployment_list = deployment_list_future.wait()?;

            if deployment_list.is_empty() {
                return Err("No deployments could be found".to_owned())?;
            }

            let deployment = &deployment_list[0].deployment_name;

            WorkerConnection::connect_locator_async(
                &locator,
                deployment,
                &params,
                queue_status_callback,
            )
        }
    };

    connection_future.wait()
}

fn queue_status_callback(_queue_status: &Result<u32, String>) -> bool {
    true
}
