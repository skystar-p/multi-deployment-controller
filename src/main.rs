use std::{sync::Arc, time::Duration};

use futures_util::StreamExt;
use k8s_openapi::api::core::v1::Pod;
use kube::{
    Client, ResourceExt,
    runtime::{Controller, controller::Action},
};
use thiserror::Error;

mod crd;

#[derive(Error, Debug)]
enum Error {
    #[error("Kubernetes error: {0}")]
    KubeError(#[from] kube::Error),
}

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

struct Context {}

async fn reconcile(obj: Arc<Pod>, ctx: Arc<Context>) -> Result<Action, Error> {
    println!("Reconciling Pod: {}", obj.name_any());
    Ok(Action::await_change())
}

fn error_policy(_obj: Arc<Pod>, _error: &Error, _ctx: Arc<Context>) -> Action {
    Action::requeue(Duration::from_secs(5))
}
