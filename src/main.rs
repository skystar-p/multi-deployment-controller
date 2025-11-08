use std::sync::Arc;

use futures_util::StreamExt;
use k8s_openapi::api::apps::v1::Deployment;
use kube::{Client, runtime::Controller};

use multi_deployment::{
    controller::{error_policy, reconcile},
    crd::MultiDeployment,
    types::{Context, Error},
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = Client::try_default().await?;

    let multi_deployments = kube::Api::<MultiDeployment>::default_namespaced(client.clone());
    let deployments = kube::Api::<Deployment>::default_namespaced(client);
    let context = Arc::new(Context {});

    Controller::new(multi_deployments, Default::default())
        .owns(deployments, Default::default())
        .run(reconcile, error_policy, context)
        .for_each(|_res| async move {})
        .await;

    Ok(())
}
