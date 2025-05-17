use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Failed to acquire lock")]
    LockError,

    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("Missing API key")]
    MissingApiKey,

    #[error("Insufficient permissions")]
    InsufficientPermissions,

    #[error("Internal error: {0}")]
    InternalError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub key: String,
    pub name: String,
    pub roles: Vec<String>,
    pub created_at: DateTime<Utc>,
}

pub struct AuthManager {
    api_keys: RwLock<HashMap<String, ApiKey>>,
}

impl AuthManager {
    pub fn new() -> Self {
        Self {
            api_keys: RwLock::new(HashMap::new()),
        }
    }

    pub fn add_api_key(&self, api_key: ApiKey) -> Result<(), AuthError> {
        let mut keys = self.api_keys.write().map_err(|_| AuthError::LockError)?;
        keys.insert(api_key.key.clone(), api_key);
        Ok(())
    }

    pub fn remove_api_key(&self, key: &str) -> Result<bool, AuthError> {
        let mut keys = self.api_keys.write().map_err(|_| AuthError::LockError)?;
        Ok(keys.remove(key).is_some())
    }

    pub fn validate_api_key(&self, key: &str) -> Result<Option<ApiKey>, AuthError> {
        let keys = self.api_keys.read().map_err(|_| AuthError::LockError)?;
        Ok(keys.get(key).cloned())
    }

    pub fn has_role(&self, key: &str, role: &str) -> Result<bool, AuthError> {
        let api_key = match self.validate_api_key(key)? {
            Some(key) => key,
            None => return Ok(false),
        };

        Ok(api_key.roles.contains(&role.to_string()))
    }

    pub fn list_api_keys(&self) -> Result<Vec<ApiKey>, AuthError> {
        let keys = self.api_keys.read().map_err(|_| AuthError::LockError)?;
        Ok(keys.values().cloned().collect())
    }
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub api_key: ApiKey,
}

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for AuthContext
where
    S: Send + Sync,
    S: State<Arc<AuthManager>>,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract API key from Authorization header
        let auth_header = parts
            .headers
            .get("Authorization")
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let auth_value = auth_header.to_str().map_err(|_| StatusCode::UNAUTHORIZED)?;

        if !auth_value.starts_with("Bearer ") {
            return Err(StatusCode::UNAUTHORIZED);
        }

        let api_key = auth_value[7..].to_string();

        // Validate API key
        let auth_manager = state.0.clone();
        let key = auth_manager
            .validate_api_key(&api_key)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::UNAUTHORIZED)?;

        Ok(AuthContext { api_key: key })
    }
}
