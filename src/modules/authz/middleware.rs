use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use std::sync::Arc;

use super::auth::{AuthContext, AuthManager};
use super::rbac::{check_permission, RbacManager};

pub async fn auth_middleware<B>(
    State(auth_manager): State<Arc<AuthManager>>,
    State(rbac_manager): State<Arc<RbacManager>>,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // Extract API key from Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let auth_value = auth_header.to_str().map_err(|_| StatusCode::UNAUTHORIZED)?;

    if !auth_value.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let api_key = &auth_value[7..];

    // Validate API key
    let key = auth_manager
        .validate_api_key(api_key)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Create auth context
    let auth_context = AuthContext { api_key: key };

    // Extract required permission from request path
    let path = request.uri().path();
    let permission = match path {
        "/v1/chat/completions" => "execute:chat",
        "/v1/models" => "read:models",
        "/v1/chains" => "execute:chains",
        _ => "access:api",
    };

    // Check permission
    let has_permission = check_permission(&auth_context, &rbac_manager, permission)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !has_permission {
        return Err(StatusCode::FORBIDDEN);
    }

    // Add auth context to request extensions
    request.extensions_mut().insert(auth_context);

    // Continue with the request
    Ok(next.run(request).await)
}

pub async fn require_permission<B>(
    permission: &'static str,
    State(rbac_manager): State<Arc<RbacManager>>,
    auth_context: AuthContext,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // Check permission
    let has_permission = check_permission(&auth_context, &rbac_manager, permission)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !has_permission {
        return Err(StatusCode::FORBIDDEN);
    }

    // Continue with the request
    Ok(next.run(request).await)
}
