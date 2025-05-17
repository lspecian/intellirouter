# IntelliRouter Docker Deployment Guide

This guide provides comprehensive instructions for deploying IntelliRouter using Docker Compose across different environments and platforms.

## Table of Contents

- [Overview](#overview)
- [System Architecture](#system-architecture)
- [Prerequisites](#prerequisites)
- [Deployment Options](#deployment-options)
  - [Production Deployment](#production-deployment)
  - [Development Environment](#development-environment)
  - [Testing Environment](#testing-environment)
- [Platform-Specific Instructions](#platform-specific-instructions)
  - [Linux Setup](#linux-setup)
  - [macOS Setup](#macos-setup)
  - [Windows (WSL2) Setup](#windows-wsl2-setup)
- [Configuration](#configuration)
  - [Environment Variables](#environment-variables)
  - [Volume Mounts](#volume-mounts)
  - [Security Considerations](#security-considerations)
- [Scaling](#scaling)
  - [Horizontal Scaling](#horizontal-scaling)
  - [Resource Considerations](#resource-considerations)
  - [Load Balancing](#load-balancing)
- [Using Ollama for Local Models](#using-ollama-for-local-models)
- [Troubleshooting](#troubleshooting)
- [Performance Tuning](#performance-tuning)

## Overview

IntelliRouter is a modular system for routing and orchestrating LLM requests. The Docker Compose deployment includes:

- **Router Service**: Routes requests to appropriate LLM backends
- **Orchestrator Service (Chain Engine)**: Manages the execution of chains and workflows
- **RAG Injector Service (RAG Manager)**: Manages retrieval-augmented generation
- **Summarizer Service (Persona Layer)**: Manages system prompts and personas
- **Supporting Services**:
  - Redis for IPC and caching
  - ChromaDB for vector storage
  - Optional Ollama for local model hosting

## System Architecture

The system is designed with a microservices architecture where each component has a specific role:

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Client    │────▶│   Router    │────▶│ Orchestrator│
└─────────────┘     └─────────────┘     └─────────────┘
                          │ ▲                  │
                          │ │                  │
                          ▼ │                  ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   ChromaDB  │◀───▶│ RAG Injector│◀───▶│  Summarizer │
└─────────────┘     └─────────────┘     └─────────────┘
                          │                    │
                          │                    │
                          ▼                    ▼
                    ┌─────────────┐     ┌─────────────┐
                    │    Redis    │     │   Ollama    │
                    └─────────────┘     └─────────────┘
```

## Prerequisites

- Docker Engine (20.10.0+)
- Docker Compose (2.0.0+)
- At least 4GB RAM (8GB+ recommended)
- 20GB+ free disk space
- For GPU support: NVIDIA Container Toolkit (for Ollama)

## Deployment Options

### Production Deployment

For production environments, use the main docker-compose.yml file:

```bash
# Create required directories
mkdir -p logs/{router,orchestrator,rag-injector,summarizer} data/{documents,personas}

# Set up environment variables (recommended: create a .env file)
cp .env.example .env
# Edit .env to add your API keys

# Start all services
docker-compose up -d

# Check service status
docker-compose ps
```

### Development Environment

For development, use the docker-compose.dev.yml file which includes hot-reloading and development-specific settings:

```bash
# Start development environment
docker-compose -f docker-compose.dev.yml up -d

# View logs
docker-compose -f docker-compose.dev.yml logs -f

# Rebuild a specific service after code changes
docker-compose -f docker-compose.dev.yml build router
docker-compose -f docker-compose.dev.yml up -d router
```

### Testing Environment

For running tests, use the docker-compose.test.yml file:

```bash
# Run integration tests
docker-compose -f docker-compose.test.yml up test-runner

# Run end-to-end tests
docker-compose -f docker-compose.test.yml up e2e-test-runner

# Run role integration audit
docker-compose -f docker-compose.test.yml up audit-runner
```

## Platform-Specific Instructions

### Linux Setup

Linux is the recommended platform for running IntelliRouter:

```bash
# Install Docker and Docker Compose
sudo apt-get update
sudo apt-get install docker.io docker-compose-plugin

# Start Docker service
sudo systemctl enable docker
sudo systemctl start docker

# Add your user to the docker group (optional, for running without sudo)
sudo usermod -aG docker $USER
# Log out and log back in for this to take effect

# For GPU support (if using Ollama)
# Install NVIDIA Container Toolkit
distribution=$(. /etc/os-release;echo $ID$VERSION_ID)
curl -s -L https://nvidia.github.io/nvidia-docker/gpgkey | sudo apt-key add -
curl -s -L https://nvidia.github.io/nvidia-docker/$distribution/nvidia-docker.list | sudo tee /etc/apt/sources.list.d/nvidia-docker.list
sudo apt-get update
sudo apt-get install -y nvidia-container-toolkit
sudo systemctl restart docker
```

### macOS Setup

macOS requires Docker Desktop:

```bash
# Install Docker Desktop
# Download from https://www.docker.com/products/docker-desktop

# Increase resource allocation in Docker Desktop
# Open Docker Desktop -> Settings -> Resources
# Recommended: 4+ CPUs, 8GB+ RAM, 2GB+ Swap

# Clone the repository and start services
git clone https://github.com/yourusername/intellirouter.git
cd intellirouter
docker-compose up -d

# Note: Ollama GPU acceleration is not available on macOS
# Use CPU-only models or connect to remote Ollama instance
```

### Windows (WSL2) Setup

Windows users should use WSL2 for optimal performance:

```bash
# Install WSL2
wsl --install

# Install Ubuntu on WSL2
wsl --install -d Ubuntu

# Inside WSL2 Ubuntu terminal:
sudo apt-get update
sudo apt-get install docker.io docker-compose-plugin

# Start Docker service
sudo systemctl enable docker
sudo systemctl start docker

# Clone the repository and start services
git clone https://github.com/yourusername/intellirouter.git
cd intellirouter
docker-compose up -d

# For GPU support (if using Ollama)
# Follow NVIDIA CUDA on WSL2 setup guide:
# https://docs.nvidia.com/cuda/wsl-user-guide/index.html
```

## Configuration

### Environment Variables

The system is configured primarily through environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `INTELLIROUTER_ENVIRONMENT` | Environment (production, development, testing) | production |
| `INTELLIROUTER__SERVER__HOST` | Host to bind server | 0.0.0.0 |
| `INTELLIROUTER__SERVER__PORT` | Port to bind server | 8080 |
| `INTELLIROUTER__TELEMETRY__LOG_LEVEL` | Log level (debug, info, warn, error) | info |
| `INTELLIROUTER__MEMORY__BACKEND_TYPE` | Memory backend type | redis |
| `INTELLIROUTER__MEMORY__REDIS_URL` | Redis connection URL | redis://redis:6379 |
| `INTELLIROUTER__IPC__SECURITY__ENABLED` | Enable IPC security | true |
| `INTELLIROUTER__IPC__SECURITY__TOKEN` | IPC security token | *required* |
| `INTELLIROUTER__RAG__ENABLED` | Enable RAG functionality | true |
| `INTELLIROUTER__RAG__VECTOR_DB_URL` | ChromaDB URL | http://chromadb:8000 |
| `INTELLIROUTER__PERSONA_LAYER__ENABLED` | Enable persona layer | true |
| `ANTHROPIC_API_KEY` | Anthropic API key | *optional* |
| `OPENAI_API_KEY` | OpenAI API key | *optional* |
| `GOOGLE_API_KEY` | Google API key | *optional* |
| `MISTRAL_API_KEY` | Mistral API key | *optional* |
| `XAI_API_KEY` | xAI API key | *optional* |
| `AZURE_OPENAI_API_KEY` | Azure OpenAI API key | *optional* |

You can set these variables in a `.env` file or pass them directly to docker-compose.

### Volume Mounts

The system uses several volume mounts for persistence:

| Volume | Purpose |
|--------|---------|
| `./config:/app/config` | Configuration files |
| `./logs/{service}:/app/logs` | Service-specific logs |
| `./data/documents:/app/data/documents` | Document storage for RAG |
| `./data/personas:/app/data/personas` | Persona definitions |
| `redis-data:/data` | Redis data persistence |
| `chroma-data:/chroma/chroma` | ChromaDB vector storage |
| `ollama-data:/root/.ollama` | Ollama model storage |

### Security Considerations

For production deployments:

1. **Change default tokens**: Set a strong `INTELLIROUTER__IPC__SECURITY__TOKEN`
2. **Secure API keys**: Use environment variables or secrets management
3. **Network isolation**: Use Docker networks to isolate services
4. **TLS termination**: Use a reverse proxy (Nginx, Traefik) for TLS
5. **Access control**: Implement authentication for public-facing services

## Scaling

### Horizontal Scaling

You can scale individual services using docker-compose:

```bash
# Scale the router service to 3 instances
docker-compose up -d --scale router=3

# Note: When scaling, you'll need to add a load balancer
```

### Resource Considerations

Each service has different resource requirements:

| Service | CPU | RAM | Disk | Notes |
|---------|-----|-----|------|-------|
| Router | 1-2 cores | 1-2GB | 1GB | Scales with request volume |
| Orchestrator | 2-4 cores | 2-4GB | 1GB | Scales with chain complexity |
| RAG Injector | 2-4 cores | 4-8GB | 10GB+ | Scales with document volume |
| Summarizer | 1-2 cores | 1-2GB | 1GB | Moderate resource usage |
| Redis | 1-2 cores | 2-4GB | 10GB+ | Scales with memory usage |
| ChromaDB | 2-4 cores | 4-8GB | 20GB+ | Scales with vector database size |
| Ollama | 4-8 cores | 8-16GB | 20GB+ | GPU recommended |

### Load Balancing

For high-availability deployments:

1. **Add a load balancer**: Use Nginx, HAProxy, or Traefik
2. **Configure health checks**: Use the `/health` endpoint
3. **Implement sticky sessions**: If needed for stateful operations
4. **Consider Redis Sentinel/Cluster**: For Redis high availability

Example Nginx load balancer configuration:

```nginx
upstream router_backend {
    server intellirouter_router_1:8080;
    server intellirouter_router_2:8080;
    server intellirouter_router_3:8080;
}

server {
    listen 80;
    server_name api.intellirouter.example.com;

    location / {
        proxy_pass http://router_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## Using Ollama for Local Models

To enable Ollama for local model hosting:

1. Uncomment the Ollama service in docker-compose.yml:
   ```yaml
   ollama:
     image: ollama/ollama:latest
     ports:
       - "11434:11434"
     volumes:
       - ollama-data:/root/.ollama
     networks:
       - intellirouter-network
     restart: unless-stopped
     # Uncomment for GPU support
     # deploy:
     #   resources:
     #     reservations:
     #       devices:
     #         - driver: nvidia
     #           count: 1
     #           capabilities: [gpu]
   ```

2. Uncomment the ollama-data volume:
   ```yaml
   volumes:
     redis-data:
     chroma-data:
     ollama-data:  # Uncomment this line
   ```

3. Start the services:
   ```bash
   docker-compose up -d
   ```

4. Pull models:
   ```bash
   # Pull a model
   curl -X POST http://localhost:11434/api/pull -d '{"name": "llama2"}'
   
   # List available models
   curl http://localhost:11434/api/tags
   ```

5. Configure IntelliRouter to use Ollama:
   - Set `INTELLIROUTER__MODEL_REGISTRY__PROVIDERS__OLLAMA__ENABLED=true`
   - Set `INTELLIROUTER__MODEL_REGISTRY__PROVIDERS__OLLAMA__URL=http://ollama:11434`

## Troubleshooting

### Common Issues

1. **Services not starting**:
   - Check logs: `docker-compose logs <service>`
   - Verify resource availability
   - Check for port conflicts

2. **Connection issues between services**:
   - Verify network configuration
   - Check service discovery settings
   - Validate security token configuration

3. **Performance issues**:
   - Check resource utilization: `docker stats`
   - Increase resource allocation
   - Consider scaling horizontally

4. **ChromaDB errors**:
   - Verify ChromaDB is running: `curl http://localhost:8000/api/v1/heartbeat`
   - Check disk space for vector storage
   - Increase memory allocation

5. **Redis errors**:
   - Check Redis connection: `docker exec -it intellirouter_redis_1 redis-cli ping`
   - Verify Redis URL configuration
   - Check Redis memory usage

### Logs and Debugging

Access logs for troubleshooting:

```bash
# View logs for all services
docker-compose logs

# View logs for a specific service
docker-compose logs router

# Follow logs in real-time
docker-compose logs -f

# View logs with timestamps
docker-compose logs --timestamps
```

## Performance Tuning

Optimize performance based on your workload:

1. **Redis optimization**:
   - Adjust `maxmemory` setting
   - Configure appropriate eviction policy
   - Consider Redis persistence options

2. **ChromaDB optimization**:
   - Tune embedding dimensions
   - Adjust similarity search parameters
   - Consider index optimization

3. **Network optimization**:
   - Use host networking for better performance
   - Optimize Docker DNS resolution
   - Consider using Docker Compose v2 for improved networking

4. **Resource allocation**:
   - Allocate resources based on workload
   - Monitor resource usage and adjust
   - Consider dedicated hosts for resource-intensive services

5. **Ollama optimization**:
   - Use GPU acceleration when available
   - Select appropriate model size for hardware
   - Adjust context window based on use case