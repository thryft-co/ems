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
        BomItemResponse, CreateBomItemRequest, CreateItemIdResponse, CreateItemRequest,
        FinishedGoodsItemResponse, ItemContext, ItemLifecycle, ItemResponse, ItemStatus,
        StoreItemResponse, UpdateBomItemRequest, UpdateItemRequest, VendorItemResponse,
    },
    services::ItemService,
    AppState,
};

#[derive(Deserialize)]
struct ListQuery {
    context: Option<ItemContext>,
    category: Option<String>,
    lifecycle: Option<ItemLifecycle>,
    #[allow(dead_code)]
    status: Option<ItemStatus>,
    limit: Option<u32>,
    offset: Option<u32>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        // General Item API
        .route("/", get(list_all_items).post(create_item))
        .route(
            "/:id",
            get(get_item_details).put(update_item).delete(delete_item),
        )
        // Context-specific Item API routes
        .route("/finished-goods", get(list_finished_goods_items))
        .route(
            "/finished-goods/:id",
            get(get_finished_goods_item_details).put(update_finished_goods_item),
        )
        .route("/store", get(list_store_items))
        .route(
            "/store/:id",
            get(get_store_item_details).put(update_store_item),
        )
        .route("/vendor", get(list_vendor_items))
        .route(
            "/vendor/:id",
            get(get_vendor_item_details).put(update_vendor_item),
        )
        // BOM API routes
        .route("/bom", get(list_bom_items).post(create_bom_item))
        .route(
            "/bom/:id",
            get(get_bom_item_details)
                .put(update_bom_item)
                .delete(delete_bom_item),
        )
        .route("/:id/bom", get(get_item_bom))
}

// Helper function to extract tenant ID from request extensions
fn extract_tenant_id(tenant_context: &TenantContext) -> Uuid {
    tenant_context.tenant_id
}

// General Item API implementations

