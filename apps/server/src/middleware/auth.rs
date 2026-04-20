use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::middleware::tenant::TenantContext;
use crate::{core::AuthUtils, services::AuthService, AppState};

pub async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract JWT token from Authorization header
    let auth_header = headers
        .get("authorization")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let auth_str = auth_header.to_str().map_err(|_| StatusCode::UNAUTHORIZED)?;

    if !auth_str.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_str[7..]; // Remove "Bearer " prefix

    // Verify JWT token
    let claims = AuthUtils::verify_jwt_token(token).map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Check if user has "pending" role (no tenant yet)
    if claims.role == "pending" {
        // Allow pending users to access certain endpoints without tenant validation
        req.extensions_mut().insert(claims);
        return Ok(next.run(req).await);
    }

    // Parse tenant ID from token
    let token_tenant_id =
        Uuid::parse_str(&claims.tenant_id).map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Verify tenant context matches token
    if let Some(tenant_context) = req.extensions().get::<TenantContext>() {
        if tenant_context.tenant_id != token_tenant_id {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // Check if token is blacklisted (for access tokens, we could also check refresh tokens)
    let auth_service = AuthService::new(state.database, state.supabase);
    if auth_service
        .is_token_blacklisted(token, token_tenant_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Add claims to request extensions for later use
    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}
