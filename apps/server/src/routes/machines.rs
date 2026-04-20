use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Extension, Router,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::tenant::TenantContext,
    models::{
        CreateMachineAssetRelationshipRequest, CreateMachineItemRelationshipRequest,
        CreateMachineJobAssignmentRequest, CreateMachineOperatorAssignmentRequest,
        CreateMachineRequest, HeartbeatRequest, ItemRelationshipType, JobAssignmentStatus,
        MachineAssetRelationshipResponse, MachineCreateIdResponse, MachineItemRelationshipResponse,
        MachineJobAssignmentResponse, MachineOperatorAssignmentResponse, MachineProtocol,
        MachineResponse, MachineStatus, UpdateMachineJobAssignmentRequest, UpdateMachineRequest,
    },
    services::MachineService,
    AppState,
};

#[derive(Deserialize)]
struct ListMachinesQuery {
    status: Option<MachineStatus>,
    protocol: Option<MachineProtocol>,
    limit: Option<u32>,
    offset: Option<u32>,
}

#[derive(Deserialize)]
struct ListByItemQuery {
    relationship_type: Option<ItemRelationshipType>,
}

#[derive(Deserialize)]
struct ListByJobQuery {
    status: Option<JobAssignmentStatus>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        // Main machine routes
        .route("/", get(list_machines).post(create_machine))
        .route(
            "/:id",
            get(get_machine_details)
                .put(update_machine)
                .delete(delete_machine),
        )
        .route("/:id/heartbeat", post(update_heartbeat))
        // Machine-Item relationship routes
        .route("/:id/items", get(list_machine_item_relationships))
        .route("/:id/items", post(create_machine_item_relationship))
        .route(
            "/items/:relationship_id",
            delete(delete_machine_item_relationship),
        )
        // Machine-Asset relationship routes
        .route("/:id/assets", get(list_machine_asset_relationships))
        .route("/:id/assets", post(create_machine_asset_relationship))
        .route(
            "/assets/:relationship_id",
            delete(delete_machine_asset_relationship),
        )
        // Machine-Operator assignment routes
        .route("/:id/operators", get(list_machine_operator_assignments))
        .route("/:id/operators", post(create_machine_operator_assignment))
        .route(
            "/operators/:assignment_id",
            delete(delete_machine_operator_assignment),
        )
        // Machine-Job assignment routes
        .route("/:id/jobs", get(list_machine_job_assignments))
        .route("/:id/jobs", post(create_machine_job_assignment))
        .route(
            "/job-assignments/:assignment_id",
            put(update_machine_job_assignment).delete(delete_machine_job_assignment),
        )
        // Utility routes
        .route("/by-item/:item_id", get(get_machines_by_item))
        .route("/by-job/:job_id", get(get_machines_by_job))
}

// Helper function to extract tenant ID from request extensions
fn extract_tenant_id(tenant_context: &TenantContext) -> Uuid {
    tenant_context.tenant_id
}

// Main machine API implementations

