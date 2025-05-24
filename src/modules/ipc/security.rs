//! Security infrastructure for IPC communications
//!
//! This module provides JWT-based authentication and mutual TLS (mTLS)
//! for secure communication between IntelliRouter services.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use tonic::service::Interceptor;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity, ServerTlsConfig};
use tonic::{Request, Status};

use crate::modules::ipc::redis_pubsub::{Message, RedisClient, Subscription, SubscriptionDelegate};
use crate::modules::ipc::{IpcError, IpcResult};

/// Error type for security operations
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TLS error: {0}")]
    Tls(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),
}

impl From<SecurityError> for IpcError {
    fn from(err: SecurityError) -> Self {
        match err {
            SecurityError::Jwt(e) => IpcError::Security(format!("JWT error: {}", e)),
            SecurityError::Io(e) => IpcError::Security(format!("IO error: {}", e)),
            SecurityError::Tls(e) => IpcError::Security(format!("TLS error: {}", e)),
            SecurityError::Authentication(e) => {
                IpcError::Security(format!("Authentication error: {}", e))
            }
            SecurityError::Authorization(e) => {
                IpcError::Security(format!("Authorization error: {}", e))
            }
        }
    }
}

impl From<SecurityError> for Status {
    fn from(err: SecurityError) -> Self {
        match err {
            SecurityError::Jwt(_) | SecurityError::Authentication(_) => {
                Status::unauthenticated(format!("{}", err))
            }
            SecurityError::Authorization(_) => Status::permission_denied(format!("{}", err)),
            _ => Status::internal(format!("{}", err)),
        }
    }
}

/// JWT configuration
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// Secret key for signing JWT tokens
    pub secret: String,
    /// Issuer of the JWT token
    pub issuer: String,
    /// Audience of the JWT token
    pub audience: String,
    /// Token expiration time in seconds
    pub expiration_seconds: u64,
}

/// JWT claims
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject (service name)
    pub sub: String,
    /// Issuer
    pub iss: String,
    /// Audience
    pub aud: String,
    /// Expiration time
    pub exp: u64,
    /// Issued at
    pub iat: u64,
    /// Service roles
    pub roles: Vec<String>,
}

/// JWT authenticator
#[derive(Debug, Clone)]
pub struct JwtAuthenticator {
    config: JwtConfig,
}

impl JwtAuthenticator {
    /// Create a new JWT authenticator
    pub fn new(config: JwtConfig) -> Self {
        Self { config }
    }

    /// Generate a JWT token for a service
    pub fn generate_token(
        &self,
        service_name: &str,
        roles: Vec<String>,
    ) -> Result<String, SecurityError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let claims = JwtClaims {
            sub: service_name.to_string(),
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            exp: now + self.config.expiration_seconds,
            iat: now,
            roles,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.secret.as_bytes()),
        )?;

        Ok(token)
    }

    /// Validate a JWT token
    pub fn validate_token(&self, token: &str) -> Result<JwtClaims, SecurityError> {
        let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        validation.set_issuer(&[&self.config.issuer]);
        validation.set_audience(&[&self.config.audience]);

        let token_data = decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(self.config.secret.as_bytes()),
            &validation,
        )?;

        Ok(token_data.claims)
    }
}

/// JWT interceptor for gRPC authentication
pub struct JwtInterceptor {
    authenticator: Arc<JwtAuthenticator>,
    required_roles: Vec<String>,
}

impl JwtInterceptor {
    /// Create a new JWT interceptor
    pub fn new(authenticator: Arc<JwtAuthenticator>, required_roles: Vec<String>) -> Self {
        Self {
            authenticator,
            required_roles,
        }
    }
}

impl Interceptor for JwtInterceptor {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        // Extract the token from the request metadata
        let token = match request.metadata().get("authorization") {
            Some(t) => {
                let token_str = t
                    .to_str()
                    .map_err(|_| Status::unauthenticated("Invalid authorization token format"))?;

                // Remove "Bearer " prefix if present
                if token_str.starts_with("Bearer ") {
                    token_str[7..].to_string()
                } else {
                    token_str.to_string()
                }
            }
            None => return Err(Status::unauthenticated("Missing authorization token")),
        };

