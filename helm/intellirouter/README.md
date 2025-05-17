# IntelliRouter Helm Chart

This Helm chart deploys IntelliRouter, a programmable LLM gateway, on Kubernetes clusters. It supports various deployment environments including MicroK8s, Minikube, and EKS.

## Components

IntelliRouter consists of the following components:

1. **Router Service**: Routes requests to appropriate LLM backends
2. **Orchestrator Service (Chain Engine)**: Manages the execution of chains and workflows
3. **RAG Injector Service (RAG Manager)**: Manages retrieval-augmented generation
4. **Summarizer Service (Persona Layer)**: Manages system prompts and personas
5. **Redis**: Used for memory storage and inter-service communication
6. **ChromaDB**: Vector database for RAG functionality
7. **Ollama** (optional): Local model hosting

## Prerequisites

- Kubernetes 1.19+
- Helm 3.2.0+
- PV provisioner support in the underlying infrastructure (for persistence)
- For EKS: AWS Load Balancer Controller

## Installation

### General Installation

```bash
# Add the Bitnami repository for Redis dependency
helm repo add bitnami https://charts.bitnami.com/bitnami
helm repo update

# Install the chart with the default values
helm install intellirouter ./helm/intellirouter

# Or install with custom values
helm install intellirouter ./helm/intellirouter -f custom-values.yaml
```

### MicroK8s Installation

1. Enable required MicroK8s addons:

```bash
microk8s enable dns ingress storage
```

2. Install the Helm chart:

```bash
microk8s helm3 install intellirouter ./helm/intellirouter -f ./helm/intellirouter/values-microk8s.yaml
```

3. Access the IntelliRouter API:

```bash
curl http://intellirouter.local/health
```

Note: You may need to add an entry to your `/etc/hosts` file to map `intellirouter.local` to the MicroK8s IP address.

### Minikube Installation

1. Start Minikube with sufficient resources:

```bash
minikube start --cpus 4 --memory 8192 --disk-size 20g
```

2. Enable the ingress addon (optional):

```bash
minikube addons enable ingress
```

3. Install the Helm chart:

```bash
helm install intellirouter ./helm/intellirouter -f ./helm/intellirouter/values-minikube.yaml
```

4. Access the IntelliRouter API:

```bash
minikube service intellirouter-router --url
```

### EKS Installation

1. Create an EKS cluster with appropriate node groups:

```bash
eksctl create cluster -f eks-cluster.yaml
```

2. Install the AWS Load Balancer Controller:

```bash
helm install aws-load-balancer-controller eks/aws-load-balancer-controller \
  -n kube-system \
  --set clusterName=<your-cluster-name> \
  --set serviceAccount.create=false \
  --set serviceAccount.name=aws-load-balancer-controller
```

3. Install the Helm chart:

```bash
helm install intellirouter ./helm/intellirouter -f ./helm/intellirouter/values-eks.yaml
```

4. Configure DNS to point to the ALB endpoint.

## Configuration

The following table lists the configurable parameters of the IntelliRouter chart and their default values.

### Global Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `image.repository` | IntelliRouter image repository | `intellirouter` |
| `image.tag` | IntelliRouter image tag | `latest` |
| `image.pullPolicy` | Image pull policy | `IfNotPresent` |
| `imagePullSecrets` | Image pull secrets | `[]` |
| `nameOverride` | Override the name of the chart | `""` |
| `fullnameOverride` | Override the full name of the chart | `""` |
| `serviceAccount.create` | Create a service account | `true` |
| `serviceAccount.annotations` | Service account annotations | `{}` |
| `serviceAccount.name` | Service account name | `""` |
| `podAnnotations` | Pod annotations | `{}` |
| `podSecurityContext` | Pod security context | `{}` |
| `securityContext` | Container security context | `{}` |
| `service.type` | Kubernetes service type | `ClusterIP` |
| `service.port` | Kubernetes service port | `8080` |
| `ingress.enabled` | Enable ingress | `false` |
| `resources` | CPU/Memory resource requests/limits | `{}` |
| `autoscaling.enabled` | Enable autoscaling | `false` |
| `nodeSelector` | Node selector | `{}` |
| `tolerations` | Tolerations | `[]` |
| `affinity` | Affinity | `{}` |

### IntelliRouter Specific Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `config.environment` | Environment (production, development) | `production` |
| `config.logLevel` | Log level | `info` |
| `config.ipcSecurityEnabled` | Enable IPC security | `true` |
| `config.ipcSecurityToken` | IPC security token | `default_token_please_change` |
| `roles` | Role-specific configuration | See `values.yaml` |
| `persistence.logs.enabled` | Enable logs persistence | `true` |
| `persistence.logs.size` | Logs PVC size | `1Gi` |
| `persistence.documents.enabled` | Enable documents persistence | `true` |
| `persistence.documents.size` | Documents PVC size | `1Gi` |
| `persistence.personas.enabled` | Enable personas persistence | `true` |
| `persistence.personas.size` | Personas PVC size | `1Gi` |
| `redis.enabled` | Enable Redis | `true` |
| `chromadb.enabled` | Enable ChromaDB | `true` |
| `ollama.enabled` | Enable Ollama | `false` |
| `networkPolicy.enabled` | Enable network policies | `false` |
| `rbac.create` | Create RBAC resources | `true` |

## Role-Specific Configuration

Each role can be configured independently with the following parameters:

```yaml
roles:
  - name: router
    replicas: 1
    resources:
      limits:
        cpu: 500m
        memory: 512Mi
      requests:
        cpu: 250m
        memory: 256Mi
    config:
      INTELLIROUTER_ROLE: Router
      # Additional environment variables
```

## Persistence

The chart supports persistence for logs, documents, personas, Redis, and ChromaDB. You can configure the storage class and size for each PVC.

## Security

### RBAC

The chart creates a service account, role, and role binding with the necessary permissions. You can customize the RBAC rules in the `values.yaml` file.

### Network Policies

Network policies can be enabled to restrict traffic between pods. By default, network policies are disabled.

### Secrets

Sensitive information such as API keys and tokens are stored in Kubernetes secrets. You can provide your own values or let the chart generate random values.

## Monitoring

The chart includes liveness and readiness probes for all components. You can customize the probe settings in the `values.yaml` file.

## Troubleshooting

### Common Issues

1. **Pods are stuck in Pending state**
   - Check if PVCs are being provisioned
   - Check if there are enough resources in the cluster

2. **Services are not accessible**
   - Check if the ingress controller is properly configured
   - Check if the services are running and have the correct ports

3. **Components cannot communicate with each other**
   - Check if network policies are blocking traffic
   - Check if the IPC security token is correctly configured

### Logs

You can check the logs of each component using:

```bash
kubectl logs -l app.kubernetes.io/component=router
kubectl logs -l app.kubernetes.io/component=orchestrator
kubectl logs -l app.kubernetes.io/component=rag-injector
kubectl logs -l app.kubernetes.io/component=summarizer
```

## Upgrading

To upgrade the chart:

```bash
helm upgrade intellirouter ./helm/intellirouter -f values.yaml
```

## Uninstallation

To uninstall the chart:

```bash
helm uninstall intellirouter
```

Note: This will not delete the PVCs. To delete the PVCs, you need to do it manually:

```bash
kubectl delete pvc -l app.kubernetes.io/name=intellirouter
```

## License

This chart is licensed under the same license as IntelliRouter.