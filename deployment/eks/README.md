# EKS Deployment

This directory contains configuration for deploying IntelliRouter on Amazon EKS.

## Requirements

- AWS CLI
- eksctl
- kubectl
- Helm

## Setup

1. Create an EKS cluster (if not already created):

```bash
eksctl create cluster \
  --name intellirouter \
  --region us-west-2 \
  --nodegroup-name intellirouter \
  --node-type t3.large \
  --nodes 3 \
  --nodes-min 2 \
  --nodes-max 5 \
  --managed
```

2. Install the AWS Load Balancer Controller:

```bash
# Add the EKS chart repo
helm repo add eks https://aws.github.io/eks-charts
helm repo update

# Create IAM service account
eksctl create iamserviceaccount \
  --cluster=intellirouter \
  --namespace=kube-system \
  --name=aws-load-balancer-controller \
  --attach-policy-arn=arn:aws:iam::$(aws sts get-caller-identity --query Account --output text):policy/AWSLoadBalancerControllerIAMPolicy \
  --approve

# Install the AWS Load Balancer Controller
helm install aws-load-balancer-controller eks/aws-load-balancer-controller \
  -n kube-system \
  --set clusterName=intellirouter \
  --set serviceAccount.create=false \
  --set serviceAccount.name=aws-load-balancer-controller
```

3. Install the Helm chart:

```bash
helm install intellirouter ../../helm/intellirouter -f values.yaml
```

4. Upgrade the deployment:

```bash
helm upgrade intellirouter ../../helm/intellirouter -f values.yaml
```

5. Uninstall the deployment:

```bash
helm uninstall intellirouter
```

## Configuration

The EKS deployment is configured for production use with:

- AWS Application Load Balancer (ALB) for ingress
- TLS termination
- Autoscaling based on CPU and memory usage
- EBS volumes for persistent storage
- Multiple replicas for each component
- Specific node group selection

## DNS Configuration

After deployment, you'll need to:

1. Get the ALB address:
```bash
kubectl get ingress
```

2. Create a DNS record for your domain (intellirouter.example.com) pointing to the ALB address

## Monitoring and Logging

For production monitoring:

1. Enable CloudWatch Container Insights:
```bash
eksctl utils enable-container-insights --cluster=intellirouter --region=us-west-2
```

2. Set up CloudWatch Logs for container logs:
```bash
kubectl apply -f https://raw.githubusercontent.com/aws-samples/amazon-cloudwatch-container-insights/latest/k8s-deployment-manifest-templates/deployment-mode/daemonset/container-insights-monitoring/fluentd/fluentd.yaml
```

## Security Considerations

For production deployments:

1. Use AWS Secrets Manager or Parameter Store for sensitive configuration
2. Enable network policies for pod-to-pod communication
3. Configure IAM roles for service accounts (IRSA) for AWS service access
4. Implement AWS WAF with the ALB for additional protection