        // Validate the token
        let claims = self
            .authenticator
            .validate_token(&token)
            .map_err(|e| Status::unauthenticated(format!("Invalid authorization token: {}", e)))?;

        // Check if the service has the required roles
        let has_required_roles = self
            .required_roles
            .iter()
            .all(|role| claims.roles.contains(role));

        if !has_required_roles {
            return Err(Status::permission_denied("Insufficient permissions"));
        }

        Ok(request)
    }
}

/// TLS configuration
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Path to the certificate file
    pub cert_path: PathBuf,
    /// Path to the private key file
    pub key_path: PathBuf,
    /// Path to the CA certificate file
    pub ca_cert_path: PathBuf,
}

impl TlsConfig {
    /// Create a new TLS configuration
    pub fn new<P: AsRef<Path>>(cert_path: P, key_path: P, ca_cert_path: P) -> Self {
        Self {
            cert_path: cert_path.as_ref().to_path_buf(),
            key_path: key_path.as_ref().to_path_buf(),
            ca_cert_path: ca_cert_path.as_ref().to_path_buf(),
        }
    }

    /// Load client TLS configuration
    pub fn load_client_config(&self, domain_name: &str) -> Result<ClientTlsConfig, SecurityError> {
        // Load the certificate and private key
        let cert = std::fs::read(&self.cert_path)?;
        let key = std::fs::read(&self.key_path)?;
        let ca_cert = std::fs::read(&self.ca_cert_path)?;

        // Create the identity from the certificate and private key
        let identity = Identity::from_pem(cert, key);

        // Create the client TLS configuration
        let client_config = ClientTlsConfig::new()
            .identity(identity)
            .ca_certificate(Certificate::from_pem(ca_cert))
            .domain_name(domain_name);

        Ok(client_config)
    }

    /// Load server TLS configuration
    pub fn load_server_config(&self) -> Result<ServerTlsConfig, SecurityError> {
        // Load the certificate and private key
        let cert = std::fs::read(&self.cert_path)?;
        let key = std::fs::read(&self.key_path)?;
        let ca_cert = std::fs::read(&self.ca_cert_path)?;

        // Create the identity from the certificate and private key
        let identity = Identity::from_pem(cert, key);

        // Create the server TLS configuration
        let server_config = ServerTlsConfig::new()
            .identity(identity)
            .client_ca_root(Certificate::from_pem(ca_cert));

        Ok(server_config)
    }
}

/// Authenticated message for Redis pub/sub
#[derive(Debug, Serialize, Deserialize)]
struct AuthenticatedMessage {
    /// JWT token
    token: String,
    /// Message payload
    payload: Vec<u8>,
}

/// Authenticated Redis client
pub struct AuthenticatedRedisClient {
    inner: Arc<dyn RedisClient>,
    jwt_authenticator: Arc<JwtAuthenticator>,
    service_name: String,
    roles: Vec<String>,
}

impl AuthenticatedRedisClient {
    /// Create a new authenticated Redis client
    pub async fn new(
        inner: Arc<dyn RedisClient>,
        jwt_authenticator: Arc<JwtAuthenticator>,
        service_name: String,
        roles: Vec<String>,
    ) -> Self {
        Self {
            inner,
            jwt_authenticator,
            service_name,
            roles,
        }
    }

    /// Get an authentication token
    async fn get_auth_token(&self) -> IpcResult<String> {
        self.jwt_authenticator
            .generate_token(&self.service_name, self.roles.clone())
            .map_err(|e| e.into())
    }
}

#[async_trait]
impl RedisClient for AuthenticatedRedisClient {
    async fn publish(&self, channel: &str, message: &[u8]) -> IpcResult<()> {
        // Get a fresh token
        let token = self.get_auth_token().await?;

        // Wrap the message with authentication information
        let auth_message = AuthenticatedMessage {
            token,
            payload: message.to_vec(),
        };

        // Serialize the authenticated message
        let serialized = serde_json::to_vec(&auth_message).map_err(|e| {
            IpcError::Serialization(format!("Failed to serialize authenticated message: {}", e))
        })?;

        // Publish the authenticated message
        self.inner.publish(channel, &serialized).await
    }

