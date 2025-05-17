# Edge Deployment

This directory contains configuration for deploying IntelliRouter on edge devices.

## Requirements

- Docker
- Docker Compose

## Deployment

1. Build and start the services:

```bash
docker-compose up -d
```

2. Stop the services:

```bash
docker-compose down
```

## Configuration

The edge deployment uses a simplified setup with all IntelliRouter components running in a single container. It includes:

- IntelliRouter running in "all" role mode
- Redis for memory storage
- Volume mounts for configuration and data persistence

## Customization

You can customize the deployment by:

1. Modifying environment variables in the `docker-compose.yml` file
2. Adding API keys through environment variables or a `.env` file
3. Adjusting the Redis configuration
4. Changing the exposed ports

## Resource Considerations

The edge deployment is designed to run on devices with limited resources. Consider the following:

- Adjust the Redis persistence settings based on your storage constraints
- Monitor memory usage and adjust as needed
- For very constrained devices, consider disabling certain features through configuration