async fn list_machines(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(params): Query<ListMachinesQuery>,
) -> Result<Json<Vec<MachineResponse>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let machine_service = MachineService::new(state.database);

    match machine_service
        .list_machines(
            tenant_id,
            params.status,
            params.protocol,
            params.limit,
            params.offset,
        )
        .await
    {
        Ok(machines) => Ok(Json(machines)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn create_machine(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Json(payload): Json<CreateMachineRequest>,
) -> Result<Json<MachineCreateIdResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let machine_service = MachineService::new(state.database);

    match machine_service.create_machine(tenant_id, payload).await {
        Ok(response) => Ok(Json(response)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_machine_details(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<MachineResponse>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let machine_service = MachineService::new(state.database);

    match machine_service.get_machine_by_id(tenant_id, id).await {
        Ok(Some(machine)) => Ok(Json(machine)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_machine(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateMachineRequest>,
) -> Result<Json<MachineResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let machine_service = MachineService::new(state.database);

    match machine_service.update_machine(tenant_id, id, payload).await {
        Ok(machine) => Ok(Json(machine)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_machine(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let machine_service = MachineService::new(state.database);

    match machine_service.delete_machine(tenant_id, id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_heartbeat(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<HeartbeatRequest>,
) -> Result<StatusCode, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let machine_service = MachineService::new(state.database);

    match machine_service
        .update_heartbeat(tenant_id, id, payload)
        .await
    {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Machine-Item relationship implementations

async fn list_machine_item_relationships(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(machine_id): Path<Uuid>,
) -> Result<Json<Vec<MachineItemRelationshipResponse>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let machine_service = MachineService::new(state.database);

    match machine_service
        .list_machine_item_relationships(tenant_id, machine_id)
        .await
    {
        Ok(relationships) => Ok(Json(relationships)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn create_machine_item_relationship(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(machine_id): Path<Uuid>,
    Json(payload): Json<CreateMachineItemRelationshipRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let machine_service = MachineService::new(state.database);

    match machine_service
        .create_machine_item_relationship(tenant_id, machine_id, payload)
        .await
    {
        Ok(relationship_id) => Ok(Json(serde_json::json!({"id": relationship_id}))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_machine_item_relationship(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(relationship_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let machine_service = MachineService::new(state.database);

    match machine_service
        .delete_machine_item_relationship(tenant_id, relationship_id)
        .await
    {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Machine-Asset relationship implementations

async fn list_machine_asset_relationships(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(machine_id): Path<Uuid>,
) -> Result<Json<Vec<MachineAssetRelationshipResponse>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let machine_service = MachineService::new(state.database);

    match machine_service
        .list_machine_asset_relationships(tenant_id, machine_id)
        .await
    {
        Ok(relationships) => Ok(Json(relationships)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn create_machine_asset_relationship(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(machine_id): Path<Uuid>,
    Json(payload): Json<CreateMachineAssetRelationshipRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let machine_service = MachineService::new(state.database);

    match machine_service
        .create_machine_asset_relationship(tenant_id, machine_id, payload)
        .await
    {
        Ok(relationship_id) => Ok(Json(serde_json::json!({"id": relationship_id}))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_machine_asset_relationship(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(relationship_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let machine_service = MachineService::new(state.database);

    match machine_service
        .delete_machine_asset_relationship(tenant_id, relationship_id)
        .await
    {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Machine-Operator assignment implementations

async fn list_machine_operator_assignments(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(machine_id): Path<Uuid>,
) -> Result<Json<Vec<MachineOperatorAssignmentResponse>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let machine_service = MachineService::new(state.database);

    match machine_service
        .list_machine_operator_assignments(tenant_id, machine_id)
        .await
    {
        Ok(assignments) => Ok(Json(assignments)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn create_machine_operator_assignment(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(machine_id): Path<Uuid>,
    Json(payload): Json<CreateMachineOperatorAssignmentRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let machine_service = MachineService::new(state.database);

    match machine_service
        .create_machine_operator_assignment(tenant_id, machine_id, payload)
        .await
    {
        Ok(assignment_id) => Ok(Json(serde_json::json!({"id": assignment_id}))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_machine_operator_assignment(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(assignment_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let machine_service = MachineService::new(state.database);

    match machine_service
        .delete_machine_operator_assignment(tenant_id, assignment_id)
        .await
    {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Machine-Job assignment implementations

async fn list_machine_job_assignments(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(machine_id): Path<Uuid>,
) -> Result<Json<Vec<MachineJobAssignmentResponse>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let machine_service = MachineService::new(state.database);

    match machine_service
        .list_machine_job_assignments(tenant_id, machine_id)
        .await
    {
        Ok(assignments) => Ok(Json(assignments)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn create_machine_job_assignment(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(machine_id): Path<Uuid>,
    Json(payload): Json<CreateMachineJobAssignmentRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let machine_service = MachineService::new(state.database);

    match machine_service
        .create_machine_job_assignment(tenant_id, machine_id, payload)
        .await
    {
        Ok(assignment_id) => Ok(Json(serde_json::json!({"id": assignment_id}))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_machine_job_assignment(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(assignment_id): Path<Uuid>,
    Json(payload): Json<UpdateMachineJobAssignmentRequest>,
) -> Result<Json<MachineJobAssignmentResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let machine_service = MachineService::new(state.database);

    match machine_service
        .update_machine_job_assignment(tenant_id, assignment_id, payload)
        .await
    {
        Ok(assignment) => Ok(Json(assignment)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_machine_job_assignment(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(assignment_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let machine_service = MachineService::new(state.database);

    match machine_service
        .delete_machine_job_assignment(tenant_id, assignment_id)
        .await
    {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Utility route implementations

async fn get_machines_by_item(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(item_id): Path<Uuid>,
    Query(params): Query<ListByItemQuery>,
) -> Result<Json<Vec<MachineResponse>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let machine_service = MachineService::new(state.database);

    match machine_service
        .get_machines_by_item_id(tenant_id, item_id, params.relationship_type)
        .await
    {
        Ok(machines) => Ok(Json(machines)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_machines_by_job(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(job_id): Path<Uuid>,
    Query(params): Query<ListByJobQuery>,
) -> Result<Json<Vec<MachineResponse>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let machine_service = MachineService::new(state.database);

    match machine_service
        .get_machines_by_job_id(tenant_id, job_id, params.status)
        .await
    {
        Ok(machines) => Ok(Json(machines)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
