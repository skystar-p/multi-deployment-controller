use std::collections::BTreeMap;

use k8s_openapi::{
    api::{
        apps::v1::DeploymentSpec,
        core::v1::{Container, PodSpec, PodTemplateSpec, ResourceRequirements},
    },
    apimachinery::pkg::{api::resource::Quantity, apis::meta::v1::LabelSelector},
};
use kube::api::ObjectMeta;
use multi_deployment_controller::crd::{ChildDeployment, MultiDeployment, MultiDeploymentSpec};

fn main() {
    let md = MultiDeployment {
        metadata: ObjectMeta {
            name: Some("example-multideployment".to_string()),
            ..Default::default()
        },
        spec: MultiDeploymentSpec {
            name: "example-multideployment".to_string(),
            replicas: Some(3),
            root_template: DeploymentSpec {
                selector: LabelSelector {
                    match_labels: Some(BTreeMap::from([(
                        "app".to_string(),
                        "root-app".to_string(),
                    )])),
                    ..Default::default()
                },
                template: PodTemplateSpec {
                    metadata: Some(ObjectMeta {
                        labels: Some(BTreeMap::from([(
                            "app".to_string(),
                            "root-app".to_string(),
                        )])),
                        ..Default::default()
                    }),
                    spec: Some(PodSpec {
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: BTreeMap::from([
                (
                    "child-a".to_string(),
                    ChildDeployment {
                        weight: Some(70),
                        min_replicas: Some(1),
                        pod_spec: PodSpec {
                            containers: vec![Container {
                                name: "debian".to_string(),
                                image: Some("debian:latest".to_string()),
                                args: Some(vec!["sleep".to_string(), "3600".to_string()]),
                                resources: Some(ResourceRequirements {
                                    requests: Some(BTreeMap::from([(
                                        "cpu".to_string(),
                                        Quantity("100m".to_string()),
                                    )])),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            }],
                            ..Default::default()
                        },
                    },
                ),
                (
                    "child-b".to_string(),
                    ChildDeployment {
                        weight: Some(30),
                        min_replicas: Some(1),
                        pod_spec: PodSpec {
                            containers: vec![Container {
                                name: "ubuntu".to_string(),
                                image: Some("ubuntu:latest".to_string()),
                                args: Some(vec!["sleep".to_string(), "3600".to_string()]),
                                resources: Some(ResourceRequirements {
                                    requests: Some(BTreeMap::from([(
                                        "cpu".to_string(),
                                        Quantity("100m".to_string()),
                                    )])),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            }],
                            ..Default::default()
                        },
                    },
                ),
            ]),
        },

        status: None,
    };

    print!("{}", serde_yaml::to_string(&md).unwrap());
}
