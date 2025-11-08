use thiserror::Error;

pub struct Context {}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Kubernetes error: {0}")]
    KubeError(#[from] kube::Error),
}
