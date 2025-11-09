use std::sync::Arc;

use futures_util::StreamExt;
use k8s_openapi::api::apps::v1::Deployment;
use kube::{Client, runtime::Controller};
use tracing::info;

use multi_deployment_controller::{
    controller::{error_policy, reconcile},
    crd::MultiDeployment,
    types::{Context, Error},
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let client = Client::try_default().await?;

    let multi_deployments = kube::Api::<MultiDeployment>::default_namespaced(client.clone());
    let deployments = kube::Api::<Deployment>::default_namespaced(client);
    let ctx = Context {
        multi_deployments: multi_deployments.clone(),
        deployments: deployments.clone(),
    };
    let context = Arc::new(ctx);

    info!("Starting MultiDeployment controller");
    Controller::new(multi_deployments, Default::default())
        .owns(deployments, Default::default())
        .run(reconcile, error_policy, context)
        .for_each(|_res| async move {})
        .await;

    Ok(())
}
