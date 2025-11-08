use std::{sync::Arc, time::Duration};

use kube::{ResourceExt, runtime::controller::Action};

use crate::{crd::MultiDeployment, types::{Context, Error}};

pub async fn reconcile(obj: Arc<MultiDeployment>, _ctx: Arc<Context>) -> Result<Action, Error> {
    println!("Reconciling Pod: {}", obj.name_any());
    Ok(Action::await_change())
}

pub fn error_policy(_obj: Arc<MultiDeployment>, _error: &Error, _ctx: Arc<Context>) -> Action {
    Action::requeue(Duration::from_secs(5))
}
