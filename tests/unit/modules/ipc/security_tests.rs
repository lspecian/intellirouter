use std::sync::Arc;

use jsonwebtoken::{decode, DecodingKey, Validation};
use tonic::{Request, Status};

use intellirouter::modules::ipc::security::{
    JwtAuthenticator, JwtClaims, JwtConfig, JwtInterceptor, RoleConfig,
};

#[test]
fn test_jwt_config() {
    let config = JwtConfig {
        secret: "test-secret".to_string(),
        issuer: "test-issuer".to_string(),
        audience: "test-audience".to_string(),
        expiration_seconds: 3600,
    };

    assert_eq!(config.secret, "test-secret");
    assert_eq!(config.issuer, "test-issuer");
    assert_eq!(config.audience, "test-audience");
    assert_eq!(config.expiration_seconds, 3600);
}

#[test]
fn test_jwt_authenticator_generate_token() {
    let config = JwtConfig {
        secret: "test-secret".to_string(),
        issuer: "test-issuer".to_string(),
        audience: "test-audience".to_string(),
        expiration_seconds: 3600,
    };

    let authenticator = JwtAuthenticator::new(config);
    let service_name = "test-service";
    let roles = vec!["role1".to_string(), "role2".to_string()];

    let token = authenticator
        .generate_token(service_name, roles.clone())
        .unwrap();

    // Verify the token
    let decoded = decode::<JwtClaims>(
        &token,
        &DecodingKey::from_secret("test-secret".as_bytes()),
        &Validation::default(),
    )
    .unwrap();

    assert_eq!(decoded.claims.sub, service_name);
    assert_eq!(decoded.claims.iss, "test-issuer");
    assert_eq!(decoded.claims.aud, "test-audience");
    assert_eq!(decoded.claims.roles, roles);
}

#[test]
fn test_jwt_authenticator_validate_token() {
    let config = JwtConfig {
        secret: "test-secret".to_string(),
        issuer: "test-issuer".to_string(),
        audience: "test-audience".to_string(),
        expiration_seconds: 3600,
    };

    let authenticator = JwtAuthenticator::new(config);
    let service_name = "test-service";
    let roles = vec!["role1".to_string(), "role2".to_string()];

    let token = authenticator
        .generate_token(service_name, roles.clone())
        .unwrap();

    let claims = authenticator.validate_token(&token).unwrap();

    assert_eq!(claims.sub, service_name);
    assert_eq!(claims.iss, "test-issuer");
    assert_eq!(claims.aud, "test-audience");
    assert_eq!(claims.roles, roles);
}

#[test]
fn test_jwt_authenticator_validate_invalid_token() {
    let config = JwtConfig {
        secret: "test-secret".to_string(),
        issuer: "test-issuer".to_string(),
        audience: "test-audience".to_string(),
        expiration_seconds: 3600,
    };

    let authenticator = JwtAuthenticator::new(config);
    let invalid_token = "invalid-token";

    let result = authenticator.validate_token(invalid_token);
    assert!(result.is_err());
}

#[test]
fn test_jwt_interceptor() {
    let config = JwtConfig {
        secret: "test-secret".to_string(),
        issuer: "test-issuer".to_string(),
        audience: "test-audience".to_string(),
        expiration_seconds: 3600,
    };

    let authenticator = Arc::new(JwtAuthenticator::new(config));
    let service_name = "test-service";
    let roles = vec!["role1".to_string(), "role2".to_string()];

    let token = authenticator
        .generate_token(service_name, roles.clone())
        .unwrap();

    let mut interceptor = JwtInterceptor::new(authenticator, vec!["role1".to_string()]);

    // Create a request with a valid token
    let mut request = Request::new(());
    request.metadata_mut().insert(
        "authorization",
        format!("Bearer {}", token).parse().unwrap(),
    );

    // The interceptor should allow the request
    let result = interceptor.call(request);
    assert!(result.is_ok());
}

#[test]
fn test_jwt_interceptor_missing_token() {
    let config = JwtConfig {
        secret: "test-secret".to_string(),
        issuer: "test-issuer".to_string(),
        audience: "test-audience".to_string(),
        expiration_seconds: 3600,
    };

    let authenticator = Arc::new(JwtAuthenticator::new(config));
    let mut interceptor = JwtInterceptor::new(authenticator, vec!["role1".to_string()]);

    // Create a request without a token
    let request = Request::new(());

    // The interceptor should reject the request
    let result = interceptor.call(request);
    assert!(result.is_err());

    match result {
        Err(status) => {
            assert_eq!(status.code(), tonic::Code::Unauthenticated);
            assert_eq!(status.message(), "Missing authorization token");
        }
        _ => panic!("Expected an error"),
    }
}

