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
#[kube(scale(
    spec_replicas_path = ".spec.replicas",
    status_replicas_path = ".status.replicas",
    label_selector_path = ".status.selector"
))]
pub struct MultiDeploymentSpec {
    pub name: String,
    pub replicas: Option<i32>,

    #[serde(rename = "rootTemplate")]
    pub root_template: DeploymentSpec,
    pub children: BTreeMap<String, ChildDeployment>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct MultiDeploymentStatus {
    replicas: Option<i32>,
    selector: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct ChildDeployment {
    pub weight: Option<i32>,
    #[serde(rename = "minReplicas")]
    pub min_replicas: Option<i32>,

    #[serde(rename = "podSpec")]
    pub pod_spec: PodSpec,
}
