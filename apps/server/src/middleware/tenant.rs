use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::{services::TenantService, AppState};

pub struct TenantMiddleware;

impl TenantMiddleware {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Clone, Debug)]
pub struct TenantContext {
    pub tenant_id: Uuid,
}

pub async fn tenant_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract tenant ID from X-Tenant-ID header
    let tenant_id = if let Some(tenant_header) = headers.get("X-Tenant-ID") {
        let tenant_id_str = tenant_header
            .to_str()
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        let tenant_uuid = Uuid::parse_str(tenant_id_str).map_err(|_| StatusCode::BAD_REQUEST)?;

        // Validate that the tenant exists and is active
        let tenant_service = TenantService::new(state.database.clone());
        match tenant_service.get_tenant_by_id(tenant_uuid).await {
            Ok(Some(tenant)) if tenant.is_active.unwrap_or(false) => tenant_uuid,
            Ok(Some(_)) => return Err(StatusCode::FORBIDDEN), // Tenant exists but inactive
            Ok(None) => return Err(StatusCode::NOT_FOUND),    // Tenant doesn't exist
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        // For certain routes (like auth), we might not require tenant header
        // Check if this is an auth route
        let path = req.uri().path();
        if path.starts_with("/api/v1/auth/") {
            // Allow auth routes without tenant header
            return Ok(next.run(req).await);
        }

        // Allow health check endpoint without tenant header
        if path == "/health" {
            return Ok(next.run(req).await);
        }

        // Allow frontend/static routes (anything not starting with /api/) without tenant header
        if !path.starts_with("/api/") {
            return Ok(next.run(req).await);
        }

        // For all other API routes, tenant header is required
        return Err(StatusCode::BAD_REQUEST);
    };

    // Add tenant context to request extensions for later use
    let tenant_context = TenantContext { tenant_id };
    req.extensions_mut().insert(tenant_context);

    Ok(next.run(req).await)
}
