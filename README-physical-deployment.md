# IntelliRouter Physical Deployment Guide

This guide provides detailed instructions for deploying IntelliRouter components across physical nodes in a production environment. It covers network setup, hardware recommendations, service management, security best practices, and backup/recovery procedures.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Components](#components)
3. [Hardware Requirements](#hardware-requirements)
4. [Network Configuration](#network-configuration)
5. [Installation](#installation)
6. [Service Configuration](#service-configuration)
7. [Service Management](#service-management)
8. [Security](#security)
9. [Backup and Recovery](#backup-and-recovery)
10. [Monitoring and Logging](#monitoring-and-logging)
11. [Troubleshooting](#troubleshooting)
12. [Scaling Guidelines](#scaling-guidelines)

## Architecture Overview

IntelliRouter is designed as a distributed system with multiple specialized roles that can be deployed across different physical nodes. This architecture provides flexibility, scalability, and fault tolerance.

### Deployment Topology

A typical production deployment consists of:

- **Frontend Node(s)**: Hosts the Router service, which acts as the entry point for client requests
- **Orchestration Node(s)**: Hosts the Chain Engine service for workflow orchestration
- **RAG Node(s)**: Hosts the RAG Manager service and vector database for retrieval-augmented generation
- **Persona Node(s)**: Hosts the Persona Layer service for managing system prompts and personas
- **Database Node(s)**: Hosts Redis for inter-service communication and state management
- **LLM Node(s)** (optional): Hosts Ollama for local model inference

![Network Topology](./deployment/physical/diagrams/network-topology.png)

## Components

IntelliRouter consists of the following core components:

1. **Router Service**: Routes requests to appropriate LLM backends
   - Primary entry point for API requests
   - Implements routing strategies (round-robin, cost-optimized, etc.)
   - Manages model registry and connection pooling

2. **Orchestrator Service (Chain Engine)**: Manages the execution of chains and workflows
   - Executes multi-step LLM workflows
   - Supports conditional branching, parallel execution, and loops
   - Handles error recovery and retries

3. **RAG Injector Service (RAG Manager)**: Manages retrieval-augmented generation
   - Integrates with vector databases (ChromaDB)
   - Handles document ingestion and embedding
   - Performs context retrieval and injection

4. **Summarizer Service (Persona Layer)**: Manages system prompts and personas
   - Stores and manages persona definitions
   - Handles prompt templating and customization
   - Provides consistent personality across interactions

5. **Redis**: Used for memory storage and inter-service communication
   - Provides pub/sub messaging between services
   - Stores session state and caching
   - Enables distributed locking and coordination

6. **ChromaDB**: Vector database for RAG functionality
   - Stores document embeddings
   - Performs vector similarity searches
   - Manages document collections and metadata

7. **Ollama** (optional): Local model hosting
   - Runs LLMs locally on GPU hardware
   - Provides API-compatible interface to hosted models
   - Supports custom model loading and quantization

## Hardware Requirements

### Minimum Requirements (Small Deployment)

| Component | CPU | Memory | Storage | Network | GPU (if using Ollama) |
|-----------|-----|--------|---------|---------|------------------------|
| Router | 4 cores | 8 GB | 20 GB SSD | 1 Gbps | N/A |
| Orchestrator | 4 cores | 8 GB | 20 GB SSD | 1 Gbps | N/A |
| RAG Manager | 4 cores | 16 GB | 100 GB SSD | 1 Gbps | N/A |
| Persona Layer | 2 cores | 4 GB | 20 GB SSD | 1 Gbps | N/A |
| Redis | 2 cores | 8 GB | 50 GB SSD | 1 Gbps | N/A |
| ChromaDB | 4 cores | 16 GB | 100 GB SSD | 1 Gbps | N/A |
| Ollama (optional) | 8 cores | 32 GB | 100 GB SSD | 1 Gbps | NVIDIA RTX 3090 or better |

### Recommended Requirements (Medium Deployment)

| Component | CPU | Memory | Storage | Network | GPU (if using Ollama) |
|-----------|-----|--------|---------|---------|------------------------|
| Router | 8 cores | 16 GB | 50 GB SSD | 10 Gbps | N/A |
| Orchestrator | 8 cores | 16 GB | 50 GB SSD | 10 Gbps | N/A |
| RAG Manager | 8 cores | 32 GB | 500 GB SSD | 10 Gbps | N/A |
| Persona Layer | 4 cores | 8 GB | 50 GB SSD | 10 Gbps | N/A |
| Redis | 4 cores | 16 GB | 100 GB SSD | 10 Gbps | N/A |
| ChromaDB | 8 cores | 32 GB | 500 GB SSD | 10 Gbps | N/A |
| Ollama (optional) | 16 cores | 64 GB | 500 GB SSD | 10 Gbps | NVIDIA RTX 4090 or A100 |

### Large-Scale Deployment

For large-scale deployments, consider:
- Multiple router instances behind a load balancer
- Distributed Redis cluster with Redis Sentinel for high availability
- Sharded ChromaDB deployment
- Multiple Ollama instances with specialized models
- Dedicated monitoring and logging infrastructure

## Network Configuration

### Network Topology

IntelliRouter services communicate over HTTP/HTTPS and gRPC. The recommended network topology includes:

- **Public-facing DMZ**: Contains load balancers and API gateways
- **Application Tier**: Contains Router and Orchestrator services
- **Data Tier**: Contains RAG Manager, Persona Layer, Redis, and ChromaDB
- **Inference Tier** (optional): Contains Ollama instances

### Network Requirements

| Connection | Protocol | Port | Description |
|------------|----------|------|-------------|
| Client → Router | HTTP/HTTPS | 8080/443 | Client API requests |
| Router → Orchestrator | HTTP/gRPC | 8081 | Chain execution requests |
| Router → RAG Manager | HTTP/gRPC | 8082 | RAG requests |
| Router → Persona Layer | HTTP/gRPC | 8083 | Persona requests |
| All Services → Redis | TCP | 6379 | Pub/sub and state management |
| RAG Manager → ChromaDB | HTTP | 8000 | Vector database access |
| All Services → Ollama (optional) | HTTP | 11434 | Local model inference |

### Firewall Configuration

Configure firewalls to allow only necessary traffic between services:

```
# Example iptables rules for Router node
iptables -A INPUT -p tcp --dport 8080 -j ACCEPT  # Allow incoming API requests
iptables -A OUTPUT -p tcp --dport 8081 -j ACCEPT  # Allow outgoing requests to Orchestrator
iptables -A OUTPUT -p tcp --dport 8082 -j ACCEPT  # Allow outgoing requests to RAG Manager
iptables -A OUTPUT -p tcp --dport 8083 -j ACCEPT  # Allow outgoing requests to Persona Layer
iptables -A OUTPUT -p tcp --dport 6379 -j ACCEPT  # Allow outgoing requests to Redis
```

### Load Balancing

For high-availability deployments, use a load balancer (such as HAProxy, NGINX, or cloud provider load balancers) in front of multiple Router instances:

```
# Example HAProxy configuration snippet
frontend intellirouter_frontend
    bind *:443 ssl crt /etc/ssl/intellirouter.pem
    default_backend intellirouter_routers

backend intellirouter_routers
    balance roundrobin
    option httpchk GET /health
    server router1 router1.example.com:8080 check
    server router2 router2.example.com:8080 check
```

### Service Discovery

IntelliRouter services can discover each other using:

1. **Static configuration**: Hardcoded service endpoints in configuration files
2. **DNS-based discovery**: Using DNS SRV records
3. **Environment variables**: Service endpoints passed as environment variables

For physical deployments, static configuration or DNS-based discovery is recommended.

## Installation

### Prerequisites

1. Operating System: Ubuntu 22.04 LTS or RHEL/CentOS 8+
2. Rust toolchain (for building from source)
3. Docker (optional, for running ChromaDB and Redis)
4. NVIDIA drivers and CUDA (if using Ollama with GPU)

### Installation Steps

1. **Prepare the system**:

```bash
# Update system packages
sudo apt update && sudo apt upgrade -y

# Install dependencies
sudo apt install -y build-essential pkg-config libssl-dev curl git

# Install Rust (if building from source)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

2. **Create directory structure**:

```bash
# Create application directories
sudo mkdir -p /opt/intellirouter/{bin,config,data,logs}
sudo mkdir -p /opt/intellirouter/data/{documents,personas}
sudo mkdir -p /opt/intellirouter/logs/{router,orchestrator,rag-manager,persona-layer}

# Set permissions
sudo chown -R $USER:$USER /opt/intellirouter
```

3. **Install IntelliRouter**:

```bash
# Clone the repository
git clone https://github.com/intellirouter/intellirouter.git
cd intellirouter

# Build the application
cargo build --release

# Copy the binary
cp target/release/intellirouter /opt/intellirouter/bin/

# Copy configuration files
cp -r config/* /opt/intellirouter/config/
```

4. **Install Redis** (on the Redis node):

```bash
# Install Redis
sudo apt install -y redis-server

# Configure Redis
sudo cp deployment/physical/redis/redis.conf /etc/redis/redis.conf
sudo systemctl restart redis-server
```

5. **Install ChromaDB** (on the RAG node):

```bash
# Install Docker
sudo apt install -y docker.io
sudo systemctl enable --now docker

# Run ChromaDB
sudo docker run -d \
  --name chromadb \
  -p 8000:8000 \
  -v /opt/intellirouter/data/chromadb:/chroma/chroma \
  ghcr.io/chroma-core/chroma:latest
```

6. **Install Ollama** (optional, on the LLM node):

```bash
# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh

# Configure Ollama
mkdir -p ~/.ollama
cp deployment/physical/ollama/config.yaml ~/.ollama/

# Pull models
ollama pull llama3
```

## Service Configuration

### Configuration Files

IntelliRouter uses TOML configuration files located in `/opt/intellirouter/config/`. Each role requires a specific configuration:

1. **Router Configuration** (`router.toml`):

```toml
# Environment
environment = "production"

# Server configuration
[server]
host = "0.0.0.0"
port = 8080

# Telemetry configuration
[telemetry]
log_level = "info"
metrics_enabled = true
tracing_enabled = true

# Memory configuration
[memory]
backend_type = "redis"
redis_url = "redis://redis.example.com:6379"

# IPC security configuration
[ipc.security]
enabled = true
token = "your_secure_token_here"

# Model registry configuration
[model_registry]
refresh_interval_seconds = 300
```

2. **Orchestrator Configuration** (`orchestrator.toml`):

```toml
# Environment
environment = "production"

# Server configuration
[server]
host = "0.0.0.0"
port = 8080

# Telemetry configuration
[telemetry]
log_level = "info"
metrics_enabled = true
tracing_enabled = true

# Memory configuration
[memory]
backend_type = "redis"
redis_url = "redis://redis.example.com:6379"

# IPC security configuration
[ipc.security]
enabled = true
token = "your_secure_token_here"

# Chain engine configuration
[chain_engine]
enable_caching = true
max_parallel_steps = 10
default_timeout_seconds = 30

# Service discovery
[ipc]
router_endpoint = "http://router.example.com:8080"
```

3. **RAG Manager Configuration** (`rag-manager.toml`):

```toml
# Environment
environment = "production"

# Server configuration
[server]
host = "0.0.0.0"
port = 8080

# Telemetry configuration
[telemetry]
log_level = "info"
metrics_enabled = true
tracing_enabled = true

# Memory configuration
[memory]
backend_type = "redis"
redis_url = "redis://redis.example.com:6379"

# IPC security configuration
[ipc.security]
enabled = true
token = "your_secure_token_here"

# RAG configuration
[rag]
enabled = true
vector_db_url = "http://chromadb.example.com:8000"
embedding_model = "all-MiniLM-L6-v2"
chunk_size = 1000
chunk_overlap = 200

# Service discovery
[ipc]
router_endpoint = "http://router.example.com:8080"
```

4. **Persona Layer Configuration** (`persona-layer.toml`):

```toml
# Environment
environment = "production"

# Server configuration
[server]
host = "0.0.0.0"
port = 8080

# Telemetry configuration
[telemetry]
log_level = "info"
metrics_enabled = true
tracing_enabled = true

# Memory configuration
[memory]
backend_type = "redis"
redis_url = "redis://redis.example.com:6379"

# IPC security configuration
[ipc.security]
enabled = true
token = "your_secure_token_here"

# Persona layer configuration
[persona_layer]
enabled = true
personas_path = "/opt/intellirouter/data/personas"

# Service discovery
[ipc]
router_endpoint = "http://router.example.com:8080"
```

### Environment Variables

Environment variables can override configuration file settings. Create environment variable files for each service:

1. **Router Environment** (`/opt/intellirouter/env/router.env`):

```bash
INTELLIROUTER_ENVIRONMENT=production
INTELLIROUTER__SERVER__HOST=0.0.0.0
INTELLIROUTER__SERVER__PORT=8080
INTELLIROUTER__TELEMETRY__LOG_LEVEL=info
INTELLIROUTER__MEMORY__BACKEND_TYPE=redis
INTELLIROUTER__MEMORY__REDIS_URL=redis://redis.example.com:6379
INTELLIROUTER__IPC__SECURITY__ENABLED=true
INTELLIROUTER__IPC__SECURITY__TOKEN=your_secure_token_here
# API Keys (set as needed)
ANTHROPIC_API_KEY=your_anthropic_key
OPENAI_API_KEY=your_openai_key
GOOGLE_API_KEY=your_google_key
MISTRAL_API_KEY=your_mistral_key
```

Similar environment files should be created for other services.

## Service Management

### Systemd Service Files

Create systemd service files for each IntelliRouter component:

1. **Router Service** (`/etc/systemd/system/intellirouter-router.service`):

```ini
[Unit]
Description=IntelliRouter Router Service
After=network.target
Wants=redis-server.service

[Service]
Type=simple
User=intellirouter
Group=intellirouter
WorkingDirectory=/opt/intellirouter
EnvironmentFile=/opt/intellirouter/env/router.env
ExecStart=/opt/intellirouter/bin/intellirouter run --role Router --config /opt/intellirouter/config/router.toml
Restart=on-failure
RestartSec=5
LimitNOFILE=65536
StandardOutput=append:/opt/intellirouter/logs/router/stdout.log
StandardError=append:/opt/intellirouter/logs/router/stderr.log

[Install]
WantedBy=multi-user.target
```

2. **Orchestrator Service** (`/etc/systemd/system/intellirouter-orchestrator.service`):

```ini
[Unit]
Description=IntelliRouter Orchestrator Service
After=network.target intellirouter-router.service
Wants=redis-server.service

[Service]
Type=simple
User=intellirouter
Group=intellirouter
WorkingDirectory=/opt/intellirouter
EnvironmentFile=/opt/intellirouter/env/orchestrator.env
ExecStart=/opt/intellirouter/bin/intellirouter run --role ChainEngine --config /opt/intellirouter/config/orchestrator.toml
Restart=on-failure
RestartSec=5
LimitNOFILE=65536
StandardOutput=append:/opt/intellirouter/logs/orchestrator/stdout.log
StandardError=append:/opt/intellirouter/logs/orchestrator/stderr.log

[Install]
WantedBy=multi-user.target
```

3. **RAG Manager Service** (`/etc/systemd/system/intellirouter-rag-manager.service`):

```ini
[Unit]
Description=IntelliRouter RAG Manager Service
After=network.target intellirouter-router.service
Wants=redis-server.service

[Service]
Type=simple
User=intellirouter
Group=intellirouter
WorkingDirectory=/opt/intellirouter
EnvironmentFile=/opt/intellirouter/env/rag-manager.env
ExecStart=/opt/intellirouter/bin/intellirouter run --role RagManager --config /opt/intellirouter/config/rag-manager.toml
Restart=on-failure
RestartSec=5
LimitNOFILE=65536
StandardOutput=append:/opt/intellirouter/logs/rag-manager/stdout.log
StandardError=append:/opt/intellirouter/logs/rag-manager/stderr.log

[Install]
WantedBy=multi-user.target
```

4. **Persona Layer Service** (`/etc/systemd/system/intellirouter-persona-layer.service`):

```ini
[Unit]
Description=IntelliRouter Persona Layer Service
After=network.target intellirouter-router.service
Wants=redis-server.service

[Service]
Type=simple
User=intellirouter
Group=intellirouter
WorkingDirectory=/opt/intellirouter
EnvironmentFile=/opt/intellirouter/env/persona-layer.env
ExecStart=/opt/intellirouter/bin/intellirouter run --role PersonaLayer --config /opt/intellirouter/config/persona-layer.toml
Restart=on-failure
RestartSec=5
LimitNOFILE=65536
StandardOutput=append:/opt/intellirouter/logs/persona-layer/stdout.log
StandardError=append:/opt/intellirouter/logs/persona-layer/stderr.log

[Install]
WantedBy=multi-user.target
```

### Service Management Commands

```bash
# Enable and start services
sudo systemctl enable intellirouter-router
sudo systemctl start intellirouter-router

sudo systemctl enable intellirouter-orchestrator
sudo systemctl start intellirouter-orchestrator

sudo systemctl enable intellirouter-rag-manager
sudo systemctl start intellirouter-rag-manager

sudo systemctl enable intellirouter-persona-layer
sudo systemctl start intellirouter-persona-layer

# Check service status
sudo systemctl status intellirouter-router
sudo systemctl status intellirouter-orchestrator
sudo systemctl status intellirouter-rag-manager
sudo systemctl status intellirouter-persona-layer

# View logs
sudo journalctl -u intellirouter-router -f
sudo journalctl -u intellirouter-orchestrator -f
sudo journalctl -u intellirouter-rag-manager -f
sudo journalctl -u intellirouter-persona-layer -f
```

### Log Rotation

Configure log rotation to manage log files:

```
# /etc/logrotate.d/intellirouter
/opt/intellirouter/logs/*/*.log {
    daily
    missingok
    rotate 14
    compress
    delaycompress
    notifempty
    create 0640 intellirouter intellirouter
    sharedscripts
    postrotate
        systemctl reload intellirouter-router intellirouter-orchestrator intellirouter-rag-manager intellirouter-persona-layer >/dev/null 2>&1 || true
    endscript
}
```

### Health Checks

Implement health checks to monitor service status:

```bash
# Check Router health
curl -f http://router.example.com:8080/health

# Check Orchestrator health
curl -f http://orchestrator.example.com:8080/health

# Check RAG Manager health
curl -f http://rag-manager.example.com:8080/health

# Check Persona Layer health
curl -f http://persona-layer.example.com:8080/health

# Check Redis health
redis-cli -h redis.example.com ping

# Check ChromaDB health
curl -f http://chromadb.example.com:8000/api/v1/heartbeat
```

## Security

### Authentication and Authorization

1. **API Authentication**:
   - Use API keys for client authentication
   - Implement JWT-based authentication for service-to-service communication
   - Consider OAuth2 for user authentication

2. **Inter-Service Security**:
   - Enable IPC security with secure tokens
   - Use mutual TLS (mTLS) for service-to-service communication
   - Implement role-based access control (RBAC) for service permissions

### Network Security

1. **Firewall Configuration**:
   - Restrict access to service ports
   - Implement network segmentation
   - Use security groups or network ACLs

2. **TLS Configuration**:
   - Use TLS 1.3 for all external communications
   - Configure secure cipher suites
   - Implement certificate rotation

3. **DDoS Protection**:
   - Implement rate limiting
   - Use a CDN or DDoS protection service
   - Configure connection timeouts

### Data Security

1. **Encryption**:
   - Encrypt sensitive data at rest
   - Use TLS for data in transit
   - Implement key rotation policies

2. **API Key Management**:
   - Store API keys securely
   - Rotate keys regularly
   - Use environment variables or secure vaults

### Secure Configuration

1. **Principle of Least Privilege**:
   - Run services as non-root users
   - Limit file permissions
   - Use capability-based security

2. **Secrets Management**:
   - Use environment variables for secrets
   - Consider HashiCorp Vault or AWS Secrets Manager
   - Avoid hardcoding secrets in configuration files

## Backup and Recovery

### Backup Strategy

1. **Configuration Backup**:
   - Back up all configuration files
   - Store backups securely off-site
   - Version control configuration changes

2. **Data Backup**:
   - Back up Redis data using RDB or AOF persistence
   - Back up ChromaDB data
   - Back up persona definitions and documents

3. **Backup Schedule**:
   - Daily incremental backups
   - Weekly full backups
   - Monthly off-site backups

### Backup Script

```bash
#!/bin/bash
# intellirouter-backup.sh

BACKUP_DIR="/opt/intellirouter/backups/$(date +%Y-%m-%d)"
mkdir -p $BACKUP_DIR

# Backup configuration
cp -r /opt/intellirouter/config $BACKUP_DIR/config

# Backup data
cp -r /opt/intellirouter/data $BACKUP_DIR/data

# Backup Redis (if on same machine)
redis-cli save
cp /var/lib/redis/dump.rdb $BACKUP_DIR/redis-dump.rdb

# Compress backup
tar -czf $BACKUP_DIR.tar.gz $BACKUP_DIR
rm -rf $BACKUP_DIR

# Rotate backups (keep last 30 days)
find /opt/intellirouter/backups -name "*.tar.gz" -type f -mtime +30 -delete
```

### Recovery Procedures

1. **Configuration Recovery**:
   - Restore configuration files from backup
   - Verify configuration integrity
   - Restart services

2. **Data Recovery**:
   - Restore Redis data
   - Restore ChromaDB data
   - Restore persona definitions and documents

3. **Disaster Recovery**:
   - Document step-by-step recovery procedures
   - Test recovery procedures regularly
   - Maintain up-to-date system documentation

### Recovery Script

```bash
#!/bin/bash
# intellirouter-restore.sh

if [ $# -ne 1 ]; then
    echo "Usage: $0 <backup_file.tar.gz>"
    exit 1
fi

BACKUP_FILE=$1
TEMP_DIR="/tmp/intellirouter-restore"

# Extract backup
mkdir -p $TEMP_DIR
tar -xzf $BACKUP_FILE -C $TEMP_DIR

# Stop services
systemctl stop intellirouter-router intellirouter-orchestrator intellirouter-rag-manager intellirouter-persona-layer

# Restore configuration
cp -r $TEMP_DIR/*/config/* /opt/intellirouter/config/

# Restore data
cp -r $TEMP_DIR/*/data/* /opt/intellirouter/data/

# Restore Redis (if on same machine)
systemctl stop redis-server
cp $TEMP_DIR/*/redis-dump.rdb /var/lib/redis/dump.rdb
chown redis:redis /var/lib/redis/dump.rdb
systemctl start redis-server

# Start services
systemctl start intellirouter-router intellirouter-orchestrator intellirouter-rag-manager intellirouter-persona-layer

# Clean up
rm -rf $TEMP_DIR
```

## Monitoring and Logging

### Monitoring

1. **System Monitoring**:
   - CPU, memory, disk, and network usage
   - Service availability and response times
   - Error rates and latency

2. **Application Monitoring**:
   - Request rates and response times
   - Error rates and types
   - Queue lengths and processing times

3. **Alerting**:
   - Set up alerts for critical errors
   - Configure notification channels (email, SMS, Slack)
   - Implement escalation policies

### Logging

1. **Log Collection**:
   - Centralize logs using ELK stack or similar
   - Implement structured logging
   - Configure log rotation and retention

2. **Log Analysis**:
   - Search and filter logs
   - Create dashboards for key metrics
   - Set up log-based alerts

### Prometheus Configuration

```yaml
# /etc/prometheus/prometheus.yml
scrape_configs:
  - job_name: 'intellirouter'
    scrape_interval: 15s
    static_configs:
      - targets: ['router.example.com:8080', 'orchestrator.example.com:8080', 'rag-manager.example.com:8080', 'persona-layer.example.com:8080']
        labels:
          group: 'intellirouter'
```

## Troubleshooting

### Common Issues

1. **Service Won't Start**:
   - Check logs: `journalctl -u intellirouter-router -n 100`
   - Verify configuration: `intellirouter validate-config --config /opt/intellirouter/config/router.toml`
   - Check permissions: `ls -la /opt/intellirouter/`

2. **Services Can't Communicate**:
   - Check network connectivity: `ping orchestrator.example.com`
   - Verify firewall rules: `sudo iptables -L`
   - Check IPC security token: Ensure all services use the same token

3. **High Latency**:
   - Check system resources: `top`, `iostat`, `netstat`
   - Monitor request queues: Check Redis queue lengths
   - Verify external API connectivity: Test API endpoints

### Diagnostic Commands

```bash
# Check service status
systemctl status intellirouter-router

# View recent logs
tail -n 100 /opt/intellirouter/logs/router/stderr.log

# Check network connectivity
netstat -tuln | grep 8080

# Test API endpoint
curl -v http://router.example.com:8080/health

# Check Redis connectivity
redis-cli -h redis.example.com ping
```

### Troubleshooting Script

```bash
#!/bin/bash
# intellirouter-diagnose.sh

echo "=== System Information ==="
uname -a
free -h
df -h

echo "=== Service Status ==="
systemctl status intellirouter-router
systemctl status intellirouter-orchestrator
systemctl status intellirouter-rag-manager
systemctl status intellirouter-persona-layer
systemctl status redis-server

echo "=== Network Status ==="
netstat -tuln
ping -c 3 router.example.com
ping -c 3 orchestrator.example.com
ping -c 3 rag-manager.example.com
ping -c 3 persona-layer.example.com
ping -c 3 redis.example.com

echo "=== Health Checks ==="
curl -s http://router.example.com:8080/health
curl -s http://orchestrator.example.com:8080/health
curl -s http://rag-manager.example.com:8080/health
curl -s http://persona-layer.example.com:8080/health

echo "=== Recent Logs ==="
tail -n 20 /opt/intellirouter/logs/router/stderr.log
```

## Scaling Guidelines

### Vertical Scaling

1. **CPU and Memory**:
   - Increase CPU cores and memory for higher throughput
   - Monitor CPU and memory usage to identify bottlenecks
   - Consider dedicated hardware for compute-intensive services

2. **Storage**:
   - Use high-performance SSDs for data storage
   - Implement proper RAID configuration for redundancy
   - Monitor disk I/O and expand as needed

### Horizontal Scaling

1. **Router Service**:
   - Deploy multiple instances behind a load balancer
   - Implement sticky sessions if needed
   - Configure consistent hashing for routing

2. **Orchestrator Service**:
   - Deploy multiple instances for parallel chain execution
   - Use Redis for distributed locking and coordination
   - Implement work distribution strategies

3. **RAG Manager**:
   - Shard document collections across multiple instances
   - Distribute embedding computation
   - Implement caching for frequent queries

4. **Redis**:
   - Implement Redis Cluster for horizontal scaling
   - Use Redis Sentinel for high availability
   - Configure proper memory management

### Load Testing

1. **Performance Benchmarks**:
   - Measure requests per second (RPS)
   - Monitor latency under load
   - Identify bottlenecks and resource constraints

2. **Capacity Planning**:
   - Estimate resource requirements based on expected load
   - Plan for peak usage scenarios
   - Implement auto-scaling where possible

3. **Stress Testing**:
   - Test system behavior under extreme load
   - Verify graceful degradation
   - Identify failure points and recovery mechanisms