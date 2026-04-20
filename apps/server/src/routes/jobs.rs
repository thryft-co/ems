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
        CreateJobIdResponse, CreateJobRequest, JobPriority, JobResponse, JobStatus, JobType,
        ManufacturingJobResponse, QaJobResponse, ServiceJobResponse, UpdateJobRequest,
    },
    services::JobService,
    AppState,
};

#[derive(Deserialize)]
struct ListQuery {
    #[serde(rename = "type")]
    job_type: Option<JobType>,
    status: Option<JobStatus>,
    #[allow(dead_code)]
    priority: Option<JobPriority>,
    limit: Option<u32>,
    offset: Option<u32>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        // General Job API
        .route("/", get(list_all_jobs).post(create_job))
        .route(
            "/:id",
            get(get_job_details).put(update_job).delete(delete_job),
        )
        // Specialized Job API routes
        .route("/manufacturing", get(list_manufacturing_jobs))
        .route(
            "/manufacturing/:id",
            get(get_manufacturing_job_details).put(update_manufacturing_job),
        )
        .route("/qa", get(list_qa_jobs))
        .route("/qa/:id", get(get_qa_job_details).put(update_qa_job))
        .route("/service", get(list_service_jobs))
        .route(
            "/service/:id",
            get(get_service_job_details).put(update_service_job),
        )
}

// Helper function to extract tenant ID from request extensions
fn extract_tenant_id(tenant_context: &TenantContext) -> Uuid {
    tenant_context.tenant_id
}

// General Job API implementations

async fn list_all_jobs(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<JobResponse>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let job_service = JobService::new(state.database);

    match job_service
        .list_jobs(
            tenant_id,
            params.job_type,
            params.status,
            params.limit,
            params.offset,
        )
        .await
    {
        Ok(jobs) => Ok(Json(jobs)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn create_job(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Json(payload): Json<CreateJobRequest>,
) -> Result<Json<CreateJobIdResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let job_service = JobService::new(state.database);

    match job_service.create_job(tenant_id, payload).await {
        Ok(response) => Ok(Json(response)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_job_details(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<JobResponse>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let job_service = JobService::new(state.database);

    match job_service.get_job_by_id(tenant_id, id).await {
        Ok(Some(job)) => Ok(Json(job)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_job(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateJobRequest>,
) -> Result<Json<JobResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let job_service = JobService::new(state.database);

    match job_service.update_job(tenant_id, id, payload).await {
        Ok(job) => Ok(Json(job)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_job(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let job_service = JobService::new(state.database);

    match job_service.delete_job(tenant_id, id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Type-specific implementations
async fn list_manufacturing_jobs(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<ManufacturingJobResponse>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let job_service = JobService::new(state.database);

    match job_service
        .list_manufacturing_jobs(tenant_id, params.limit, params.offset)
        .await
    {
        Ok(jobs) => Ok(Json(jobs)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_manufacturing_job_details(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<ManufacturingJobResponse>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let job_service = JobService::new(state.database);

    match job_service.get_manufacturing_job_by_id(tenant_id, id).await {
        Ok(Some(job)) => Ok(Json(job)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_manufacturing_job(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateJobRequest>,
) -> Result<Json<ManufacturingJobResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let job_service = JobService::new(state.database);

    match job_service.update_job(tenant_id, id, payload).await {
        Ok(_) => {
            // Return the updated manufacturing job
            match job_service.get_manufacturing_job_by_id(tenant_id, id).await {
                Ok(Some(job)) => Ok(Json(job)),
                Ok(None) => Err(StatusCode::NOT_FOUND),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn list_qa_jobs(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<QaJobResponse>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let job_service = JobService::new(state.database);

    match job_service
        .list_qa_jobs(tenant_id, params.limit, params.offset)
        .await
    {
        Ok(jobs) => Ok(Json(jobs)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_qa_job_details(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<QaJobResponse>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let job_service = JobService::new(state.database);

    match job_service.get_qa_job_by_id(tenant_id, id).await {
        Ok(Some(job)) => Ok(Json(job)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_qa_job(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateJobRequest>,
) -> Result<Json<QaJobResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let job_service = JobService::new(state.database);

    match job_service.update_job(tenant_id, id, payload).await {
        Ok(_) => {
            // Return the updated QA job
            match job_service.get_qa_job_by_id(tenant_id, id).await {
                Ok(Some(job)) => Ok(Json(job)),
                Ok(None) => Err(StatusCode::NOT_FOUND),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn list_service_jobs(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<ServiceJobResponse>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let job_service = JobService::new(state.database);

    match job_service
        .list_service_jobs(tenant_id, params.limit, params.offset)
        .await
    {
        Ok(jobs) => Ok(Json(jobs)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_service_job_details(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<ServiceJobResponse>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let job_service = JobService::new(state.database);

    match job_service.get_service_job_by_id(tenant_id, id).await {
        Ok(Some(job)) => Ok(Json(job)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_service_job(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateJobRequest>,
) -> Result<Json<ServiceJobResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let job_service = JobService::new(state.database);

    match job_service.update_job(tenant_id, id, payload).await {
        Ok(_) => {
            // Return the updated service job
            match job_service.get_service_job_by_id(tenant_id, id).await {
                Ok(Some(job)) => Ok(Json(job)),
                Ok(None) => Err(StatusCode::NOT_FOUND),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
