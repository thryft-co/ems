use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Extension, Router,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::tenant::TenantContext,
    models::{
        AssetResponse, AssetSummary, AssetTypeResponse, Claims, CreateAssetIdResponse,
        CreateAssetRequest, CreateAssetTypeRequest, UpdateAssetRequest, UpdateAssetTypeRequest,
    },
    services::AssetService,
    AppState,
};

#[derive(Deserialize)]
struct ListAssetsQuery {
    asset_type_id: Option<Uuid>,
    item_id: Option<Uuid>,
    is_active: Option<bool>,
    limit: Option<u32>,
    offset: Option<u32>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        // Asset Type routes
        .route("/types", get(list_asset_types).post(create_asset_type))
        .route(
            "/types/:id",
            get(get_asset_type)
                .put(update_asset_type)
                .delete(delete_asset_type),
        )
        // Asset routes
        .route("/", get(list_assets).post(create_asset))
        .route(
            "/:id",
            get(get_asset).put(update_asset).delete(delete_asset),
        )
        // Utility routes
        .route("/by-item/:item_id", get(get_assets_by_item))
        .route("/by-type/:asset_type_id", get(get_assets_by_type))
}

// Helper function to extract tenant ID from request extensions
fn extract_tenant_id(tenant_context: &TenantContext) -> Uuid {
    tenant_context.tenant_id
}

// Helper function to extract user ID from JWT claims
fn extract_user_id(claims: &Claims) -> Result<Uuid, StatusCode> {
    Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::UNAUTHORIZED)
}

// Asset Type API implementations

async fn create_asset_type(
    State(state): State<AppState>,
    Json(payload): Json<CreateAssetTypeRequest>,
) -> Result<Json<AssetTypeResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let asset_service = AssetService::new(state.database);

    match asset_service.create_asset_type(payload).await {
        Ok(asset_type) => {
            let response = AssetTypeResponse {
                id: asset_type.id,
                name: asset_type.name,
                description: asset_type.description,
                created_at: asset_type.created_at.unwrap_or_else(|| chrono::Utc::now()),
                updated_at: asset_type.updated_at.unwrap_or_else(|| chrono::Utc::now()),
            };
            Ok(Json(response))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_asset_type(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AssetTypeResponse>, StatusCode> {
    let asset_service = AssetService::new(state.database);

    match asset_service.get_asset_type_by_id(id).await {
        Ok(Some(asset_type)) => {
            let response = AssetTypeResponse {
                id: asset_type.id,
                name: asset_type.name,
                description: asset_type.description,
                created_at: asset_type.created_at.unwrap_or_else(|| chrono::Utc::now()),
                updated_at: asset_type.updated_at.unwrap_or_else(|| chrono::Utc::now()),
            };
            Ok(Json(response))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn list_asset_types(
    State(state): State<AppState>,
) -> Result<Json<Vec<AssetTypeResponse>>, StatusCode> {
    let asset_service = AssetService::new(state.database);

    match asset_service.list_asset_types().await {
        Ok(asset_types) => Ok(Json(asset_types)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_asset_type(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateAssetTypeRequest>,
) -> Result<Json<AssetTypeResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let asset_service = AssetService::new(state.database);

    match asset_service.update_asset_type(id, payload).await {
        Ok(asset_type) => {
            let response = AssetTypeResponse {
                id: asset_type.id,
                name: asset_type.name,
                description: asset_type.description,
                created_at: asset_type.created_at.unwrap_or_else(|| chrono::Utc::now()),
                updated_at: asset_type.updated_at.unwrap_or_else(|| chrono::Utc::now()),
            };
            Ok(Json(response))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_asset_type(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let asset_service = AssetService::new(state.database);

    match asset_service.delete_asset_type(id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Asset API implementations

async fn create_asset(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateAssetRequest>,
) -> Result<Json<CreateAssetIdResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let created_by_id = extract_user_id(&claims)?;
    let asset_service = AssetService::new(state.database);

    match asset_service
        .create_asset(tenant_id, created_by_id, payload)
        .await
    {
        Ok(response) => Ok(Json(response)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_asset(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<AssetResponse>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let asset_service = AssetService::new(state.database);

    match asset_service.get_asset_by_id(tenant_id, id).await {
        Ok(Some(asset)) => Ok(Json(asset)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn list_assets(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(params): Query<ListAssetsQuery>,
) -> Result<Json<Vec<AssetSummary>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let asset_service = AssetService::new(state.database);

    match asset_service
        .list_assets(
            tenant_id,
            params.asset_type_id,
            params.item_id,
            params.is_active,
            params.limit,
            params.offset,
        )
        .await
    {
        Ok(assets) => Ok(Json(assets)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_asset(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateAssetRequest>,
) -> Result<Json<AssetResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let asset_service = AssetService::new(state.database);

    match asset_service.update_asset(tenant_id, id, payload).await {
        Ok(asset) => Ok(Json(asset)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_asset(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let asset_service = AssetService::new(state.database);

    match asset_service.delete_asset(tenant_id, id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Utility endpoints

async fn get_assets_by_item(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(item_id): Path<Uuid>,
) -> Result<Json<Vec<AssetSummary>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let asset_service = AssetService::new(state.database);

    match asset_service
        .get_assets_by_item_id(tenant_id, item_id)
        .await
    {
        Ok(assets) => Ok(Json(assets)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_assets_by_type(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(asset_type_id): Path<Uuid>,
) -> Result<Json<Vec<AssetSummary>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let asset_service = AssetService::new(state.database);

    match asset_service
        .get_assets_by_type(tenant_id, asset_type_id)
        .await
    {
        Ok(assets) => Ok(Json(assets)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
