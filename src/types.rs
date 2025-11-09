use k8s_openapi::api::apps::v1::Deployment;
use kube::Api;
use thiserror::Error;

use crate::crd::MultiDeployment;
use crate::utils::AllocationError;

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
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Replica calcuataion error: {0}")]
    ReplicaCalculationError(#[from] AllocationError),
}
