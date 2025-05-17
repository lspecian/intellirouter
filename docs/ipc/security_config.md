# IntelliRouter IPC Security Configuration

This document provides detailed information about the security configuration for the IntelliRouter IPC infrastructure.

## Table of Contents

- [Overview](#overview)
- [JWT Authentication](#jwt-authentication)
  - [Configuration](#jwt-configuration)
  - [Token Generation](#token-generation)
  - [Token Validation](#token-validation)
  - [Role-Based Access Control](#role-based-access-control)
- [Mutual TLS (mTLS)](#mutual-tls-mtls)
  - [Certificate Generation](#certificate-generation)
  - [Configuration](#mtls-configuration)
  - [Client Configuration](#client-configuration)
  - [Server Configuration](#server-configuration)
- [Combining JWT and mTLS](#combining-jwt-and-mtls)
- [Secure Redis Pub/Sub](#secure-redis-pubsub)
- [Best Practices](#best-practices)

## Overview

The IntelliRouter IPC infrastructure provides two main security mechanisms:

1. **JWT-based Authentication**: Ensures that only authorized services can communicate with each other.
2. **Mutual TLS (mTLS)**: Provides encrypted communication between services and verifies the identity of both client and server.

These security mechanisms can be used independently or together for enhanced security.

## JWT Authentication

JWT (JSON Web Token) authentication is used to verify the identity of services and control access to resources based on roles.

### JWT Configuration

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

The `JwtConfig` struct has the following fields:

| Field | Description |
|-------|-------------|
| `secret` | Secret key used to sign JWT tokens |
| `issuer` | Issuer of the JWT token (typically the service name) |
| `audience` | Audience of the JWT token (typically the service name) |
| `expiration_seconds` | Token expiration time in seconds |

### Token Generation

To generate a JWT token for a service, use the `generate_token` method of the `JwtAuthenticator`:

```rust
let service_name = "chain_engine";
let roles = vec!["execute_chain".to_string(), "cancel_chain".to_string()];

let token = jwt_authenticator.generate_token(service_name, roles)?;
```

The generated token contains the following claims:

```json
{
  "sub": "chain_engine",
  "iss": "intellirouter",
  "aud": "intellirouter-services",
  "exp": 1620000000,
  "iat": 1619996400,
  "roles": ["execute_chain", "cancel_chain"]
}
```

### Token Validation

To validate a JWT token, use the `validate_token` method of the `JwtAuthenticator`:

```rust
let claims = jwt_authenticator.validate_token(&token)?;

// Check if the service has a specific role
if claims.roles.contains(&"execute_chain".to_string()) {
    // Allow the service to execute chains
}
```

The validation process checks the following:

1. Token signature is valid
2. Token is not expired
3. Token issuer matches the expected issuer
4. Token audience matches the expected audience

### Role-Based Access Control

You can define roles for services and check if a service has a specific role:

```rust
use intellirouter::modules::ipc::security::RoleConfig;

let mut role_config = RoleConfig::new();
role_config.add_role("chain_engine", "execute_chain");
role_config.add_role("chain_engine", "cancel_chain");
role_config.add_role("router_core", "route_request");

// Check if a service has a role
if role_config.has_role("chain_engine", "execute_chain") {
    // Allow the service to execute chains
}
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

### mTLS Configuration

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

The `TlsConfig` struct has the following fields:

| Field | Description |
|-------|-------------|
| `cert_path` | Path to the certificate file |
| `key_path` | Path to the private key file |
| `ca_cert_path` | Path to the CA certificate file |

### Client Configuration

To configure a gRPC client with mTLS, use the `load_client_config` method of the `TlsConfig`:

```rust
let client_tls_config = tls_config.load_client_config("intellirouter.local")?;

let channel = tonic::transport::Channel::from_static("https://intellirouter.local:50051")
    .tls_config(client_tls_config)?
    .connect()
    .await?;
```

Alternatively, you can use the `SecureGrpcClientBuilder` to create a secure gRPC client:

```rust
use intellirouter::modules::ipc::security::SecureGrpcClientBuilder;
use intellirouter::generated::intellirouter::chain_engine::v1::chain_engine_client::ChainEngineClient;

let client = SecureGrpcClientBuilder::<ChainEngineClient<_>>::new("intellirouter.local")
    .with_tls(tls_config.clone())
    .build("https://intellirouter.local:50051")
    .await?;
```

### Server Configuration

To configure a gRPC server with mTLS, use the `load_server_config` method of the `TlsConfig`:

```rust
let server_tls_config = tls_config.load_server_config()?;

let server = tonic::transport::Server::builder()
    .tls_config(server_tls_config)?
    .add_service(chain_engine_server::ChainEngineServer::new(service))
    .serve("0.0.0.0:50051".parse()?)
    .await?;
```

Alternatively, you can use the `SecureGrpcServerBuilder` to create a secure gRPC server:

```rust
use intellirouter::modules::ipc::security::SecureGrpcServerBuilder;
use intellirouter::generated::intellirouter::chain_engine::v1::chain_engine_server::{ChainEngine, ChainEngineServer};

let service = ChainEngineServer::new(your_implementation);

let server_builder = SecureGrpcServerBuilder::new()
    .with_tls(tls_config.clone());

let router = server_builder.build::<ChainEngineServer<_>>()?;
let router = server_builder.add_service(router, service);

let addr = "[::1]:50051".parse()?;
router.serve(addr).await?;
```

## Combining JWT and mTLS

For maximum security, you can combine JWT authentication and mTLS:

```rust
// Secure gRPC client with JWT and mTLS
let client = SecureGrpcClientBuilder::<ChainEngineClient<_>>::new("intellirouter.local")
    .with_tls(tls_config.clone())
    .with_jwt(
        jwt_authenticator.clone(),
        "router_core".to_string(),
        vec!["route_requests".to_string()],
    )
    .build("https://intellirouter.local:50051")
    .await?;

// Secure gRPC server with JWT and mTLS
let server_builder = SecureGrpcServerBuilder::new()
    .with_tls(tls_config.clone())
    .with_jwt(
        jwt_authenticator.clone(),
        vec!["execute_chain".to_string(), "cancel_chain".to_string()],
    );
```

## Secure Redis Pub/Sub

To secure Redis Pub/Sub communication, you can use the `AuthenticatedRedisClient` and `SecureRedisClientBuilder`:

```rust
// Create a secure Redis client
let redis_client = SecureRedisClientBuilder::new()
    .with_tls(tls_config.clone())
    .with_jwt(
        jwt_authenticator.clone(),
        "chain_engine".to_string(),
        vec!["publish_events".to_string()],
    )
    .build("rediss://localhost:6379")
    .await?;

// Publish a message
redis_client.publish("intellirouter:chain_engine:router_core:chain_execution_completed", &payload).await?;

// Subscribe to a channel
let subscription = redis_client.subscribe("intellirouter:chain_engine:router_core:chain_execution_completed").await?;
```

The `AuthenticatedRedisClient` wraps messages with authentication information:

```rust
struct AuthenticatedMessage {
    token: String,
    payload: Vec<u8>,
}
```

When a message is received, the token is validated before the message is delivered to the subscriber.

## Best Practices

### JWT Authentication

1. **Use Strong Secrets**: Use a strong, randomly generated secret key for JWT authentication.
2. **Rotate Keys Regularly**: Regularly rotate JWT secret keys to minimize the impact of key compromise.
3. **Limit Token Expiration**: Set a reasonable expiration time for JWT tokens to minimize the impact of token theft.
4. **Use Role-Based Access Control**: Define specific roles for each service and restrict access based on these roles.
5. **Validate Tokens**: Always validate tokens before processing requests.
6. **Handle Validation Errors**: Handle token validation errors gracefully and provide meaningful error messages.
7. **Log Authentication Failures**: Log authentication failures to help detect potential attacks.

### Mutual TLS (mTLS)

1. **Protect Private Keys**: Keep private keys secure and restrict access to them.
2. **Use Strong Certificates**: Use strong certificates with at least 2048-bit keys.
3. **Validate Certificates**: Always validate certificates against a trusted CA.
4. **Rotate Certificates**: Regularly rotate certificates to minimize the impact of key compromise.
5. **Use Certificate Revocation Lists (CRLs)**: Use CRLs to revoke compromised certificates.
6. **Use Online Certificate Status Protocol (OCSP)**: Use OCSP to check certificate status in real-time.
7. **Log TLS Errors**: Log TLS errors to help detect potential attacks.

### General Security

1. **Defense in Depth**: Use multiple security mechanisms to provide defense in depth.
2. **Principle of Least Privilege**: Grant services only the permissions they need to function.
3. **Secure Configuration**: Store security configuration securely and restrict access to it.
4. **Regular Auditing**: Regularly audit security configuration and logs to detect potential issues.
5. **Keep Dependencies Updated**: Keep security-related dependencies updated to address known vulnerabilities.
6. **Security Testing**: Regularly test security mechanisms to ensure they are functioning correctly.
7. **Incident Response Plan**: Have a plan for responding to security incidents.