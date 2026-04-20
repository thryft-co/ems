use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::post,
    Router,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    models::{
        AuthResponse, CreateAndJoinTenantRequest, InternalPersonOAuthRegisterRequest,
        JoinTenantRequest, LoginRequest, LogoutRequest, OAuthCallbackRequest, OAuthLoginRequest,
        OAuthUrlResponse, PersonOnlyAuthResponse, PersonOnlyRegisterRequest, RefreshTokenRequest,
        RefreshTokenResponse, RegisterRequest,
    },
    core::AuthUtils,
    services::AuthService,
    AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/person-register", post(person_only_register))
        .route("/person-login", post(person_only_login))
        .route("/join-tenant", post(join_tenant))
        .route("/create-tenant", post(create_tenant))
        .route("/refresh", post(refresh_token))
        .route("/logout", post(logout))
        // OAuth routes
        .route("/oauth/url", post(oauth_get_url))
        .route("/oauth/callback", post(oauth_callback))
        .route("/oauth/register/internal", post(oauth_register_internal))
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Create auth service
    let auth_service = AuthService::new(state.database, state.supabase);

    // Authenticate person
    match auth_service.login(payload).await {
        Ok(auth_response) => Ok(Json(auth_response)),
        Err(e) => {
            tracing::error!("Login failed: {}", e);
            match e.to_string().as_str() {
                s if s.contains("Person not found") => Err(StatusCode::NOT_FOUND),
                s if s.contains("Authentication failed") => Err(StatusCode::UNAUTHORIZED),
                s if s.contains("Tenant not found") => Err(StatusCode::NOT_FOUND),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Create auth service
    let auth_service = AuthService::new(state.database, state.supabase);

    // Register person
    match auth_service.register(payload).await {
        Ok(auth_response) => Ok(Json(auth_response)),
        Err(e) => {
            tracing::error!("Registration failed: {}", e);
            match e.to_string().as_str() {
                s if s.contains("subdomain already exists") => Err(StatusCode::CONFLICT),
                s if s.contains("Email already registered") => Err(StatusCode::CONFLICT),
                s if s.contains("Registration failed") => Err(StatusCode::BAD_REQUEST),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

async fn person_only_register(
    State(state): State<AppState>,
    Json(payload): Json<PersonOnlyRegisterRequest>,
) -> Result<Json<PersonOnlyAuthResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Create auth service
    let auth_service = AuthService::new(state.database, state.supabase);

    // Register person without tenant
    match auth_service.person_only_register(payload).await {
        Ok(auth_response) => Ok(Json(auth_response)),
        Err(e) => {
            tracing::error!("Person-only registration failed: {}", e);
            match e.to_string().as_str() {
                s if s.contains("Email already registered") => Err(StatusCode::CONFLICT),
                s if s.contains("Registration failed") => Err(StatusCode::BAD_REQUEST),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

async fn person_only_login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<PersonOnlyAuthResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Create auth service
    let auth_service = AuthService::new(state.database, state.supabase);

    // Login person without tenant requirement
    match auth_service.person_only_login(payload).await {
        Ok(auth_response) => Ok(Json(auth_response)),
        Err(e) => {
            tracing::error!("Person-only login failed: {}", e);
            match e.to_string().as_str() {
                s if s.contains("Authentication failed") => Err(StatusCode::UNAUTHORIZED),
                s if s.contains("User not found") => Err(StatusCode::NOT_FOUND),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

async fn join_tenant(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<JoinTenantRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Extract person ID from JWT token
    let person_id = match extract_person_id_from_headers(&headers) {
        Ok(id) => id,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    // Create auth service
    let auth_service = AuthService::new(state.database, state.supabase);

    // Join existing tenant
    match auth_service.join_existing_tenant(person_id, payload).await {
        Ok(auth_response) => Ok(Json(auth_response)),
        Err(e) => {
            tracing::error!("Join tenant failed: {}", e);
            match e.to_string().as_str() {
                s if s.contains("Tenant not found") => Err(StatusCode::NOT_FOUND),
                s if s.contains("not active") => Err(StatusCode::FORBIDDEN),
                s if s.contains("already associated") => Err(StatusCode::CONFLICT),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

async fn create_tenant(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateAndJoinTenantRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Extract person ID from JWT token
    let person_id = match extract_person_id_from_headers(&headers) {
        Ok(id) => id,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    // Create auth service
    let auth_service = AuthService::new(state.database, state.supabase);

    // Create new tenant and associate person
    match auth_service
        .create_and_join_tenant(person_id, payload)
        .await
    {
        Ok(auth_response) => Ok(Json(auth_response)),
        Err(e) => {
            tracing::error!("Create tenant failed: {}", e);
            match e.to_string().as_str() {
                s if s.contains("subdomain already exists") => Err(StatusCode::CONFLICT),
                s if s.contains("Failed to create tenant") => Err(StatusCode::BAD_REQUEST),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<RefreshTokenResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Create auth service
    let auth_service = AuthService::new(state.database, state.supabase);

    // Refresh token
    match auth_service.refresh_token(payload).await {
        Ok(refresh_response) => Ok(Json(refresh_response)),
        Err(e) => {
            tracing::error!("Token refresh failed: {}", e);
            match e.to_string().as_str() {
                s if s.contains("Invalid refresh token") => Err(StatusCode::UNAUTHORIZED),
                s if s.contains("not found or inactive") => Err(StatusCode::UNAUTHORIZED),
                s if s.contains("Failed to validate token") => Err(StatusCode::UNAUTHORIZED),
                s if s.contains("Token has been revoked") => Err(StatusCode::UNAUTHORIZED),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LogoutRequest>,
) -> Result<StatusCode, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Extract JWT token from Authorization header
    let auth_header = headers
        .get("authorization")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let auth_str = auth_header.to_str().map_err(|_| StatusCode::UNAUTHORIZED)?;

    if !auth_str.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let access_token = &auth_str[7..]; // Remove "Bearer " prefix

    // Verify JWT token and extract claims
    let claims = AuthUtils::verify_jwt_token(access_token).map_err(|e| {
        tracing::error!("Token verification failed: {}", e);
        StatusCode::UNAUTHORIZED
    })?;

    // Parse person and tenant IDs from claims
    let person_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::BAD_REQUEST)?;
    let tenant_id = Uuid::parse_str(&claims.tenant_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Validate X-Tenant-ID header matches token tenant
    if let Some(header_tenant_id) = headers.get("X-Tenant-ID") {
        if let Ok(header_tenant_str) = header_tenant_id.to_str() {
            if let Ok(header_tenant_uuid) = Uuid::parse_str(header_tenant_str) {
                if header_tenant_uuid != tenant_id {
                    return Err(StatusCode::FORBIDDEN);
                }
            }
        }
    }

    let auth_service = AuthService::new(state.database.clone(), state.supabase.clone());

    // Check if access token is already blacklisted
    match auth_service
        .is_token_blacklisted(access_token, tenant_id)
        .await
    {
        Ok(true) => {
            tracing::warn!("Attempted to use blacklisted access token");
            return Err(StatusCode::UNAUTHORIZED);
        }
        Ok(false) => {
            // Token is valid, proceed with logout
        }
        Err(e) => {
            tracing::error!("Failed to check token blacklist: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    match auth_service
        .logout(payload, access_token, person_id, tenant_id)
        .await
    {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            tracing::error!("Logout failed: {}", e);
            match e.to_string().as_str() {
                s if s.contains("Invalid refresh token") => Err(StatusCode::BAD_REQUEST),
                s if s.contains("Token does not belong") => Err(StatusCode::FORBIDDEN),
                s if s.contains("Token has been revoked") => Err(StatusCode::UNAUTHORIZED),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

// OAuth endpoint implementations

async fn oauth_get_url(
    State(state): State<AppState>,
    Json(payload): Json<OAuthLoginRequest>,
) -> Result<Json<OAuthUrlResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Create auth service
    let auth_service = AuthService::new(state.database, state.supabase);

    // Get OAuth URL
    match auth_service.get_oauth_url(payload).await {
        Ok(oauth_url_response) => Ok(Json(oauth_url_response)),
        Err(e) => {
            tracing::error!("OAuth URL generation failed: {}", e);
            match e.to_string().as_str() {
                s if s.contains("Tenant not found") => Err(StatusCode::NOT_FOUND),
                s if s.contains("not active") => Err(StatusCode::FORBIDDEN),
                s if s.contains("not configured") => Err(StatusCode::SERVICE_UNAVAILABLE),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

async fn oauth_callback(
    State(state): State<AppState>,
    Json(payload): Json<OAuthCallbackRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Create auth service
    let auth_service = AuthService::new(state.database, state.supabase);

    // Handle OAuth callback
    match auth_service.oauth_callback(payload).await {
        Ok(auth_response) => Ok(Json(auth_response)),
        Err(e) => {
            tracing::error!("OAuth callback failed: {}", e);
            match e.to_string().as_str() {
                s if s.contains("Tenant not found") => Err(StatusCode::NOT_FOUND),
                s if s.contains("not active") => Err(StatusCode::FORBIDDEN),
                s if s.contains("Invalid state") => Err(StatusCode::BAD_REQUEST),
                s if s.contains("Person not found") => Err(StatusCode::NOT_FOUND),
                s if s.contains("not an internal person") => Err(StatusCode::FORBIDDEN),
                s if s.contains("Failed to exchange") => Err(StatusCode::BAD_REQUEST),
                s if s.contains("not configured") => Err(StatusCode::SERVICE_UNAVAILABLE),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

async fn oauth_register_internal(
    State(state): State<AppState>,
    Json(payload): Json<InternalPersonOAuthRegisterRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    // Validate the request
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Create auth service
    let auth_service = AuthService::new(state.database, state.supabase);

    // Register internal person via OAuth
    match auth_service.oauth_register_internal_person(payload).await {
        Ok(auth_response) => Ok(Json(auth_response)),
        Err(e) => {
            tracing::error!("OAuth internal person registration failed: {}", e);
            match e.to_string().as_str() {
                s if s.contains("subdomain already exists") => Err(StatusCode::CONFLICT),
                s if s.contains("Email already registered") => Err(StatusCode::CONFLICT),
                s if s.contains("Invalid state") => Err(StatusCode::BAD_REQUEST),
                s if s.contains("Failed to exchange") => Err(StatusCode::BAD_REQUEST),
                s if s.contains("not configured") => Err(StatusCode::SERVICE_UNAVAILABLE),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

// Helper function to extract person ID from JWT token in headers
fn extract_person_id_from_headers(headers: &HeaderMap) -> Result<Uuid, anyhow::Error> {
    let auth_header = headers
        .get("authorization")
        .ok_or_else(|| anyhow::anyhow!("Missing authorization header"))?
        .to_str()
        .map_err(|_| anyhow::anyhow!("Invalid authorization header"))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| anyhow::anyhow!("Invalid authorization format"))?;

    let claims = AuthUtils::validate_token(token)?;
    Uuid::parse_str(&claims.claims.sub).map_err(|_| anyhow::anyhow!("Invalid person ID in token"))
}
