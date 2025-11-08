use std::{sync::Arc, time::Duration};

use k8s_openapi::api::apps::v1::Deployment;
use kube::{
    Resource, ResourceExt,
    api::{ObjectMeta, Patch, PatchParams},
    runtime::controller::Action,
};
use tracing::{error, info};

use crate::{
    crd::MultiDeployment,
    types::{Context, Error},
};

pub async fn reconcile(obj: Arc<MultiDeployment>, ctx: Arc<Context>) -> Result<Action, Error> {
    info!("Reconciling MultiDeployment: {}", obj.name_any());
    let _multi_deployments = ctx.multi_deployments.clone();
    let deployments = ctx.deployments.clone();

    for child_name in obj.spec.children.keys() {
        let deployment_data = create_owned_deployment(&obj, child_name.clone())?;
        let server_side = PatchParams::apply("multi-deployment-controller");

        // create or patch the Deployment
        info!("Reconciling Deployment: {}", deployment_data.name_any());
        deployments
            .patch(
                &deployment_data.name_any(),
                &server_side,
                &Patch::Apply(deployment_data),
            )
            .await?;
    }

    Ok(Action::await_change())
}

pub fn error_policy(_obj: Arc<MultiDeployment>, error: &Error, _ctx: Arc<Context>) -> Action {
    error!("Reconciliation error: {:?}", error);
    Action::requeue(Duration::from_secs(5))
}

fn create_owned_deployment(
    source: &MultiDeployment,
    child_name: String,
) -> Result<Deployment, Error> {
    let oref = source.controller_owner_ref(&()).unwrap();
    let source_name = source.name_any();
    let child_deployment = source.spec.children.get(&child_name).unwrap();

    // merge two deployment specs
    let mut root_spec = serde_json::to_value(&source.spec.root_template)?;
    let child_spec = serde_json::to_value(&child_deployment.spec)?;

    json_patch::merge(&mut root_spec, &child_spec);
    let root_spec = serde_json::from_value(root_spec)?;

    let deployment = Deployment {
        metadata: ObjectMeta {
            name: Some(format!("{}-{}", source_name, child_name)),
            owner_references: Some(vec![oref]),
            ..Default::default()
        },
        spec: Some(root_spec),

        ..Default::default()
    };

    Ok(deployment)
}
