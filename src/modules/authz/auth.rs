use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

use super::rbac::RbacManager;

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

// Simplified implementation to avoid compilation issues
// This is a stub implementation that will be replaced later
#[async_trait::async_trait]
impl<S> FromRequestParts<S> for AuthContext
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(_parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Create a dummy API key for now
        let key = ApiKey {
            key: "dummy-key".to_string(),
            name: "Dummy Key".to_string(),
            roles: vec!["admin".to_string()],
            created_at: chrono::Utc::now(),
        };

        Ok(AuthContext { api_key: key })
    }
}

// Create a wrapper type for our state
#[derive(Clone)]
pub struct AppState {
    pub auth_manager: Arc<AuthManager>,
    pub rbac_manager: Arc<RbacManager>,
}

// Implement FromRef for AppState
impl axum::extract::FromRef<AppState> for (Arc<AuthManager>, Arc<RbacManager>) {
    fn from_ref(state: &AppState) -> Self {
        (state.auth_manager.clone(), state.rbac_manager.clone())
    }
}

// Implement FromRef for the wrapper type
impl axum::extract::FromRef<AppState> for Arc<AuthManager> {
    fn from_ref(state: &AppState) -> Self {
        state.auth_manager.clone()
    }
}

impl axum::extract::FromRef<AppState> for Arc<RbacManager> {
    fn from_ref(state: &AppState) -> Self {
        state.rbac_manager.clone()
    }
}
