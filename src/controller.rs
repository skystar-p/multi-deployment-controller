use std::{sync::Arc, time::Duration};

use k8s_openapi::api::core::v1::Pod;
use kube::{ResourceExt, runtime::controller::Action};

use crate::types::{Context, Error};

pub async fn reconcile(obj: Arc<Pod>, ctx: Arc<Context>) -> Result<Action, Error> {
    println!("Reconciling Pod: {}", obj.name_any());
    Ok(Action::await_change())
}

pub fn error_policy(_obj: Arc<Pod>, _error: &Error, _ctx: Arc<Context>) -> Action {
    Action::requeue(Duration::from_secs(5))
}
