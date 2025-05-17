use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use super::auth::{ApiKey, AuthContext, AuthManager};
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

pub fn create_routes(auth_manager: Arc<AuthManager>, rbac_manager: Arc<RbacManager>) -> Router {
    Router::new()
        .route("/api-keys", get(list_api_keys))
        .route("/api-keys", post(create_api_key))
        .route("/api-keys/:key", delete(delete_api_key))
        .route("/roles", get(list_roles))
        .route("/roles", post(create_role))
        .route("/roles/:name", delete(delete_role))
        .route("/roles/:name/permissions", post(add_permission))
        .route(
            "/roles/:name/permissions/:permission",
            delete(remove_permission),
        )
        .with_state((auth_manager, rbac_manager))
}

async fn list_api_keys(
    State((auth_manager, _)): State<(Arc<AuthManager>, Arc<RbacManager>)>,
    auth_context: AuthContext,
) -> Result<Json<Vec<ApiKeyResponse>>, StatusCode> {
    // Only admins can list API keys
    if !auth_context.api_key.roles.contains(&"admin".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    let api_keys = auth_manager
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

async fn create_api_key(
    State((auth_manager, _)): State<(Arc<AuthManager>, Arc<RbacManager>)>,
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

    auth_manager
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

async fn delete_api_key(
    State((auth_manager, _)): State<(Arc<AuthManager>, Arc<RbacManager>)>,
    auth_context: AuthContext,
    Path(key): Path<String>,
) -> Result<StatusCode, StatusCode> {
    // Only admins can delete API keys
    if !auth_context.api_key.roles.contains(&"admin".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    let removed = auth_manager
        .remove_api_key(&key)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if removed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn list_roles(
    State((_, rbac_manager)): State<(Arc<AuthManager>, Arc<RbacManager>)>,
    auth_context: AuthContext,
) -> Result<Json<Vec<super::rbac::Role>>, StatusCode> {
    // Only admins can list roles
    if !auth_context.api_key.roles.contains(&"admin".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    let roles = rbac_manager
        .list_roles()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(roles))
}

async fn create_role(
    State((_, rbac_manager)): State<(Arc<AuthManager>, Arc<RbacManager>)>,
    auth_context: AuthContext,
    Json(request): Json<AddRoleRequest>,
) -> Result<StatusCode, StatusCode> {
    // Only admins can create roles
    if !auth_context.api_key.roles.contains(&"admin".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    rbac_manager
        .add_role(&request.name)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(StatusCode::CREATED)
}

async fn delete_role(
    State((_, rbac_manager)): State<(Arc<AuthManager>, Arc<RbacManager>)>,
    auth_context: AuthContext,
    Path(name): Path<String>,
) -> Result<StatusCode, StatusCode> {
    // Only admins can delete roles
    if !auth_context.api_key.roles.contains(&"admin".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    rbac_manager
        .remove_role(&name)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(StatusCode::NO_CONTENT)
}

async fn add_permission(
    State((_, rbac_manager)): State<(Arc<AuthManager>, Arc<RbacManager>)>,
    auth_context: AuthContext,
    Path(name): Path<String>,
    Json(request): Json<AddPermissionRequest>,
) -> Result<StatusCode, StatusCode> {
    // Only admins can add permissions
    if !auth_context.api_key.roles.contains(&"admin".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    rbac_manager
        .add_permission_to_role(&name, &request.permission)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(StatusCode::CREATED)
}

async fn remove_permission(
    State((_, rbac_manager)): State<(Arc<AuthManager>, Arc<RbacManager>)>,
    auth_context: AuthContext,
    Path((name, permission)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    // Only admins can remove permissions
    if !auth_context.api_key.roles.contains(&"admin".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    rbac_manager
        .remove_permission_from_role(&name, &permission)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(StatusCode::NO_CONTENT)
}
