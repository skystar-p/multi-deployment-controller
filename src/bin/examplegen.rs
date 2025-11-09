use std::collections::BTreeMap;

use k8s_openapi::{
    api::{
        apps::v1::DeploymentSpec,
        core::v1::{Container, PodSpec, PodTemplateSpec},
    },
    apimachinery::pkg::apis::meta::v1::LabelSelector,
};
use kube::api::ObjectMeta;
use multi_deployment::crd::{ChildDeployment, MultiDeployment, MultiDeploymentSpec};

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
                        containers: vec![Container {
                            image: Some("nginx:latest".to_string()),
                            name: "nginx".to_string(),
                            ..Default::default()
                        }],
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
                                name: "alpine".to_string(),
                                image: Some("alpine:latest".to_string()),
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
