use std::collections::HashMap;

use k8s_openapi::{
    api::apps::v1::{Deployment, DeploymentSpec},
    serde::{Deserialize, Serialize},
};
use kube::{CustomResource, Resource, api::ObjectMeta};
use schemars::JsonSchema;

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(
    kind = "MultiDeployment",
    group = "skystar.dev",
    version = "v1",
    namespaced
)]
#[kube(status = "MultiDeploymentStatus")]
pub struct MultiDeploymentSpec {
    name: String,
    root_template: DeploymentSpec,
    children: HashMap<String, PartialDeployment>,

    replicas: Option<i32>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct MultiDeploymentStatus {}

#[derive(Resource, Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[resource(inherit = Deployment)]
pub struct PartialDeployment {
    metadata: ObjectMeta,
    weight: Option<i32>,
}
