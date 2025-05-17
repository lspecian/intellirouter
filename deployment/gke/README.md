# GKE Deployment

This directory contains configuration for deploying IntelliRouter on Google Kubernetes Engine (GKE).

## Requirements

- Google Cloud SDK
- kubectl
- Helm

## Setup

1. Create a GKE cluster (if not already created):

```bash
# Create a cluster with a dedicated node pool
gcloud container clusters create intellirouter \
  --zone us-central1-a \
  --num-nodes 3 \
  --machine-type e2-standard-2

# Create a dedicated node pool for IntelliRouter
gcloud container node-pools create intellirouter-pool \
  --cluster intellirouter \
  --zone us-central1-a \
  --num-nodes 3 \
  --machine-type e2-standard-4
```

2. Configure kubectl to use the GKE cluster:

```bash
gcloud container clusters get-credentials intellirouter --zone us-central1-a
```

3. Reserve a static IP address for the ingress:

```bash
gcloud compute addresses create intellirouter-ip --global
```

4. Create a managed certificate:

```bash
cat <<EOF | kubectl apply -f -
apiVersion: networking.gke.io/v1
kind: ManagedCertificate
metadata:
  name: intellirouter-cert
spec:
  domains:
    - intellirouter.example.com
EOF
```

5. Install the Helm chart:

```bash
helm install intellirouter ../../helm/intellirouter -f values.yaml
```

6. Upgrade the deployment:

```bash
helm upgrade intellirouter ../../helm/intellirouter -f values.yaml
```

7. Uninstall the deployment:

```bash
helm uninstall intellirouter
```

## Configuration

The GKE deployment is configured for production use with:

- Google Cloud Load Balancer for ingress
- Google-managed TLS certificates
- Autoscaling based on CPU and memory usage
- Persistent disks for storage
- Multiple replicas for each component
- Specific node pool selection

## DNS Configuration

After deployment, you'll need to:

1. Get the global IP address:
```bash
gcloud compute addresses describe intellirouter-ip --global --format='value(address)'
```

2. Create a DNS record for your domain (intellirouter.example.com) pointing to this IP address

## Monitoring and Logging

For production monitoring:

1. Enable Cloud Monitoring and Logging:
```bash
gcloud container clusters update intellirouter \
  --zone us-central1-a \
  --enable-stackdriver-kubernetes
```

2. Set up Cloud Monitoring dashboards:
```bash
gcloud monitoring dashboards create --config-from-file=monitoring-dashboard.json
```

## Security Considerations

For production deployments:

1. Use Google Secret Manager for sensitive configuration
2. Enable network policies for pod-to-pod communication
3. Configure Workload Identity for GCP service access
4. Set up Binary Authorization for container image verification
5. Enable VPC Service Controls for additional network security