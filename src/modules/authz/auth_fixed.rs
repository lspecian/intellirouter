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
    #[error("API key not found")]
    KeyNotFound,
    #[error("Invalid API key")]
    InvalidKey,
    #[error("Unauthorized")]
    Unauthorized,
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

#[derive(Debug)]
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

// Temporary implementation to avoid compilation errors
impl AuthContext {
    pub fn dummy() -> Self {
        let key = ApiKey {
            key: "dummy-key".to_string(),
            name: "Dummy Key".to_string(),
            roles: vec!["admin".to_string()],
            created_at: chrono::Utc::now(),
        };

        AuthContext { api_key: key }
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
