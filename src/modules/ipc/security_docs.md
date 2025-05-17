# IntelliRouter IPC Security Documentation

This document provides guidance on configuring and using the security features of the IntelliRouter IPC infrastructure.

## Overview

The IntelliRouter IPC security infrastructure provides two main security mechanisms:

1. **JWT-based Authentication**: Ensures that only authorized services can communicate with each other.
2. **Mutual TLS (mTLS)**: Provides encrypted communication between services and verifies the identity of both client and server.

These security mechanisms can be used independently or together for enhanced security.

## JWT Authentication

JWT (JSON Web Token) authentication is used to verify the identity of services and control access to resources based on roles.

### Configuration

To configure JWT authentication, you need to create a `JwtConfig` object:

```rust
use intellirouter::modules::ipc::security::{JwtConfig, JwtAuthenticator};
use std::sync::Arc;

let jwt_config = JwtConfig {
    secret: "your-secret-key".to_string(),
    issuer: "intellirouter".to_string(),
    audience: "intellirouter-services".to_string(),
    expiration_seconds: 3600, // 1 hour
};

let jwt_authenticator = Arc::new(JwtAuthenticator::new(jwt_config));
```

### Role-Based Access Control

You can define roles for services and check if a service has a specific role:

```rust
use intellirouter::modules::ipc::security::RoleConfig;

let mut role_config = RoleConfig::new();
role_config.add_role("model_registry", "read_models");
role_config.add_role("model_registry", "write_models");
role_config.add_role("router_core", "route_requests");

// Check if a service has a role
if role_config.has_role("model_registry", "read_models") {
    // Allow access to read models
}
```

### Secure gRPC Client

To create a secure gRPC client with JWT authentication:

```rust
use intellirouter::modules::ipc::security::SecureGrpcClientBuilder;
use intellirouter::generated::intellirouter::model_registry::v1::model_registry_client::ModelRegistryClient;

let client = SecureGrpcClientBuilder::<ModelRegistryClient>::new("intellirouter.local")
    .with_jwt(
        jwt_authenticator.clone(),
        "router_core".to_string(),
        vec!["route_requests".to_string()],
    )
    .build("http://localhost:50051")
    .await?;
```

### Secure gRPC Server

To create a secure gRPC server with JWT authentication:

```rust
use intellirouter::modules::ipc::security::SecureGrpcServerBuilder;
use intellirouter::generated::intellirouter::model_registry::v1::model_registry_server::{ModelRegistry, ModelRegistryServer};

let service = ModelRegistryServer::new(your_implementation);

let server_builder = SecureGrpcServerBuilder::new()
    .with_jwt(
        jwt_authenticator.clone(),
        vec!["read_models".to_string(), "write_models".to_string()],
    );

let router = server_builder.build::<ModelRegistryServer<_>>()?;
let router = server_builder.add_service(router, service);

let addr = "[::1]:50051".parse().unwrap();
router.serve(addr).await?;
```

### Secure Redis Client

To create a secure Redis client with JWT authentication:

```rust
use intellirouter::modules::ipc::security::SecureRedisClientBuilder;
use intellirouter::modules::ipc::redis_pubsub::RedisClient;

let redis_client = SecureRedisClientBuilder::new()
    .with_jwt(
        jwt_authenticator.clone(),
        "router_core".to_string(),
        vec!["route_requests".to_string()],
    )
    .build("redis://localhost:6379")
    .await?;

// Publish a message
redis_client.publish("channel", b"message").await?;

// Subscribe to a channel
let subscription = redis_client.subscribe("channel").await?;
```

## Mutual TLS (mTLS)

Mutual TLS (mTLS) provides encrypted communication and verifies the identity of both client and server using certificates.

### Certificate Generation

Before using mTLS, you need to generate certificates for your services. Here's a simple example using OpenSSL:

