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
        CreateOrderIdResponse, CreateOrderRequest, CustomerOrderResponse, DistributorOrderResponse,
        OrderResponse, OrderStatus, OrderType, PurchaseOrderResponse, UpdateOrderRequest,
    },
    services::OrderService,
    AppState,
};

#[derive(Deserialize)]
struct ListQuery {
    #[serde(rename = "type")]
    order_type: Option<OrderType>,
    status: Option<OrderStatus>,
    limit: Option<u32>,
    offset: Option<u32>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        // General Order API
        .route("/", get(list_all_orders).post(create_order))
        .route(
            "/:id",
            get(get_order_details)
                .put(update_order)
                .delete(delete_order),
        )
        .route("/:id/history", get(get_order_history))
        // Type-specific Order API routes
        .route("/purchase", get(list_purchase_orders))
        .route("/purchase/:id", get(get_purchase_order_details))
        .route("/customer", get(list_customer_orders))
        .route("/customer/:id", get(get_customer_order_details))
        .route("/distributor", get(list_distributor_orders))
        .route("/distributor/:id", get(get_distributor_order_details))
}

// Helper function to extract tenant ID from request extensions
fn extract_tenant_id(tenant_context: &TenantContext) -> Uuid {
    tenant_context.tenant_id
}

// General Order API implementations

async fn list_all_orders(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<OrderResponse>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let order_service = OrderService::new(state.database);

    match order_service
        .list_orders(
            tenant_id,
            params.order_type,
            params.status,
            params.limit,
            params.offset,
        )
        .await
    {
        Ok(orders) => Ok(Json(orders)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn create_order(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Json(payload): Json<CreateOrderRequest>,
) -> Result<Json<CreateOrderIdResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let order_service = OrderService::new(state.database);

    match order_service.create_order(tenant_id, payload).await {
        Ok(response) => Ok(Json(response)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_order_details(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<OrderResponse>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let order_service = OrderService::new(state.database);

    match order_service.get_order_by_id(tenant_id, id).await {
        Ok(Some(order)) => Ok(Json(order)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_order(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateOrderRequest>,
) -> Result<Json<OrderResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let order_service = OrderService::new(state.database);

    match order_service.update_order(tenant_id, id, payload).await {
        Ok(order) => Ok(Json(order)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_order(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let order_service = OrderService::new(state.database);

    match order_service.delete_order(tenant_id, id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_order_history(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<crate::models::OrderHistory>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let order_service = OrderService::new(state.database);

    match order_service.get_order_history(tenant_id, id).await {
        Ok(history) => Ok(Json(history)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Type-specific implementations

async fn list_purchase_orders(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<PurchaseOrderResponse>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let order_service = OrderService::new(state.database);

    match order_service
        .list_purchase_orders(tenant_id, params.limit, params.offset)
        .await
    {
        Ok(orders) => Ok(Json(orders)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_purchase_order_details(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<PurchaseOrderResponse>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let order_service = OrderService::new(state.database);

    match order_service.get_purchase_order_by_id(tenant_id, id).await {
        Ok(Some(order)) => Ok(Json(order)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn list_customer_orders(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<CustomerOrderResponse>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let order_service = OrderService::new(state.database);

    match order_service
        .list_customer_orders(tenant_id, params.limit, params.offset)
        .await
    {
        Ok(orders) => Ok(Json(orders)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_customer_order_details(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<CustomerOrderResponse>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let order_service = OrderService::new(state.database);

    match order_service.get_customer_order_by_id(tenant_id, id).await {
        Ok(Some(order)) => Ok(Json(order)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn list_distributor_orders(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<DistributorOrderResponse>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let order_service = OrderService::new(state.database);

    match order_service
        .list_distributor_orders(tenant_id, params.limit, params.offset)
        .await
    {
        Ok(orders) => Ok(Json(orders)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_distributor_order_details(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<DistributorOrderResponse>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let order_service = OrderService::new(state.database);

    match order_service
        .get_distributor_order_by_id(tenant_id, id)
        .await
    {
        Ok(Some(order)) => Ok(Json(order)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