async fn list_all_items(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<ItemResponse>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let item_service = ItemService::new(state.database);

    match item_service
        .list_items(
            tenant_id,
            params.context,
            params.category,
            params.lifecycle,
            params.limit,
            params.offset,
        )
        .await
    {
        Ok(items) => Ok(Json(items)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn create_item(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Json(payload): Json<CreateItemRequest>,
) -> Result<Json<CreateItemIdResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let item_service = ItemService::new(state.database);

    match item_service.create_item(tenant_id, payload).await {
        Ok(response) => Ok(Json(response)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_item_details(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Query(params): Query<ListQuery>,
) -> Result<Json<ItemResponse>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let item_service = ItemService::new(state.database);
    let context = params.context.unwrap_or(ItemContext::Store);

    match item_service.get_item_by_id(tenant_id, id, context).await {
        Ok(Some(item)) => Ok(Json(item)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_item(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Query(params): Query<ListQuery>,
    Json(payload): Json<UpdateItemRequest>,
) -> Result<Json<ItemResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let item_service = ItemService::new(state.database);
    let context = params.context.unwrap_or(ItemContext::Store);

    match item_service
        .update_item(tenant_id, id, context, payload)
        .await
    {
        Ok(item) => Ok(Json(item)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_item(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Query(params): Query<ListQuery>,
) -> Result<StatusCode, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let item_service = ItemService::new(state.database);
    let context = params.context.unwrap_or(ItemContext::Store);

    match item_service.delete_item(tenant_id, id, context).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Context-specific implementations

async fn list_finished_goods_items(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<FinishedGoodsItemResponse>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let item_service = ItemService::new(state.database);

    match item_service
        .list_finished_goods_items(tenant_id, params.limit, params.offset)
        .await
    {
        Ok(items) => Ok(Json(items)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_finished_goods_item_details(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<FinishedGoodsItemResponse>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let item_service = ItemService::new(state.database);

    match item_service
        .get_finished_goods_item_by_id(tenant_id, id)
        .await
    {
        Ok(Some(item)) => Ok(Json(item)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_finished_goods_item(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateItemRequest>,
) -> Result<Json<FinishedGoodsItemResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let item_service = ItemService::new(state.database);

    match item_service
        .update_item(tenant_id, id, ItemContext::FinishedGoods, payload)
        .await
    {
        Ok(_) => {
            // Return the updated finished goods item
            match item_service
                .get_finished_goods_item_by_id(tenant_id, id)
                .await
            {
                Ok(Some(item)) => Ok(Json(item)),
                Ok(None) => Err(StatusCode::NOT_FOUND),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn list_store_items(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<StoreItemResponse>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let item_service = ItemService::new(state.database);

    match item_service
        .list_store_items(tenant_id, params.limit, params.offset)
        .await
    {
        Ok(items) => Ok(Json(items)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_store_item_details(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<StoreItemResponse>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let item_service = ItemService::new(state.database);

    match item_service.get_store_item_by_id(tenant_id, id).await {
        Ok(Some(item)) => Ok(Json(item)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_store_item(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateItemRequest>,
) -> Result<Json<StoreItemResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let item_service = ItemService::new(state.database);

    match item_service
        .update_item(tenant_id, id, ItemContext::Store, payload)
        .await
    {
        Ok(_) => {
            // Return the updated store item
            match item_service.get_store_item_by_id(tenant_id, id).await {
                Ok(Some(item)) => Ok(Json(item)),
                Ok(None) => Err(StatusCode::NOT_FOUND),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn list_vendor_items(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<VendorItemResponse>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let item_service = ItemService::new(state.database);

    match item_service
        .list_vendor_items(tenant_id, params.limit, params.offset)
        .await
    {
        Ok(items) => Ok(Json(items)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_vendor_item_details(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<VendorItemResponse>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let item_service = ItemService::new(state.database);

    match item_service.get_vendor_item_by_id(tenant_id, id).await {
        Ok(Some(item)) => Ok(Json(item)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_vendor_item(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateItemRequest>,
) -> Result<Json<VendorItemResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let item_service = ItemService::new(state.database);

    match item_service
        .update_item(tenant_id, id, ItemContext::Vendor, payload)
        .await
    {
        Ok(_) => {
            // Return the updated vendor item
            match item_service.get_vendor_item_by_id(tenant_id, id).await {
                Ok(Some(item)) => Ok(Json(item)),
                Ok(None) => Err(StatusCode::NOT_FOUND),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// BOM (Bill of Materials) API implementations

async fn list_bom_items(
    State(_state): State<AppState>,
    Extension(_tenant_context): Extension<TenantContext>,
    Query(_params): Query<ListQuery>,
) -> Result<Json<Vec<BomItemResponse>>, StatusCode> {
    // This would typically list all BOM items, but it's not commonly used
    // Instead, we usually get BOM by parent item
    Err(StatusCode::NOT_IMPLEMENTED)
}

async fn create_bom_item(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Json(payload): Json<CreateBomItemRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let item_service = ItemService::new(state.database);

    match item_service.create_bom_item(tenant_id, payload).await {
        Ok(bom_id) => Ok(Json(serde_json::json!({"id": bom_id}))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_bom_item_details(
    State(_state): State<AppState>,
    Extension(_tenant_context): Extension<TenantContext>,
    Path(_id): Path<Uuid>,
) -> Result<Json<BomItemResponse>, StatusCode> {
    // Getting a single BOM item by ID is not commonly implemented
    // Usually we get BOM by parent item
    Err(StatusCode::NOT_IMPLEMENTED)
}

async fn update_bom_item(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateBomItemRequest>,
) -> Result<StatusCode, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let item_service = ItemService::new(state.database);

    match item_service.update_bom_item(tenant_id, id, payload).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_bom_item(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let item_service = ItemService::new(state.database);

    match item_service.delete_bom_item(tenant_id, id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_item_bom(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(parent_item_id): Path<Uuid>,
) -> Result<Json<Vec<BomItemResponse>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let item_service = ItemService::new(state.database);

    match item_service
        .get_bom_by_parent_item(tenant_id, parent_item_id)
        .await
    {
        Ok(bom_items) => Ok(Json(bom_items)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
