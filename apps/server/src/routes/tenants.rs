use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::{
    models::{CreateTenantRequest, Tenant, UpdateTenantRequest},
    services::TenantService,
    AppState,
};

#[derive(Deserialize)]
struct ListQuery {
    limit: Option<u32>,
    offset: Option<u32>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_tenants).post(create_tenant))
        .route("/accessible", get(list_accessible_tenants))
        .route(
            "/:id",
            get(get_tenant).put(update_tenant).delete(delete_tenant),
        )
}

async fn create_tenant(
    State(state): State<AppState>,
    Json(payload): Json<CreateTenantRequest>,
) -> Result<Json<Tenant>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_service = TenantService::new(state.database);

    match tenant_service.create_tenant(payload).await {
        Ok(tenant) => Ok(Json(tenant)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_tenant(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Tenant>, StatusCode> {
    let tenant_service = TenantService::new(state.database);

    match tenant_service.get_tenant_by_id(id).await {
        Ok(Some(tenant)) => Ok(Json(tenant)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_tenant(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTenantRequest>,
) -> Result<Json<Tenant>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_service = TenantService::new(state.database);

    match tenant_service.update_tenant(id, payload).await {
        Ok(tenant) => Ok(Json(tenant)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_tenant(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let tenant_service = TenantService::new(state.database);

    match tenant_service.delete_tenant(id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn list_tenants(
    State(state): State<AppState>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<Tenant>>, StatusCode> {
    let tenant_service = TenantService::new(state.database);

    match tenant_service
        .list_tenants(params.limit, params.offset)
        .await
    {
        Ok(tenants) => Ok(Json(tenants)),
        Err(e) => {
            tracing::error!("Failed to list tenants: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn list_accessible_tenants(
    State(state): State<AppState>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<Tenant>>, StatusCode> {
    let tenant_service = TenantService::new(state.database);

    // This endpoint returns all active tenants
    // In the future, this could be filtered based on user permissions
    match tenant_service
        .list_tenants(params.limit, params.offset)
        .await
    {
        Ok(tenants) => {
            // Filter to only active tenants
            let active_tenants: Vec<Tenant> = tenants
                .into_iter()
                .filter(|t| t.is_active.unwrap_or(false))
                .collect();
            Ok(Json(active_tenants))
        }
        Err(e) => {
            tracing::error!("Failed to list accessible tenants: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
