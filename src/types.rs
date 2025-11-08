use k8s_openapi::api::apps::v1::Deployment;
use kube::Api;
use thiserror::Error;

use crate::crd::MultiDeployment;

pub struct Context {
    pub multi_deployments: Api<MultiDeployment>,
    pub deployments: Api<Deployment>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Kubernetes error: {0}")]
    KubeError(#[from] kube::Error),
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
}
