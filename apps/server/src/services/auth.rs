use anyhow::Result;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl, SimpleAsyncConnection};
use uuid::Uuid;

use crate::models::{
    AuthPersonWithoutTenant, AuthResponse, CreateAndJoinTenantRequest,
    InternalPersonOAuthRegisterRequest, JoinTenantRequest, LoginRequest, LogoutRequest,
    NewInternalPerson, NewPerson, NewTenant, NewTenantPerson, NewTokenBlacklist,
    OAuthCallbackRequest, OAuthLoginRequest, OAuthUrlResponse, Person, PersonOnlyAuthResponse,
    PersonOnlyRegisterRequest, PersonRole, RefreshTokenRequest, RefreshTokenResponse,
    RegisterRequest, Tenant, TenantPerson, TokenBlacklist,
};
use crate::schema::{internal_person, person, tenant_person, tenants, token_blacklist};
use crate::{
    core::AuthUtils,
    integrations::SupabaseService,
    repositories::{DatabaseService, TenantService},
};

pub struct AuthService {
    database: DatabaseService,
    tenant_service: TenantService,
    supabase_service: SupabaseService,
}

impl AuthService {
    pub fn new(database: DatabaseService, supabase_service: SupabaseService) -> Self {
        let tenant_service = TenantService::new(database.clone());
        Self {
            database,
            tenant_service,
            supabase_service,
        }
    }

    /// Check if a token is blacklisted
    pub async fn is_token_blacklisted(&self, token: &str, tenant_id: Uuid) -> Result<bool> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let token_hash = AuthUtils::hash_token(token);

        let blacklisted_token = token_blacklist::table
            .filter(token_blacklist::token_hash.eq(&token_hash))
            .filter(token_blacklist::expires_at.gt(Utc::now()))
            .select(TokenBlacklist::as_select())
            .first::<TokenBlacklist>(&mut conn)
            .await
            .optional()?;

