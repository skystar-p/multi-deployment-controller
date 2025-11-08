use std::{collections::BTreeMap, sync::Arc, time::Duration};

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

const LABEL_SELECTOR_KEY: &str = "multi-deployment.skystar.dev/managed-by";

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
    let mut child_deployment = source.spec.children.get(&child_name).unwrap().clone();

    // update child deployment's selector
    child_deployment.spec.selector = source.spec.root_template.selector.clone();
    // ensure match_labels is initialized
    let child_match_labels = child_deployment
        .spec
        .selector
        .match_labels
        .get_or_insert_with(|| BTreeMap::new());
    // add unique label to match_labels
    child_match_labels.insert(
        LABEL_SELECTOR_KEY.to_string(),
        format!("{}-{}", source_name, child_name),
    );

    // update child deployment's template metadata labels
    let child_template_metadata = &mut child_deployment
        .spec
        .template
        .metadata
        .get_or_insert_with(|| ObjectMeta::default());

    // inherit labels from root template
    child_template_metadata.labels = source
        .spec
        .root_template
        .template
        .metadata
        .as_ref()
        .and_then(|m| m.labels.clone());

    // add unique label to template labels
    child_template_metadata
        .labels
        .get_or_insert_with(|| BTreeMap::new())
        .insert(
            LABEL_SELECTOR_KEY.to_string(),
            format!("{}-{}", source_name, child_name),
        );

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
