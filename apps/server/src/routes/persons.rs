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
        CreatePersonIdResponse, CreatePersonRequest, CustomerPersonResponse,
        DistributorPersonResponse, InternalPersonResponse, PersonResponse, PersonRole,
        UpdatePersonRequest, VendorPersonResponse,
    },
    services::PersonService,
    AppState,
};

#[derive(Deserialize)]
struct ListQuery {
    #[serde(rename = "type")]
    person_type: Option<PersonRole>,
    limit: Option<u32>,
    offset: Option<u32>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        // General Person API
        .route("/", get(list_all_persons).post(create_person))
        .route(
            "/:id",
            get(get_person_details)
                .put(update_person)
                .delete(delete_person),
        )
        // Specialized Person API routes
        .route("/internal", get(list_internal_persons))
        .route(
            "/internal/:id",
            get(get_internal_person_details).put(update_internal_person),
        )
        .route("/customer", get(list_customer_persons))
        .route(
            "/customer/:id",
            get(get_customer_person_details).put(update_customer_person),
        )
        .route("/vendor", get(list_vendor_persons))
        .route(
            "/vendor/:id",
            get(get_vendor_person_details).put(update_vendor_person),
        )
        .route("/distributor", get(list_distributor_persons))
        .route(
            "/distributor/:id",
            get(get_distributor_person_details).put(update_distributor_person),
        )
}

// Helper function to extract tenant ID from request extensions
fn extract_tenant_id(tenant_context: &TenantContext) -> Uuid {
    tenant_context.tenant_id
}

// General Person API implementations

async fn list_all_persons(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<PersonResponse>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let person_service = PersonService::new(state.database);

    match person_service
        .list_persons(tenant_id, params.person_type, params.limit, params.offset)
        .await
    {
        Ok(persons) => Ok(Json(persons)),
        Err(_) => Err(StatusCode::NOT_IMPLEMENTED),
    }
}

async fn create_person(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Json(payload): Json<CreatePersonRequest>,
) -> Result<Json<CreatePersonIdResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let person_service = PersonService::new(state.database);

    match person_service.create_person(tenant_id, payload).await {
        Ok(response) => Ok(Json(response)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_person_details(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<PersonResponse>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let person_service = PersonService::new(state.database);

    match person_service.get_person_by_id(tenant_id, id).await {
        Ok(Some(person)) => Ok(Json(person)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_person(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdatePersonRequest>,
) -> Result<Json<PersonResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let person_service = PersonService::new(state.database);

    match person_service.update_person(tenant_id, id, payload).await {
        Ok(person) => Ok(Json(person)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_person(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let person_service = PersonService::new(state.database);

    match person_service.delete_person(tenant_id, id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Type-specific implementations
async fn list_internal_persons(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<InternalPersonResponse>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let person_service = PersonService::new(state.database);

    match person_service
        .list_internal_persons(tenant_id, params.limit, params.offset)
        .await
    {
        Ok(persons) => Ok(Json(persons)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_internal_person_details(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<InternalPersonResponse>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let person_service = PersonService::new(state.database);

    match person_service
        .get_internal_person_by_id(tenant_id, id)
        .await
    {
        Ok(Some(person)) => Ok(Json(person)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_internal_person(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdatePersonRequest>,
) -> Result<Json<InternalPersonResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let person_service = PersonService::new(state.database);

    match person_service
        .update_internal_person(tenant_id, id, payload)
        .await
    {
        Ok(_) => {
            // Return the updated person
            match person_service
                .get_internal_person_by_id(tenant_id, id)
                .await
            {
                Ok(Some(person)) => Ok(Json(person)),
                Ok(None) => Err(StatusCode::NOT_FOUND),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn list_customer_persons(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<CustomerPersonResponse>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let person_service = PersonService::new(state.database);

    match person_service
        .list_customer_persons(tenant_id, params.limit, params.offset)
        .await
    {
        Ok(persons) => Ok(Json(persons)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_customer_person_details(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<CustomerPersonResponse>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let person_service = PersonService::new(state.database);

    match person_service
        .get_customer_person_by_id(tenant_id, id)
        .await
    {
        Ok(Some(person)) => Ok(Json(person)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_customer_person(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdatePersonRequest>,
) -> Result<Json<CustomerPersonResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let person_service = PersonService::new(state.database);

    match person_service
        .update_customer_person(tenant_id, id, payload)
        .await
    {
        Ok(_) => {
            // Return the updated person
            match person_service
                .get_customer_person_by_id(tenant_id, id)
                .await
            {
                Ok(Some(person)) => Ok(Json(person)),
                Ok(None) => Err(StatusCode::NOT_FOUND),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn list_vendor_persons(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<VendorPersonResponse>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let person_service = PersonService::new(state.database);

    match person_service
        .list_vendor_persons(tenant_id, params.limit, params.offset)
        .await
    {
        Ok(persons) => Ok(Json(persons)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_vendor_person_details(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<VendorPersonResponse>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let person_service = PersonService::new(state.database);

    match person_service.get_vendor_person_by_id(tenant_id, id).await {
        Ok(Some(person)) => Ok(Json(person)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_vendor_person(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdatePersonRequest>,
) -> Result<Json<VendorPersonResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let person_service = PersonService::new(state.database);

    match person_service
        .update_vendor_person(tenant_id, id, payload)
        .await
    {
        Ok(_) => {
            // Return the updated person
            match person_service.get_vendor_person_by_id(tenant_id, id).await {
                Ok(Some(person)) => Ok(Json(person)),
                Ok(None) => Err(StatusCode::NOT_FOUND),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn list_distributor_persons(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<DistributorPersonResponse>>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let person_service = PersonService::new(state.database);

    match person_service
        .list_distributor_persons(tenant_id, params.limit, params.offset)
        .await
    {
        Ok(persons) => Ok(Json(persons)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_distributor_person_details(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<DistributorPersonResponse>, StatusCode> {
    let tenant_id = extract_tenant_id(&tenant_context);
    let person_service = PersonService::new(state.database);

    match person_service
        .get_distributor_person_by_id(tenant_id, id)
        .await
    {
        Ok(Some(person)) => Ok(Json(person)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_distributor_person(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdatePersonRequest>,
) -> Result<Json<DistributorPersonResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let tenant_id = extract_tenant_id(&tenant_context);
    let person_service = PersonService::new(state.database);

    match person_service
        .update_distributor_person(tenant_id, id, payload)
        .await
    {
        Ok(_) => {
            // Return the updated person
            match person_service
                .get_distributor_person_by_id(tenant_id, id)
                .await
            {
                Ok(Some(person)) => Ok(Json(person)),
                Ok(None) => Err(StatusCode::NOT_FOUND),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
