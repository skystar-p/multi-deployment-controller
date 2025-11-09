use std::collections::BTreeMap;

use k8s_openapi::{
    api::{apps::v1::DeploymentSpec, core::v1::PodSpec},
    serde::{Deserialize, Serialize},
};
use kube::CustomResource;
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
    pub name: String,
    pub replicas: Option<i32>,

    pub root_template: DeploymentSpec,
    pub children: BTreeMap<String, ChildDeployment>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct MultiDeploymentStatus {}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct ChildDeployment {
    pub weight: Option<i32>,
    pub min_replicas: Option<i32>,

    pub pod_spec: PodSpec,
}
