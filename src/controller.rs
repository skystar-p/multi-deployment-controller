use std::{collections::BTreeMap, sync::Arc, time::Duration};

use k8s_openapi::api::{
    apps::v1::{Deployment, DeploymentSpec},
    core::v1::PodTemplateSpec,
};
use kube::{
    Resource, ResourceExt,
    api::{ObjectMeta, Patch, PatchParams},
    runtime::controller::Action,
};
use tracing::{error, info};

use crate::{
    crd::MultiDeployment,
    types::{Context, Error},
    utils,
};

const CONTROLLER_NAME: &str = "multi-deployment-controller";
const LABEL_SELECTOR_KEY: &str = "multi-deployment.skystar.dev/managed-by";

pub async fn reconcile(obj: Arc<MultiDeployment>, ctx: Arc<Context>) -> Result<Action, Error> {
    info!("Reconciling MultiDeployment: {}", obj.name_any());
    let _multi_deployments = ctx.multi_deployments.clone();
    let deployments = ctx.deployments.clone();

    // check that at least one child deployment is defined
    if obj.spec.children.len() == 0 {
        return Err(Error::ValidationError(
            "At least one child deployment must be defined".to_string(),
        ));
    }

    // validate that no child deployment has negative values
    if obj
        .spec
        .children
        .values()
        .map(|c| c.weight.unwrap_or(0))
        .any(|w| w < 0)
    {
        return Err(Error::ValidationError(
            "Child deployment weights cannot be negative".to_string(),
        ));
    }
    if obj
        .spec
        .children
        .values()
        .map(|c| c.min_replicas.unwrap_or(0))
        .any(|r| r < 0)
    {
        return Err(Error::ValidationError(
            "Child deployment min_replicas cannot be negative".to_string(),
        ));
    }

    let total_replicas = obj.spec.replicas.unwrap_or(0);
    if total_replicas < 0 {
        return Err(Error::ValidationError(
            "Total replicas cannot be negative".to_string(),
        ));
    }

    // calculate replicas
    let total_min_replicas: i32 = obj
        .spec
        .children
        .values()
        .map(|child| child.min_replicas.unwrap_or(0))
        .sum();

    // validate that total min_replicas does not exceed total replicas
    // total_replicas == 0 is exception, meaning "(temporarily) disabled"
    if total_min_replicas > total_replicas && total_replicas != 0 {
        return Err(Error::ValidationError(
            "Sum of min_replicas of child deployments exceeds total replicas".to_string(),
        ));
    }

    let total_weight: i32 = obj
        .spec
        .children
        .values()
        .map(|child| child.weight.unwrap_or(0))
        .sum();

    if total_weight == 0 && total_replicas != 0 {
        // total_replicas is non-zero, but total_weight is zero
        // this can be regarded as even distribution, but to avoid confusion, we raise an error
        return Err(Error::ValidationError(
            "Total weight of child deployments cannot be zero when total replicas is non-zero"
                .to_string(),
        ));
    }

    // do allocation
    let minimums: Vec<i64> = obj
        .spec
        .children
        .values()
        .map(|c| c.min_replicas.unwrap_or(0).into())
        .collect();
    let weights: Vec<f64> = obj
        .spec
        .children
        .values()
        .map(|c| c.weight.unwrap_or(0).into())
        .collect();
    let calculated_replicas =
        utils::allocate_weighted_with_minima(total_replicas.into(), &minimums, &weights)?;

    for (i, child_name) in obj.spec.children.keys().enumerate() {
        let replicas = Some(calculated_replicas[i] as i32);
        let deployment_data = create_owned_deployment(&obj, child_name.clone(), replicas)?;
        let server_side = PatchParams::apply(CONTROLLER_NAME);

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
    Action::requeue(Duration::from_secs(5 * 60))
}

fn create_owned_deployment(
    source: &MultiDeployment,
    child_name: String,
    replicas: Option<i32>,
) -> Result<Deployment, Error> {
    let oref = source.controller_owner_ref(&()).unwrap();
    let source_name = source.name_any();
    let child_deployment = source.spec.children.get(&child_name).unwrap();

    // create unique selector based on source and child names
    let mut new_selector = source.spec.root_template.selector.clone();
    new_selector
        .match_labels
        .get_or_insert_with(|| BTreeMap::new())
        .insert(
            LABEL_SELECTOR_KEY.to_string(),
            format!("{}-{}", source_name, child_name),
        );

    // create new labels based on root template labels
    let mut new_labels = source
        .spec
        .root_template
        .template
        .metadata
        .as_ref()
        .and_then(|m| m.labels.clone())
        .unwrap_or_default();
    new_labels.insert(
        LABEL_SELECTOR_KEY.to_string(),
        format!("{}-{}", source_name, child_name),
    );

    // build child deployment spec
    let child_deployment_data = DeploymentSpec {
        selector: new_selector,
        template: PodTemplateSpec {
            metadata: Some(ObjectMeta {
                labels: Some(new_labels),
                ..Default::default()
            }),
            spec: Some(child_deployment.pod_spec.clone()),
        },
        replicas,
        ..source.spec.root_template.clone()
    };

    // merge two deployment specs
    let mut root_spec = serde_json::to_value(&source.spec.root_template)?;
    let child_spec = serde_json::to_value(&child_deployment_data)?;

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
