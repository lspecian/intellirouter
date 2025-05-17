# MicroK8s Deployment

This directory contains configuration for deploying IntelliRouter on MicroK8s.

## Requirements

- MicroK8s
- Helm

## Setup

1. Enable required MicroK8s addons:

```bash
microk8s enable dns ingress storage
```

2. Install the Helm chart:

```bash
helm install intellirouter ../../helm/intellirouter -f values.yaml
```

3. Upgrade the deployment:

```bash
helm upgrade intellirouter ../../helm/intellirouter -f values.yaml
```

4. Uninstall the deployment:

```bash
helm uninstall intellirouter
```

## Configuration

The MicroK8s deployment is configured to:

- Use the NGINX ingress controller
- Set resource limits appropriate for smaller clusters
- Enable local storage for ChromaDB
- Deploy separate pods for router and orchestrator roles

## Accessing the Application

Once deployed, you can access the application at:

- http://intellirouter.local (requires DNS configuration or hosts file entry)

## Troubleshooting

If you encounter issues:

1. Check the pod status:
```bash
microk8s kubectl get pods
```

2. View logs for a specific pod:
```bash
microk8s kubectl logs <pod-name>
```

3. Verify ingress configuration:
```bash
microk8s kubectl get ingress
```

4. Check service endpoints:
```bash
microk8s kubectl get endpoints