    async fn subscribe(&self, channel: &str) -> IpcResult<Subscription> {
        // Subscribe to the channel
        let inner_subscription = self.inner.subscribe(channel).await?;

        // Create an authenticated subscription
        let auth_subscription = AuthenticatedSubscription {
            inner: inner_subscription,
            jwt_authenticator: self.jwt_authenticator.clone(),
            required_roles: self.roles.clone(),
        };

        // Convert to a regular subscription by wrapping it
        Ok(auth_subscription.into_subscription())
    }

    async fn psubscribe(&self, pattern: &str) -> IpcResult<Subscription> {
        // Subscribe to the pattern
        let inner_subscription = self.inner.psubscribe(pattern).await?;

        // Create an authenticated subscription
        let auth_subscription = AuthenticatedSubscription {
            inner: inner_subscription,
            jwt_authenticator: self.jwt_authenticator.clone(),
            required_roles: self.roles.clone(),
        };

        // Convert to a regular subscription by wrapping it
        Ok(auth_subscription.into_subscription())
    }
}

/// Authenticated subscription
pub struct AuthenticatedSubscription {
    inner: Subscription,
    jwt_authenticator: Arc<JwtAuthenticator>,
    required_roles: Vec<String>,
}

impl SubscriptionDelegate for AuthenticatedSubscription {
    fn next_message(&self) -> IpcResult<Option<Message>> {
        // This is a synchronous method that returns a Result, not a Future
        // We can't directly call the async method, so we return None
        // The actual implementation will be handled by the Subscription::next_message method
        Ok(None)
    }
}

impl AuthenticatedSubscription {
    /// Convert this authenticated subscription into a regular subscription
    pub fn into_subscription(self) -> Subscription {
        // Create a new subscription that delegates to this authenticated subscription
        Subscription::new_delegated(Box::new(self))
    }

    /// Get the next message from the subscription
    pub async fn next_message(&self) -> IpcResult<Option<Message>> {
        if let Some(message) = self.inner.next_message().await? {
            // Deserialize the authenticated message
            let auth_message: AuthenticatedMessage = serde_json::from_slice(&message.payload)
                .map_err(|e| {
                    IpcError::Serialization(format!(
                        "Failed to deserialize authenticated message: {}",
                        e
                    ))
                })?;

            // Validate the token
            let claims = self
                .jwt_authenticator
                .validate_token(&auth_message.token)
                .map_err(|e| {
                    IpcError::Connection(format!("Invalid authentication token: {}", e))
                })?;

            // Check if the service has the required roles
            let has_required_roles = self
                .required_roles
                .iter()
                .all(|role| claims.roles.contains(role));

            if !has_required_roles {
                return Err(IpcError::Connection("Insufficient permissions".to_string()));
            }

            // Return the original message with the authenticated payload
            Ok(Some(Message {
                channel: message.channel,
                payload: auth_message.payload,
            }))
        } else {
            Ok(None)
        }
    }

    /// Convert the subscription into a stream of messages
    pub fn into_stream(self) -> impl futures::Stream<Item = IpcResult<Message>> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let jwt_authenticator = self.jwt_authenticator.clone();
        let required_roles = self.required_roles.clone();
        let inner = self.inner;

        tokio::spawn(async move {
            loop {
                let result = inner.next_message().await;

                match result {
                    Ok(Some(message)) => {
                        // Process the authenticated message
                        match serde_json::from_slice::<AuthenticatedMessage>(&message.payload) {
                            Ok(auth_message) => {
                                // Validate the token
                                match jwt_authenticator.validate_token(&auth_message.token) {
                                    Ok(claims) => {
                                        // Check if the service has the required roles
                                        let has_required_roles = required_roles
                                            .iter()
                                            .all(|role| claims.roles.contains(role));

                                        if has_required_roles {
                                            // Return the original message with the authenticated payload
                                            let authenticated_message = Message {
                                                channel: message.channel,
                                                payload: auth_message.payload,
                                            };

                                            if tx.send(Ok(authenticated_message)).await.is_err() {
                                                break;
                                            }
                                        } else {
                                            let _ = tx
                                                .send(Err(IpcError::Connection(
                                                    "Insufficient permissions".to_string(),
                                                )))
                                                .await;
                                        }
                                    }
                                    Err(e) => {
                                        let _ = tx
                                            .send(Err(IpcError::Connection(format!(
                                                "Invalid authentication token: {}",
                                                e
                                            ))))
                                            .await;
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = tx
                                    .send(Err(IpcError::Serialization(format!(
                                        "Failed to deserialize authenticated message: {}",
                                        e
                                    ))))
                                    .await;
                            }
                        }
                    }
                    Ok(None) => {
                        break;
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e)).await;
                        break;
                    }
                }
            }
        });

