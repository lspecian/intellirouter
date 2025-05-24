use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use super::auth::{ApiKey, AppState, AuthContext, AuthManager};
use super::rbac::RbacManager;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateApiKeyResponse {
    pub key: String,
    pub name: String,
    pub roles: Vec<String>,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKeyResponse {
    pub name: String,
    pub roles: Vec<String>,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddRoleRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddPermissionRequest {
    pub permission: String,
}

pub fn create_routes(_auth_manager: Arc<AuthManager>, _rbac_manager: Arc<RbacManager>) -> Router {
    // TODO: Re-enable authorization routes once dependencies are resolved.
    // Issue: [Link to GitHub issue to be created for tracking this task]
    Router::new()
}

async fn _list_api_keys(
    State(state): State<AppState>,
    auth_context: AuthContext,
) -> Result<Json<Vec<ApiKeyResponse>>, StatusCode> {
    // Only admins can list API keys
    if !auth_context.api_key.roles.contains(&"admin".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    let api_keys = state
        .auth_manager
        .list_api_keys()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let responses = api_keys
        .into_iter()
        .map(|key| ApiKeyResponse {
            name: key.name,
            roles: key.roles,
            created_at: key.created_at,
        })
        .collect();

    Ok(Json(responses))
}

async fn _create_api_key(
    State(state): State<AppState>,
    auth_context: AuthContext,
    Json(request): Json<CreateApiKeyRequest>,
) -> Result<Json<CreateApiKeyResponse>, StatusCode> {
    // Only admins can create API keys
    if !auth_context.api_key.roles.contains(&"admin".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Generate a new API key
    let key = format!("ir-{}", Uuid::new_v4());

    let api_key = ApiKey {
        key: key.clone(),
        name: request.name,
        roles: request.roles,
        created_at: Utc::now(),
    };

    state
        .auth_manager
        .add_api_key(api_key.clone())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = CreateApiKeyResponse {
        key,
        name: api_key.name,
        roles: api_key.roles,
        created_at: api_key.created_at,
    };

    Ok(Json(response))
}

async fn _delete_api_key(
    State(state): State<AppState>,
    auth_context: AuthContext,
    Path(key): Path<String>,
) -> Result<StatusCode, StatusCode> {
    // Only admins can delete API keys
    if !auth_context.api_key.roles.contains(&"admin".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    let removed = state
        .auth_manager
        .remove_api_key(&key)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if removed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn _list_roles(
    State(state): State<AppState>,
    auth_context: AuthContext,
) -> Result<Json<Vec<super::rbac::Role>>, StatusCode> {
    // Only admins can list roles
    if !auth_context.api_key.roles.contains(&"admin".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    let roles = state
        .rbac_manager
        .list_roles()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(roles))
}

async fn _create_role(
    State(state): State<AppState>,
    auth_context: AuthContext,
    Json(request): Json<AddRoleRequest>,
) -> Result<StatusCode, StatusCode> {
    // Only admins can create roles
    if !auth_context.api_key.roles.contains(&"admin".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    state
        .rbac_manager
        .add_role(&request.name)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(StatusCode::CREATED)
}

async fn _delete_role(
    State(state): State<AppState>,
    auth_context: AuthContext,
    Path(name): Path<String>,
) -> Result<StatusCode, StatusCode> {
    // Only admins can delete roles
    if !auth_context.api_key.roles.contains(&"admin".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    state
        .rbac_manager
        .remove_role(&name)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(StatusCode::NO_CONTENT)
}

async fn _add_permission(
    State(state): State<AppState>,
    auth_context: AuthContext,
    Path(name): Path<String>,
    Json(request): Json<AddPermissionRequest>,
) -> Result<StatusCode, StatusCode> {
    // Only admins can add permissions
    if !auth_context.api_key.roles.contains(&"admin".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    state
        .rbac_manager
        .add_permission_to_role(&name, &request.permission)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(StatusCode::CREATED)
}

async fn _remove_permission(
    State(state): State<AppState>,
    auth_context: AuthContext,
    Path((name, permission)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    // Only admins can remove permissions
    if !auth_context.api_key.roles.contains(&"admin".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    state
        .rbac_manager
        .remove_permission_from_role(&name, &permission)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(StatusCode::NO_CONTENT)
}
