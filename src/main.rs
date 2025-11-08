use std::sync::Arc;

use futures_util::StreamExt;
use k8s_openapi::api::core::v1::Pod;
use kube::{Client, runtime::Controller};

use multi_deployment::{
    controller::{error_policy, reconcile},
    types::{Context, Error},
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = Client::try_default().await?;

    let pods = kube::Api::<Pod>::all(client);
    let context = Arc::new(Context {});

    Controller::new(pods, Default::default())
        .run(reconcile, error_policy, context)
        .for_each(|res| async move {})
        .await;

    Ok(())
}
