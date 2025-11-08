use kube::CustomResourceExt;

use multi_deployment::crd::MultiDeployment;

fn main() {
    print!(
        "{}",
        serde_yaml::to_string(&MultiDeployment::crd()).unwrap()
    )
}