        Ok(blacklisted_token.is_some())
    }

    /// Generate OAuth authorization URL
    pub async fn get_oauth_url(&self, request: OAuthLoginRequest) -> Result<OAuthUrlResponse> {
        // Validate that the tenant exists and is active
        let tenant = self
            .tenant_service
            .get_tenant_by_subdomain(&request.tenant_subdomain)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Tenant not found"))?;

        if !tenant.is_active.unwrap_or(false) {
            return Err(anyhow::anyhow!("Tenant is not active"));
        }

        // Generate a state parameter that includes tenant info
        let state = format!("{}:{}", tenant.subdomain, Uuid::new_v4());

        // Generate OAuth URL using Supabase service
        let auth_url = self
            .supabase_service
            .generate_oauth_url(&request.provider, &state)?;

        Ok(OAuthUrlResponse { auth_url, state })
    }

    /// Handle OAuth callback and authenticate user
    pub async fn oauth_callback(&self, request: OAuthCallbackRequest) -> Result<AuthResponse> {
        // Validate that the tenant exists and is active
        let tenant = self
            .tenant_service
            .get_tenant_by_subdomain(&request.tenant_subdomain)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Tenant not found"))?;

        if !tenant.is_active.unwrap_or(false) {
            return Err(anyhow::anyhow!("Tenant is not active"));
        }

        // Validate state parameter
        let expected_state_prefix = format!("{}:", tenant.subdomain);
        if !request.state.starts_with(&expected_state_prefix) {
            return Err(anyhow::anyhow!("Invalid state parameter"));
        }

        // Exchange code for user info
        let user_info = self
            .supabase_service
            .handle_oauth_callback(&request.provider, &request.code)
            .await?;

        // Find existing user by email
        let mut conn = self.database.get_connection().await?;

        let person_result = person::table
            .filter(person::email.eq(&user_info.email))
            .first::<Person>(&mut conn)
            .await
            .optional()?;

        let person = person_result.ok_or_else(|| {
            anyhow::anyhow!(
                "User not found. Please register first or contact your administrator to add you to this tenant."
            )
        })?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant.id))
            .await?;

        // Find tenant relationship - must be internal user
        let tenant_person_result = tenant_person::table
            .filter(tenant_person::person_id.eq(person.id))
            .filter(tenant_person::tenant_id.eq(tenant.id))
            .filter(tenant_person::role.eq("internal"))
            .inner_join(tenants::table.on(tenant_person::tenant_id.eq(tenants::id)))
            .select((TenantPerson::as_select(), Tenant::as_select()))
            .first::<(TenantPerson, Tenant)>(&mut conn)
            .await
            .optional()?;

        let (tenant_person, tenant) = tenant_person_result
            .ok_or_else(|| anyhow::anyhow!("User is not an internal person for this tenant"))?;

        // Parse role
        let role = PersonRole::try_from(tenant_person.role)
            .map_err(|e| anyhow::anyhow!("Invalid role: {}", e))?;

        // Generate JWT tokens
        let access_token = AuthUtils::generate_access_token(person.id, tenant.id, &role)?;
        let refresh_token = AuthUtils::generate_refresh_token(person.id, tenant.id)?;

        // Update last_login timestamp
        diesel::update(person::table.filter(person::id.eq(person.id)))
            .set(person::last_login.eq(Some(Utc::now())))
            .execute(&mut conn)
            .await?;

        // Create response
        let auth_user = AuthUtils::create_auth_user(person.id, person.email, person.name, role);
        let auth_tenant = AuthUtils::create_auth_tenant(tenant.id, tenant.name, tenant.subdomain);

        Ok(AuthResponse {
            access_token,
            refresh_token,
            user: auth_user,
            tenant: auth_tenant,
        })
    }

    /// Register a new internal person using OAuth
    pub async fn oauth_register_internal_person(
        &self,
        request: InternalPersonOAuthRegisterRequest,
    ) -> Result<AuthResponse> {
        let mut conn = self.database.get_connection().await?;

        // Check if tenant subdomain is available
        if self
            .tenant_service
            .get_tenant_by_subdomain(&request.tenant_subdomain)
            .await?
            .is_some()
        {
            return Err(anyhow::anyhow!("Tenant subdomain already exists"));
        }

        // Validate state parameter
        let expected_state_prefix = format!("{}:", request.tenant_subdomain);
        if !request.state.starts_with(&expected_state_prefix) {
            return Err(anyhow::anyhow!("Invalid state parameter"));
        }

        // Exchange code for user info
        let user_info = self
            .supabase_service
            .handle_oauth_callback(&request.provider, &request.code)
            .await?;

        // Check if email is already registered
        let existing_person = person::table
            .filter(person::email.eq(&user_info.email))
            .first::<Person>(&mut conn)
            .await
            .optional()?;

        if existing_person.is_some() {
            return Err(anyhow::anyhow!("Email already registered"));
        }

        let result = conn
            .transaction::<_, diesel::result::Error, _>(|conn| {
                Box::pin(async move {
                    // Create new tenant
                    let new_tenant = NewTenant {
                        name: request.tenant_name.clone(),
                        subdomain: request.tenant_subdomain.clone(),
                        database_url: None,
                        settings: Some(serde_json::json!({})),
                        is_active: Some(true),
                    };

                    let tenant: Tenant = diesel::insert_into(tenants::table)
                        .values(&new_tenant)
                        .returning(Tenant::as_returning())
                        .get_result(conn)
                        .await?;

                    // Set tenant context for RLS
                    conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant.id))
                        .await
                        .map_err(|_| diesel::result::Error::RollbackTransaction)?;

                    // Create a dummy Supabase UID for OAuth users (they don't have a Supabase Auth record)
                    let supabase_uid = Uuid::new_v4();

                    // Create user account
                    let new_person = NewPerson {
                        supabase_uid,
                        name: user_info.name.clone(),
                        email: user_info.email.clone(),
                        phone: None,
                        global_access: Some(vec![]),
                        is_active: Some(true),
                    };

                    let person: Person = diesel::insert_into(person::table)
                        .values(&new_person)
                        .returning(Person::as_returning())
                        .get_result(conn)
                        .await?;

                    // Create tenant-person relationship with internal role
                    let new_tenant_person = NewTenantPerson {
                        person_id: person.id,
                        tenant_id: tenant.id,
                        role: PersonRole::Internal.to_string(),
                        access_level: Some(vec![Some("admin".to_string())]),
                        is_primary: Some(true),
                    };

                    let _tenant_person: TenantPerson = diesel::insert_into(tenant_person::table)
                        .values(&new_tenant_person)
                        .returning(TenantPerson::as_returning())
                        .get_result(conn)
                        .await?;

                    // Create internal person specific record
                    let new_internal_person = NewInternalPerson {
                        person_id: person.id,
                        tenant_id: tenant.id,
                        department: request.department.clone(),
                        position: request.position.clone(),
                        employee_id: request.employee_id.clone(),
                        hire_date: Some(Utc::now()),
                    };

                    diesel::insert_into(internal_person::table)
                        .values(&new_internal_person)
                        .execute(conn)
                        .await?;

                    Ok((person, tenant))
                })
            })
            .await;

        let (person, tenant) = result
            .map_err(|e| anyhow::anyhow!("Failed to create internal person and tenant: {:?}", e))?;

        // Generate JWT tokens
        let role = PersonRole::Internal;
        let access_token = AuthUtils::generate_access_token(person.id, tenant.id, &role)?;
        let refresh_token = AuthUtils::generate_refresh_token(person.id, tenant.id)?;

        // Create response
        let auth_user = AuthUtils::create_auth_user(person.id, person.email, person.name, role);
        let auth_tenant = AuthUtils::create_auth_tenant(tenant.id, tenant.name, tenant.subdomain);

        Ok(AuthResponse {
            access_token,
            refresh_token,
            user: auth_user,
            tenant: auth_tenant,
        })
    }

    pub async fn login(&self, request: LoginRequest) -> Result<AuthResponse> {
        // 1. Authenticate with Supabase using direct HTTP requests
        let _supabase_session = self
            .supabase_service
            .authenticate_user(&request.email, &request.password)
            .await
            .map_err(|e| {
                tracing::error!(
                    "Supabase authentication failed for {}: {}",
                    request.email,
                    e
                );
                anyhow::anyhow!("Authentication failed: Invalid email or password")
            })?;

        let mut conn = self.database.get_connection().await?;

        // 2. Find user by email in our database
        let person_result = person::table
            .filter(person::email.eq(&request.email))
            .first::<Person>(&mut conn)
            .await
            .optional()?;

        let person = person_result
            .ok_or_else(|| anyhow::anyhow!("User not found in system. Please register first."))?;

        // 3. Find tenant relationship
        let tenant_person_result = if let Some(tenant_subdomain) = &request.tenant_subdomain {
            // Get tenant by subdomain first
            let tenant = self
                .tenant_service
                .get_tenant_by_subdomain(tenant_subdomain)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Tenant not found"))?;

            // Set tenant context for RLS
            conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant.id))
                .await?;

            // Get tenant_person relationship
            tenant_person::table
                .filter(tenant_person::person_id.eq(person.id))
                .filter(tenant_person::tenant_id.eq(tenant.id))
                .inner_join(tenants::table.on(tenant_person::tenant_id.eq(tenants::id)))
                .select((TenantPerson::as_select(), Tenant::as_select()))
                .first::<(TenantPerson, Tenant)>(&mut conn)
                .await
                .optional()?
        } else {
            // Get primary tenant relationship
            tenant_person::table
                .filter(tenant_person::person_id.eq(person.id))
                .filter(tenant_person::is_primary.eq(true))
                .inner_join(tenants::table.on(tenant_person::tenant_id.eq(tenants::id)))
                .select((TenantPerson::as_select(), Tenant::as_select()))
                .first::<(TenantPerson, Tenant)>(&mut conn)
                .await
                .optional()?
        };

        let (tenant_person, tenant) = tenant_person_result
            .ok_or_else(|| anyhow::anyhow!("No tenant relationship found for user"))?;

        // 4. Parse role
        let role = PersonRole::try_from(tenant_person.role)
            .map_err(|e| anyhow::anyhow!("Invalid role: {}", e))?;

        // 5. Generate JWT tokens
        let access_token = AuthUtils::generate_access_token(person.id, tenant.id, &role)?;
        let refresh_token = AuthUtils::generate_refresh_token(person.id, tenant.id)?;

        // 6. Update last_login timestamp
        diesel::update(person::table.filter(person::id.eq(person.id)))
            .set(person::last_login.eq(Some(Utc::now())))
            .execute(&mut conn)
            .await?;

        // 7. Create response
        let auth_user = AuthUtils::create_auth_user(person.id, person.email, person.name, role);
        let auth_tenant = AuthUtils::create_auth_tenant(tenant.id, tenant.name, tenant.subdomain);

        Ok(AuthResponse {
            access_token,
            refresh_token,
            user: auth_user,
            tenant: auth_tenant,
        })
    }

    pub async fn register(&self, request: RegisterRequest) -> Result<AuthResponse> {
        let mut conn = self.database.get_connection().await?;

        // 1. Check if tenant subdomain is available
        if self
            .tenant_service
            .get_tenant_by_subdomain(&request.tenant_subdomain)
            .await?
            .is_some()
        {
            return Err(anyhow::anyhow!("Tenant subdomain already exists"));
        }

        // 2. Check if email is already registered
        let existing_person = person::table
            .filter(person::email.eq(&request.email))
            .first::<Person>(&mut conn)
            .await
            .optional()?;

        if existing_person.is_some() {
            return Err(anyhow::anyhow!("Email already registered"));
        }

        let result = conn
            .transaction::<_, diesel::result::Error, _>(|conn| {
                let supabase_service = self.supabase_service.clone();
                let email = request.email.clone();
                let password = request.password.clone();
                let first_name = request.first_name.clone();
                let last_name = request.last_name.clone();
                let tenant_subdomain = request.tenant_subdomain.clone();
                let tenant_name = request.tenant_name.clone();

                Box::pin(async move {
                    // 3. Register with Supabase using direct HTTP requests
                    let user_metadata = serde_json::json!({
                        "first_name": first_name,
                        "last_name": last_name,
                        "tenant_subdomain": tenant_subdomain
                    });

                    let supabase_user = supabase_service
                        .create_user(&email, &password, Some(user_metadata))
                        .await
                        .map_err(|e| {
                            tracing::error!("Supabase user creation failed: {}", e);
                            diesel::result::Error::RollbackTransaction
                        })?;

                    // 4. Create new tenant
                    let new_tenant = NewTenant {
                        name: tenant_name,
                        subdomain: tenant_subdomain,
                        database_url: None,
                        settings: Some(serde_json::json!({})),
                        is_active: Some(true),
                    };

                    let tenant: Tenant = diesel::insert_into(tenants::table)
                        .values(&new_tenant)
                        .returning(Tenant::as_returning())
                        .get_result(conn)
                        .await?;

                    // Set tenant context for RLS
                    conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant.id))
                        .await
                        .map_err(|_| diesel::result::Error::RollbackTransaction)?;

                    // 5. Parse Supabase UUID
                    let supabase_uid =
                        Uuid::parse_str(supabase_user["id"].as_str().unwrap_or_default())
                            .map_err(|_| diesel::result::Error::RollbackTransaction)?;

                    // 6. Create user account with TenantAdmin role
                    let full_name = format!("{} {}", first_name, last_name);
                    let new_person = NewPerson {
                        supabase_uid,
                        name: full_name,
                        email: email.clone(),
                        phone: None,
                        global_access: Some(vec![Some("admin".to_string())]),
                        is_active: Some(true),
                    };

                    let person: Person = diesel::insert_into(person::table)
                        .values(&new_person)
                        .returning(Person::as_returning())
                        .get_result(conn)
                        .await?;

                    // 7. Create tenant_person relationship with internal role
                    let new_tenant_person = NewTenantPerson {
                        person_id: person.id,
                        tenant_id: tenant.id,
                        role: "internal".to_string(),
                        access_level: Some(vec![Some("admin".to_string())]),
                        is_primary: Some(true),
                    };

                    diesel::insert_into(tenant_person::table)
                        .values(&new_tenant_person)
                        .execute(conn)
                        .await?;

                    Ok((person, tenant))
                })
            })
            .await
            .map_err(|e| anyhow::anyhow!("Registration transaction failed: {}", e))?;

        let (person, tenant) = result;

        // 8. Generate JWT tokens
        let role = PersonRole::Internal;
        let access_token = AuthUtils::generate_access_token(person.id, tenant.id, &role)?;
        let refresh_token = AuthUtils::generate_refresh_token(person.id, tenant.id)?;

        // 9. Create response
        let auth_user = AuthUtils::create_auth_user(person.id, person.email, person.name, role);
        let auth_tenant = AuthUtils::create_auth_tenant(tenant.id, tenant.name, tenant.subdomain);

        Ok(AuthResponse {
            access_token,
            refresh_token,
            user: auth_user,
            tenant: auth_tenant,
        })
    }

    /// Register a person without creating a tenant
    pub async fn person_only_register(
        &self,
        request: PersonOnlyRegisterRequest,
    ) -> Result<PersonOnlyAuthResponse> {
        let mut conn = self.database.get_connection().await?;

        // 1. Check if email is already registered
        let existing_person = person::table
            .filter(person::email.eq(&request.email))
            .first::<Person>(&mut conn)
            .await
            .optional()?;

        if existing_person.is_some() {
            return Err(anyhow::anyhow!("Email already registered"));
        }

        // 2. Register with Supabase using direct HTTP requests
        let user_metadata = serde_json::json!({
            "first_name": request.first_name,
            "last_name": request.last_name
        });

        let supabase_user = self
            .supabase_service
            .create_user(&request.email, &request.password, Some(user_metadata))
            .await?;

        // 3. Parse Supabase UUID
        let supabase_uid = Uuid::parse_str(supabase_user["id"].as_str().unwrap_or_default())?;

        // 4. Create person account without tenant association
        let full_name = format!("{} {}", request.first_name, request.last_name);
        let new_person = NewPerson {
            supabase_uid,
            name: full_name,
            email: request.email.clone(),
            phone: None,
            global_access: Some(vec![]),
            is_active: Some(true),
        };

        let person: Person = diesel::insert_into(person::table)
            .values(&new_person)
            .returning(Person::as_returning())
            .get_result(&mut conn)
            .await?;

        // 5. Generate temporary JWT tokens without tenant (empty tenant_id for now)
        let access_token = AuthUtils::generate_temporary_access_token(person.id)?;
        let refresh_token = AuthUtils::generate_temporary_refresh_token(person.id)?;

        // 6. Create response
        let auth_person = AuthPersonWithoutTenant {
            id: person.id,
            email: person.email,
            first_name: request.first_name,
            last_name: request.last_name,
        };

        Ok(PersonOnlyAuthResponse {
            access_token,
            refresh_token,
            person: auth_person,
        })
    }

    /// Login without tenant - for users who haven't joined a tenant yet
    pub async fn person_only_login(&self, request: LoginRequest) -> Result<PersonOnlyAuthResponse> {
        // 1. Authenticate with Supabase
        let _supabase_session = self
            .supabase_service
            .authenticate_user(&request.email, &request.password)
            .await
            .map_err(|e| {
                tracing::error!(
                    "Supabase authentication failed for {}: {}",
                    request.email,
                    e
                );
                anyhow::anyhow!("Authentication failed: Invalid email or password")
            })?;

        let mut conn = self.database.get_connection().await?;

        // 2. Find user by email in our database
        let person_result = person::table
            .filter(person::email.eq(&request.email))
            .first::<Person>(&mut conn)
            .await
            .optional()?;

        let person = person_result
            .ok_or_else(|| anyhow::anyhow!("User not found in system. Please register first."))?;

        // 3. Update last_login timestamp
        diesel::update(person::table.filter(person::id.eq(person.id)))
            .set(person::last_login.eq(Some(Utc::now())))
            .execute(&mut conn)
            .await?;

        // 4. Generate temporary JWT tokens without tenant
        let access_token = AuthUtils::generate_temporary_access_token(person.id)?;
        let refresh_token = AuthUtils::generate_temporary_refresh_token(person.id)?;

        // 5. Extract first/last name from full name
        let name_parts: Vec<&str> = person.name.split_whitespace().collect();
        let first_name = name_parts.get(0).map(|s| s.to_string()).unwrap_or_default();
        let last_name = if name_parts.len() > 1 {
            name_parts[1..].join(" ")
        } else {
            String::new()
        };

        // 6. Create response
        let auth_person = AuthPersonWithoutTenant {
            id: person.id,
            email: person.email,
            first_name,
            last_name,
        };

        Ok(PersonOnlyAuthResponse {
            access_token,
            refresh_token,
            person: auth_person,
        })
    }

    /// Associate person with an existing tenant
    pub async fn join_existing_tenant(
        &self,
        person_id: Uuid,
        request: JoinTenantRequest,
    ) -> Result<AuthResponse> {
        let mut conn = self.database.get_connection().await?;

        // 1. Get the tenant by subdomain
        let tenant = self
            .tenant_service
            .get_tenant_by_subdomain(&request.tenant_subdomain)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Tenant not found"))?;

        if !tenant.is_active.unwrap_or(false) {
            return Err(anyhow::anyhow!("Tenant is not active"));
        }

        // 2. Get the person
        let person = person::table
            .filter(person::id.eq(person_id))
            .first::<Person>(&mut conn)
            .await?;

        // 3. Check if person is already associated with this tenant
        let existing_association = tenant_person::table
            .filter(tenant_person::person_id.eq(person.id))
            .filter(tenant_person::tenant_id.eq(tenant.id))
            .first::<TenantPerson>(&mut conn)
            .await
            .optional()?;

        if existing_association.is_some() {
            return Err(anyhow::anyhow!(
                "Person is already associated with this tenant"
            ));
        }

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant.id))
            .await?;

        // 4. Create tenant-person relationship
        let new_tenant_person = NewTenantPerson {
            person_id: person.id,
            tenant_id: tenant.id,
            role: PersonRole::Internal.to_string(),
            access_level: Some(vec![Some("standard".to_string())]),
            is_primary: Some(false), // Not primary since they're joining existing
        };

        diesel::insert_into(tenant_person::table)
            .values(&new_tenant_person)
            .execute(&mut conn)
            .await?;

        // 5. Generate JWT tokens with tenant context
        let role = PersonRole::Internal;
        let access_token = AuthUtils::generate_access_token(person.id, tenant.id, &role)?;
        let refresh_token = AuthUtils::generate_refresh_token(person.id, tenant.id)?;

        // 6. Create response
        let auth_user = AuthUtils::create_auth_user(person.id, person.email, person.name, role);
        let auth_tenant = AuthUtils::create_auth_tenant(tenant.id, tenant.name, tenant.subdomain);

        Ok(AuthResponse {
            access_token,
            refresh_token,
            user: auth_user,
            tenant: auth_tenant,
        })
    }

    /// Create a new tenant and associate person with it
    pub async fn create_and_join_tenant(
        &self,
        person_id: Uuid,
        request: CreateAndJoinTenantRequest,
    ) -> Result<AuthResponse> {
        let mut conn = self.database.get_connection().await?;

        // 1. Check if tenant subdomain is available
        if self
            .tenant_service
            .get_tenant_by_subdomain(&request.tenant_subdomain)
            .await?
            .is_some()
        {
            return Err(anyhow::anyhow!("Tenant subdomain already exists"));
        }

        // 2. Get the person
        let person = person::table
            .filter(person::id.eq(person_id))
            .first::<Person>(&mut conn)
            .await?;

        // Extract needed values before the transaction
        let person_id = person.id;
        let person_email = person.email.clone();
        let person_name = person.name.clone();

        let result = conn
            .transaction::<_, diesel::result::Error, _>(move |conn| {
                let tenant_name = request.tenant_name.clone();
                let tenant_subdomain = request.tenant_subdomain.clone();

                Box::pin(async move {
                    // 3. Create new tenant
                    let new_tenant = NewTenant {
                        name: tenant_name,
                        subdomain: tenant_subdomain,
                        database_url: None,
                        settings: Some(serde_json::json!({})),
                        is_active: Some(true),
                    };

                    let tenant: Tenant = diesel::insert_into(tenants::table)
                        .values(&new_tenant)
                        .returning(Tenant::as_returning())
                        .get_result(conn)
                        .await?;

                    // Set tenant context for RLS
                    conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant.id))
                        .await
                        .map_err(|_| diesel::result::Error::RollbackTransaction)?;

                    // 4. Create tenant-person relationship with admin role since they created the tenant
                    let new_tenant_person = NewTenantPerson {
                        person_id,
                        tenant_id: tenant.id,
                        role: PersonRole::Internal.to_string(),
                        access_level: Some(vec![Some("admin".to_string())]),
                        is_primary: Some(true),
                    };

                    diesel::insert_into(tenant_person::table)
                        .values(&new_tenant_person)
                        .execute(conn)
                        .await?;

                    Ok(tenant)
                })
            })
            .await
            .map_err(|e| {
                anyhow::anyhow!("Failed to create tenant and associate person: {:?}", e)
            })?;

        let tenant = result;

        // 5. Generate JWT tokens with tenant context
        let role = PersonRole::Internal;
        let access_token = AuthUtils::generate_access_token(person_id, tenant.id, &role)?;
        let refresh_token = AuthUtils::generate_refresh_token(person_id, tenant.id)?;

        // 6. Create response
        let auth_user = AuthUtils::create_auth_user(person_id, person_email, person_name, role);
        let auth_tenant = AuthUtils::create_auth_tenant(tenant.id, tenant.name, tenant.subdomain);

        Ok(AuthResponse {
            access_token,
            refresh_token,
            user: auth_user,
            tenant: auth_tenant,
        })
    }

    pub async fn refresh_token(
        &self,
        request: RefreshTokenRequest,
    ) -> Result<RefreshTokenResponse> {
        let mut conn = self.database.get_connection().await?;

        // Verify JWT token and extract claims
        let claims = AuthUtils::verify_jwt_token(&request.refresh_token)?;

        // Parse user and tenant IDs from claims
        let person_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| anyhow::anyhow!("Invalid person ID in token"))?;
        let tenant_id = Uuid::parse_str(&claims.tenant_id)
            .map_err(|_| anyhow::anyhow!("Invalid tenant ID in token"))?;

        // Check if refresh token is blacklisted
        if self
            .is_token_blacklisted(&request.refresh_token, tenant_id)
            .await?
        {
            return Err(anyhow::anyhow!("Token has been revoked"));
        }

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        // Verify the person-tenant relationship still exists and get role
        let tenant_person = tenant_person::table
            .filter(tenant_person::tenant_id.eq(tenant_id))
            .filter(tenant_person::person_id.eq(person_id))
            .select(TenantPerson::as_select())
            .first::<TenantPerson>(&mut conn)
            .await?;

        // Parse role
        let role = PersonRole::try_from(tenant_person.role)
            .map_err(|e| anyhow::anyhow!("Invalid role: {}", e))?;

        // Generate new tokens
        let new_access_token = AuthUtils::generate_access_token(person_id, tenant_id, &role)?;
        let new_refresh_token = AuthUtils::generate_refresh_token(person_id, tenant_id)?;

        Ok(RefreshTokenResponse {
            access_token: new_access_token,
            refresh_token: new_refresh_token,
        })
    }

    pub async fn logout(
        &self,
        request: LogoutRequest,
        access_token: &str,
        person_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<()> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        // 1. Validate refresh token to ensure it's valid before blacklisting
        let token_data = AuthUtils::validate_token(&request.refresh_token)?;
        let claims = token_data.claims;

        // Check if it's actually a refresh token
        if claims.role != "refresh" {
            return Err(anyhow::anyhow!("Invalid refresh token"));
        }

        // Check if token is already blacklisted
        if self
            .is_token_blacklisted(&request.refresh_token, tenant_id)
            .await?
        {
            return Err(anyhow::anyhow!("Token has been revoked"));
        }

        // Ensure the token belongs to the requesting person
        let token_person_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| anyhow::anyhow!("Invalid person ID in token"))?;
        let token_tenant_id = Uuid::parse_str(&claims.tenant_id)
            .map_err(|_| anyhow::anyhow!("Invalid tenant ID in token"))?;

        if token_person_id != person_id || token_tenant_id != tenant_id {
            return Err(anyhow::anyhow!(
                "Token does not belong to the requesting person"
            ));
        }

        // 2. Validate and blacklist access token
        let access_token_data = AuthUtils::validate_token(access_token)?;
        let access_claims = access_token_data.claims;

        let access_token_hash = AuthUtils::hash_token(access_token);
        let access_expires_at = chrono::DateTime::from_timestamp(access_claims.exp as i64, 0)
            .ok_or_else(|| anyhow::anyhow!("Invalid access token expiration time"))?
            .with_timezone(&Utc);

        let access_blacklist_entry = NewTokenBlacklist {
            token_hash: access_token_hash,
            token_type: "access".to_string(),
            person_id,
            tenant_id,
            expires_at: access_expires_at,
        };

        diesel::insert_into(token_blacklist::table)
            .values(&access_blacklist_entry)
            .execute(&mut conn)
            .await?;

        // 3. Add refresh token to blacklist
        let token_hash = AuthUtils::hash_token(&request.refresh_token);
        let expires_at = chrono::DateTime::from_timestamp(claims.exp as i64, 0)
            .ok_or_else(|| anyhow::anyhow!("Invalid token expiration time"))?
            .with_timezone(&Utc);

        let new_blacklist_entry = NewTokenBlacklist {
            token_hash,
            token_type: "refresh".to_string(),
            person_id,
            tenant_id,
            expires_at,
        };

        diesel::insert_into(token_blacklist::table)
            .values(&new_blacklist_entry)
            .execute(&mut conn)
            .await?;

        Ok(())
    }
}