#[test]
fn test_jwt_interceptor_invalid_token() {
    let config = JwtConfig {
        secret: "test-secret".to_string(),
        issuer: "test-issuer".to_string(),
        audience: "test-audience".to_string(),
        expiration_seconds: 3600,
    };

    let authenticator = Arc::new(JwtAuthenticator::new(config));
    let mut interceptor = JwtInterceptor::new(authenticator, vec!["role1".to_string()]);

    // Create a request with an invalid token
    let mut request = Request::new(());
    request
        .metadata_mut()
        .insert("authorization", "Bearer invalid-token".parse().unwrap());

    // The interceptor should reject the request
    let result = interceptor.call(request);
    assert!(result.is_err());

    match result {
        Err(status) => {
            assert_eq!(status.code(), tonic::Code::Unauthenticated);
        }
        _ => panic!("Expected an error"),
    }
}

#[test]
fn test_jwt_interceptor_insufficient_permissions() {
    let config = JwtConfig {
        secret: "test-secret".to_string(),
        issuer: "test-issuer".to_string(),
        audience: "test-audience".to_string(),
        expiration_seconds: 3600,
    };

    let authenticator = Arc::new(JwtAuthenticator::new(config));
    let service_name = "test-service";
    let roles = vec!["role1".to_string()];

    let token = authenticator
        .generate_token(service_name, roles.clone())
        .unwrap();

    let mut interceptor = JwtInterceptor::new(authenticator, vec!["role2".to_string()]);

    // Create a request with a valid token but insufficient permissions
    let mut request = Request::new(());
    request.metadata_mut().insert(
        "authorization",
        format!("Bearer {}", token).parse().unwrap(),
    );

    // The interceptor should reject the request
    let result = interceptor.call(request);
    assert!(result.is_err());

    match result {
        Err(status) => {
            assert_eq!(status.code(), tonic::Code::PermissionDenied);
            assert_eq!(status.message(), "Insufficient permissions");
        }
        _ => panic!("Expected an error"),
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

// The following tests require TLS certificates and a running Redis instance
// They are commented out to avoid test failures in CI environments

// #[tokio::test]
// async fn test_tls_config() {
//     use intellirouter::modules::ipc::security::TlsConfig;
//     use std::path::PathBuf;
//
//     let tls_config = TlsConfig::new(
//         PathBuf::from("tests/certs/client.crt"),
//         PathBuf::from("tests/certs/client.key"),
//         PathBuf::from("tests/certs/ca.crt"),
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
//
// #[tokio::test]
// async fn test_authenticated_redis_client() {
//     use intellirouter::modules::ipc::redis_pubsub::{Message, RedisClient, RedisClientImpl};
//     use intellirouter::modules::ipc::security::{AuthenticatedRedisClient, JwtAuthenticator, JwtConfig};
//     use std::sync::Arc;
//     use tokio::time::{sleep, Duration};
//
//     let redis_url = "redis://localhost:6379";
//     let inner_client = Arc::new(RedisClientImpl::new(redis_url).await.unwrap());
//
//     let jwt_config = JwtConfig {
//         secret: "test-secret".to_string(),
//         issuer: "test-issuer".to_string(),
//         audience: "test-audience".to_string(),
//         expiration_seconds: 3600,
//     };
//
//     let jwt_authenticator = Arc::new(JwtAuthenticator::new(jwt_config));
//     let service_name = "test-service";
//     let roles = vec!["role1".to_string(), "role2".to_string()];
//
//     let authenticated_client = AuthenticatedRedisClient::new(
//         inner_client,
//         jwt_authenticator.clone(),
//         service_name.to_string(),
//         roles.clone(),
//     )
//     .await;
//
//     let channel = "test-channel";
//     let message = b"test-message";
//
//     // Subscribe to the channel
//     let subscription = authenticated_client.subscribe(channel).await.unwrap();
//
//     // Publish a message
//     authenticated_client.publish(channel, message).await.unwrap();
//
//     // Wait for the message
//     sleep(Duration::from_millis(100)).await;
//
//     // Get the message
//     let received = subscription.next_message().await.unwrap().unwrap();
//     assert_eq!(received.channel, channel);
//     assert_eq!(received.payload, message);
// }