```bash
# Generate CA key and certificate
openssl genrsa -out ca.key 4096
openssl req -new -x509 -key ca.key -sha256 -subj "/CN=intellirouter-ca" -out ca.crt -days 365

# Generate server key and certificate signing request (CSR)
openssl genrsa -out server.key 4096
openssl req -new -key server.key -out server.csr -config <(
cat <<-EOF
[req]
default_bits = 4096
prompt = no
default_md = sha256
req_extensions = req_ext
distinguished_name = dn

[dn]
CN = intellirouter.local

[req_ext]
subjectAltName = @alt_names

[alt_names]
DNS.1 = intellirouter.local
DNS.2 = localhost
EOF
)

# Sign the server certificate with the CA
openssl x509 -req -in server.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out server.crt -days 365 -sha256 -extfile <(
cat <<-EOF
subjectAltName = @alt_names

[alt_names]
DNS.1 = intellirouter.local
DNS.2 = localhost
EOF
)

# Generate client key and certificate signing request (CSR)
openssl genrsa -out client.key 4096
openssl req -new -key client.key -out client.csr -subj "/CN=intellirouter-client"

# Sign the client certificate with the CA
openssl x509 -req -in client.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out client.crt -days 365 -sha256
```

### Configuration

To configure mTLS, you need to create a `TlsConfig` object:

```rust
use intellirouter::modules::ipc::security::TlsConfig;
use std::path::PathBuf;

let tls_config = TlsConfig::new(
    PathBuf::from("path/to/cert.crt"),
    PathBuf::from("path/to/key.key"),
    PathBuf::from("path/to/ca.crt"),
);
```

### Secure gRPC Client with mTLS

To create a secure gRPC client with mTLS:

```rust
use intellirouter::modules::ipc::security::SecureGrpcClientBuilder;
use intellirouter::generated::intellirouter::model_registry::v1::model_registry_client::ModelRegistryClient;

let client = SecureGrpcClientBuilder::<ModelRegistryClient>::new("intellirouter.local")
    .with_tls(tls_config.clone())
    .build("https://localhost:50051")
    .await?;
```

### Secure gRPC Server with mTLS

To create a secure gRPC server with mTLS:

```rust
use intellirouter::modules::ipc::security::SecureGrpcServerBuilder;
use intellirouter::generated::intellirouter::model_registry::v1::model_registry_server::{ModelRegistry, ModelRegistryServer};

let service = ModelRegistryServer::new(your_implementation);

let server_builder = SecureGrpcServerBuilder::new()
    .with_tls(tls_config.clone());

let router = server_builder.build::<ModelRegistryServer<_>>()?;
let router = server_builder.add_service(router, service);

let addr = "[::1]:50051".parse().unwrap();
router.serve(addr).await?;
```

### Secure Redis Client with mTLS

To create a secure Redis client with mTLS:

```rust
use intellirouter::modules::ipc::security::SecureRedisClientBuilder;
use intellirouter::modules::ipc::redis_pubsub::RedisClient;

let redis_client = SecureRedisClientBuilder::new()
    .with_tls(tls_config.clone())
    .build("rediss://localhost:6379")
    .await?;
```

## Combining JWT and mTLS

For maximum security, you can combine JWT authentication and mTLS:

```rust
// Secure gRPC client with JWT and mTLS
let client = SecureGrpcClientBuilder::<ModelRegistryClient>::new("intellirouter.local")
    .with_tls(tls_config.clone())
    .with_jwt(
        jwt_authenticator.clone(),
        "router_core".to_string(),
        vec!["route_requests".to_string()],
    )
    .build("https://localhost:50051")
    .await?;

// Secure gRPC server with JWT and mTLS
let server_builder = SecureGrpcServerBuilder::new()
    .with_tls(tls_config.clone())
    .with_jwt(
        jwt_authenticator.clone(),
        vec!["read_models".to_string(), "write_models".to_string()],
    );

// Secure Redis client with JWT and mTLS
let redis_client = SecureRedisClientBuilder::new()
    .with_tls(tls_config.clone())
    .with_jwt(
        jwt_authenticator.clone(),
        "router_core".to_string(),
        vec!["route_requests".to_string()],
    )
    .build("rediss://localhost:6379")
    .await?;
```

## Best Practices

1. **Use Strong Secrets**: Use a strong, randomly generated secret key for JWT authentication.
2. **Rotate Keys Regularly**: Regularly rotate JWT secret keys and TLS certificates.
3. **Limit Token Expiration**: Set a reasonable expiration time for JWT tokens.
4. **Protect Certificates**: Keep private keys and certificates secure.
5. **Use Role-Based Access Control**: Define specific roles for each service and restrict access based on these roles.
6. **Validate Certificates**: Always validate certificates against a trusted CA.
7. **Monitor Authentication Failures**: Log and monitor authentication failures to detect potential attacks.
8. **Use Both JWT and mTLS**: For critical services, use both JWT authentication and mTLS for enhanced security.