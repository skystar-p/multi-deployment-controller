use kube::CustomResourceExt;

use multi_deployment_controller::crd::MultiDeployment;

fn main() {
    print!(
        "{}",
        serde_yaml::to_string(&MultiDeployment::crd()).unwrap()
    )
}