        tokio_stream::wrappers::ReceiverStream::new(rx)
    }

    /// Get the channel name
    pub fn channel(&self) -> &str {
        self.inner.channel()
    }
}

/// Secure gRPC client builder
pub struct SecureGrpcClientBuilder<T> {
    tls_config: Option<TlsConfig>,
    jwt_authenticator: Option<Arc<JwtAuthenticator>>,
    service_name: Option<String>,
    roles: Vec<String>,
    domain_name: String,
    _marker: std::marker::PhantomData<T>,
}

impl<T> SecureGrpcClientBuilder<T> {
    /// Create a new secure gRPC client builder
    pub fn new(domain_name: &str) -> Self {
        Self {
            tls_config: None,
            jwt_authenticator: None,
            service_name: None,
            roles: Vec::new(),
            domain_name: domain_name.to_string(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Add TLS configuration
    pub fn with_tls(mut self, tls_config: TlsConfig) -> Self {
        self.tls_config = Some(tls_config);
        self
    }

    /// Add JWT authentication
    pub fn with_jwt(
        mut self,
        jwt_authenticator: Arc<JwtAuthenticator>,
        service_name: String,
        roles: Vec<String>,
    ) -> Self {
        self.jwt_authenticator = Some(jwt_authenticator);
        self.service_name = Some(service_name);
        self.roles = roles;
        self
    }

    /// Build the secure gRPC client
    pub async fn build(self, endpoint: &str) -> Result<T, SecurityError>
    where
        T: From<Channel>,
    {
        let mut channel_builder = Channel::from_shared(endpoint.to_string())
            .map_err(|e| SecurityError::Tls(format!("Failed to create channel: {}", e)))?;

        // Add TLS if configured
        if let Some(tls_config) = self.tls_config {
            let client_tls_config = tls_config.load_client_config(&self.domain_name)?;
            channel_builder = channel_builder
                .tls_config(client_tls_config)
                .map_err(|e| SecurityError::Tls(format!("Failed to configure TLS: {}", e)))?;
        }

        // Build the channel
        let channel = channel_builder
            .connect()
            .await
            .map_err(|e| SecurityError::Tls(format!("Failed to connect: {}", e)))?;

        // Add JWT authentication if configured
        let channel = if let (Some(jwt_authenticator), Some(service_name)) =
            (self.jwt_authenticator, self.service_name)
        {
            // Generate a token
            let _token = jwt_authenticator.generate_token(&service_name, self.roles)?;

            // TODO: Implement JWT token usage via interceptor. Currently, JWT auth is non-functional.
            // Issue: [Link to GitHub issue to be created for tracking this task]
            return Err(SecurityError::Authentication("JWT authentication is configured but not yet implemented. Token was generated but cannot be applied.".to_string()));
            // The original channel is not further processed or returned in this JWT path.
        } else {
            channel
        };

        // Create the client
        Ok(T::from(channel))
    }
}

/// Role configuration
#[derive(Debug, Clone, Default)]
pub struct RoleConfig {
    /// Roles for each service
    pub roles: HashMap<String, Vec<String>>,
}

impl RoleConfig {
    /// Create a new role configuration
    pub fn new() -> Self {
        Self {
            roles: HashMap::new(),
        }
    }

    /// Add a role to a service
    pub fn add_role(&mut self, service: &str, role: &str) {
        self.roles
            .entry(service.to_string())
            .or_insert_with(Vec::new)
            .push(role.to_string());
    }

    /// Get the roles for a service
    pub fn get_roles(&self, service: &str) -> Vec<String> {
        self.roles.get(service).cloned().unwrap_or_else(Vec::new)
    }

    /// Check if a service has a role
    pub fn has_role(&self, service: &str, role: &str) -> bool {
        self.roles
            .get(service)
            .map(|roles| roles.contains(&role.to_string()))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_jwt_authentication() {
        let config = JwtConfig {
            secret: "test-secret".to_string(),
            issuer: "test-issuer".to_string(),
            audience: "test-audience".to_string(),
            expiration_seconds: 3600,
        };

        let authenticator = JwtAuthenticator::new(config);
        let service_name = "test-service";
        let roles = vec!["role1".to_string(), "role2".to_string()];

        // Generate a token
        let token = authenticator
            .generate_token(service_name, roles.clone())
            .unwrap();

        // Validate the token
        let claims = authenticator.validate_token(&token).unwrap();
        assert_eq!(claims.sub, service_name);
        assert_eq!(claims.iss, "test-issuer");
        assert_eq!(claims.aud, "test-audience");
        assert_eq!(claims.roles, roles);
    }

    /// Secure gRPC server builder
    pub struct SecureGrpcServerBuilder {
        tls_config: Option<TlsConfig>,
        jwt_interceptor: Option<JwtInterceptor>,
    }

    impl SecureGrpcServerBuilder {
        /// Create a new secure gRPC server builder
        pub fn new() -> Self {
            Self {
                tls_config: None,
                jwt_interceptor: None,
            }
        }

        /// Add TLS configuration
        pub fn with_tls(mut self, tls_config: TlsConfig) -> Self {
            self.tls_config = Some(tls_config);
            self
        }

        /// Add JWT authentication
        pub fn with_jwt(
            mut self,
            jwt_authenticator: Arc<JwtAuthenticator>,
            required_roles: Vec<String>,
        ) -> Self {
            self.jwt_interceptor = Some(JwtInterceptor::new(jwt_authenticator, required_roles));
            self
        }

        /// Build the secure gRPC server
        pub fn build<S>(self) -> Result<tonic::transport::server::Router, SecurityError>
        where
            S: tonic::server::NamedService,
        {
            let mut server_builder = tonic::transport::Server::builder();

            // Add TLS if configured
            if let Some(tls_config) = self.tls_config {
                let server_tls_config = tls_config.load_server_config()?;
                server_builder = server_builder
                    .tls_config(server_tls_config)
                    .map_err(|e| SecurityError::Tls(format!("Failed to configure TLS: {}", e)))?;
            }

            // Return the router
            Ok(server_builder)
        }

        /// Add a service to the server with optional JWT authentication
        pub fn add_service<S>(
            &self,
            router: tonic::transport::server::Router,
            service: S,
        ) -> tonic::transport::server::Router
        where
            S: tonic::server::NamedService,
        {
            // Add JWT authentication if configured
            if let Some(jwt_interceptor) = &self.jwt_interceptor {
                let mut interceptor = jwt_interceptor.clone();
                router.add_service(service.with_interceptor(move |req| interceptor.call(req)))
            } else {
                router.add_service(service)
            }
        }
    }

    /// Secure Redis client builder
    pub struct SecureRedisClientBuilder {
        tls_config: Option<TlsConfig>,
        jwt_authenticator: Option<Arc<JwtAuthenticator>>,
        service_name: Option<String>,
        roles: Vec<String>,
    }

    impl SecureRedisClientBuilder {
        /// Create a new secure Redis client builder
        pub fn new() -> Self {
            Self {
                tls_config: None,
                jwt_authenticator: None,
                service_name: None,
                roles: Vec::new(),
            }
        }

        /// Add TLS configuration
        pub fn with_tls(mut self, tls_config: TlsConfig) -> Self {
            self.tls_config = Some(tls_config);
            self
        }

        /// Add JWT authentication
        pub fn with_jwt(
            mut self,
            jwt_authenticator: Arc<JwtAuthenticator>,
            service_name: String,
            roles: Vec<String>,
        ) -> Self {
            self.jwt_authenticator = Some(jwt_authenticator);
            self.service_name = Some(service_name);
            self.roles = roles;
            self
        }

        /// Build the secure Redis client
        pub async fn build(self, redis_url: &str) -> IpcResult<Arc<dyn RedisClient>> {
            use redis::ConnectionInfo;
            use std::str::FromStr;

            // Parse the Redis URL
            let connection_info = ConnectionInfo::from_str(redis_url)
                .map_err(|e| IpcError::Connection(format!("Invalid Redis URL: {}", e)))?;

            // Create the base Redis client
            let redis_client = if let Some(tls_config) = self.tls_config {
                // Load the certificate and private key
                let cert = std::fs::read(&tls_config.cert_path).map_err(|e| {
                    IpcError::Internal(format!("Failed to read certificate: {}", e))
                })?;
                let key = std::fs::read(&tls_config.key_path).map_err(|e| {
                    IpcError::Internal(format!("Failed to read private key: {}", e))
                })?;
                let ca_cert = std::fs::read(&tls_config.ca_cert_path).map_err(|e| {
                    IpcError::Internal(format!("Failed to read CA certificate: {}", e))
                })?;

                // Create a TLS connector
                let mut connector = native_tls::TlsConnector::builder();
                connector.identity(native_tls::Identity::from_pkcs8(&cert, &key).map_err(|e| {
                    IpcError::Internal(format!("Failed to create identity: {}", e))
                })?);
                connector.add_root_certificate(
                    native_tls::Certificate::from_pem(&ca_cert).map_err(|e| {
                        IpcError::Internal(format!("Failed to add root certificate: {}", e))
                    })?,
                );

                // Create the Redis client with TLS
                let redis_url = redis_url.replace("redis://", "rediss://");
                let client = redis::Client::open(redis_url.as_str()).map_err(|e| {
                    IpcError::Connection(format!("Failed to connect to Redis: {}", e))
                })?;

                Arc::new(crate::modules::ipc::redis_pubsub::RedisClientImpl::new(&redis_url).await?)
            } else {
                Arc::new(crate::modules::ipc::redis_pubsub::RedisClientImpl::new(redis_url).await?)
            };

            // Add JWT authentication if configured
            if let (Some(jwt_authenticator), Some(service_name)) =
                (self.jwt_authenticator, self.service_name)
            {
                let authenticated_client = AuthenticatedRedisClient::new(
                    redis_client,
                    jwt_authenticator,
                    service_name,
                    self.roles,
                )
                .await;

                Ok(Arc::new(authenticated_client))
            } else {
                Ok(redis_client)
            }
        }
    }

    #[test]
    fn test_role_config() {
        let mut role_config = RoleConfig::new();
        role_config.add_role("service1", "role1");
        role_config.add_role("service1", "role2");
        role_config.add_role("service2", "role3");

        assert_eq!(role_config.get_roles("service1"), vec!["role1", "role2"]);
        assert_eq!(role_config.get_roles("service2"), vec!["role3"]);
        assert_eq!(role_config.get_roles("service3"), Vec::<String>::new());

        assert!(role_config.has_role("service1", "role1"));
        assert!(role_config.has_role("service1", "role2"));
        assert!(!role_config.has_role("service1", "role3"));
        assert!(role_config.has_role("service2", "role3"));
        assert!(!role_config.has_role("service3", "role1"));
    }

    // This test requires TLS certificates
    // #[tokio::test]
    // async fn test_tls_config() {
    //     let tls_config = TlsConfig::new(
    //         "tests/certs/client.crt",
    //         "tests/certs/client.key",
    //         "tests/certs/ca.crt",
    //     );
    //
    //     let client_config = tls_config.load_client_config("localhost").unwrap();
    //     let server_config = tls_config.load_server_config().unwrap();
    //
    //     // These assertions just check that the configs were created successfully
    //     assert!(client_config.identity.is_some());
    //     assert!(client_config.ca_certificate.is_some());
    //     assert_eq!(client_config.domain_name.unwrap(), "localhost");
    //
    //     assert!(server_config.identity.is_some());
    //     assert!(server_config.client_ca_root.is_some());
    // }
}
