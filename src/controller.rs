use std::{sync::Arc, time::Duration};

use k8s_openapi::api::apps::v1::Deployment;
use kube::{Resource, ResourceExt, api::ObjectMeta, runtime::controller::Action};

use crate::{
    crd::{ChildDeployment, MultiDeployment},
    types::{Context, Error},
};

pub async fn reconcile(obj: Arc<MultiDeployment>, _ctx: Arc<Context>) -> Result<Action, Error> {
    println!("Reconciling Pod: {}", obj.name_any());
    Ok(Action::await_change())
}

pub fn error_policy(_obj: Arc<MultiDeployment>, _error: &Error, _ctx: Arc<Context>) -> Action {
    Action::requeue(Duration::from_secs(5))
}

fn create_owned_deployment(source: &MultiDeployment, child_name: String) -> Result<Deployment, Error> {
    let oref = source.controller_owner_ref(&()).unwrap();
    let source_name = source.name_any();
    let child_deployment = source.spec.children.get(&child_name).unwrap();

    let deployment = Deployment {
        metadata: ObjectMeta {
            name: Some(format!("{}-{}", source_name, child_name)),
            owner_references: Some(vec![oref]),
            ..Default::default()
        },

        ..Default::default()
    };

    // 1. Get spec from root_template
    // 2. Merge with spec from child_deployment

    todo!()
}
