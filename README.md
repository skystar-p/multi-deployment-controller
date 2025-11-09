# multi-deployment Controller

This controller lets you make multiple deployments based on root template with some derivations, with weighted replica count assignment and minimum replica.

## Use cases
* Deploy some pods on spot node, but with minimum guaranteed pod count of on-demand pods.
* Deploy canary with different image tag, and environment variables.

## Build

```bash
cargo build --release

# or, use Nix
nix build
```

## Usage

1. Apply CRD.

```bash
cargo run --bin crdgen | kubectl apply -f - --server-side
```

1. Apply MultiDeployment custom resource.

```yaml
apiVersion: skystar.dev/v1
kind: MultiDeployment
metadata:
  name: example
spec:
  name: example
  replicas: 10
  rootTemplate:
    selector:
      matchLabels:
        app: root-app
    template:
      metadata:
        labels:
          app: root-app
  children:
    debian:
      weight: 70
      minReplicas: 1
      podSpec:
        containers:
        - name: debian
          image: debian:latest
          args:
          - sleep
          - "3600"
          resources:
            requests:
              cpu: 100m
    ubuntu:
      weight: 30
      minReplicas: 1
      podSpec:
        containers:
        - name: ubuntu
          image: ubuntu:latest
          args:
          - sleep
          - "3600"
          resources:
            requests:
              cpu: 100m
```

```bash
kubectl apply -f example.yaml
```

1. Run controller.

```bash
cargo run --bin multi-deployment
# if you prefer Nix, then use `nix run`
```

1. (Optional) Add HPA.

```bash
kubectl autoscale multideployments example --min=2 --max=20 --cpu=50%
```
