# multi-deployment Controller

This controller lets you make multiple deployments based on root template, with some derivations, with weighted replica count assignment and minimum replica feature.

## Usage

1. Apply CRD.

```bash
cargo run --bin crdgen | kubectl apply -f - --server-side
```

1. Apply MultiDeployment custom resource.

```bash
# TODO!
```

1. Run controller.

```bash
cargo run --bin multi-deployment
```

1. (Optional) Add HPA.

```bash
kubectl autoscale multideployments example-multideployment --min=2 --max=20 --cpu=50%
```
