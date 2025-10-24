use tokio_graceful_shutdown::{SubsystemBuilder, SubsystemHandle};
use tracing::{error, info};

use shared::error::CommonError;

use crate::commands::dev::restate::{start_restate_deployment, RestateServerParams};
use crate::utils::restate::deploy::DeploymentType;

pub struct StartDeploymentSubsystemParams<'a> {
    pub restate_params: &'a RestateServerParams,
    pub deployment_type: DeploymentType,
    pub service_path: String,
}

/// Starts the Restate deployment registration subsystem
pub fn start_deployment_subsystem(
    subsys: &SubsystemHandle,
    params: StartDeploymentSubsystemParams,
) {
    let restate_params = params.restate_params.clone();
    let deployment_type = params.deployment_type.clone();
    let service_path = params.service_path.clone();

    subsys.start(SubsystemBuilder::new(
        "restate-deployment",
        move |subsys: SubsystemHandle| async move {
            tokio::select! {
                _ = subsys.on_shutdown_requested() => {
                    info!("system shutdown requested");
                    info!("Restate deployment shutdown complete");
                }
                result = start_restate_deployment(&restate_params, deployment_type, service_path) => {
                    if let Err(e) = result {
                        error!("Restate deployment stopped unexpectedly: {:?}", e);
                        subsys.request_shutdown();
                    }
                    info!("Restate deployment completed");
                }
            }

            Ok::<(), CommonError>(())
        },
    ));
}
