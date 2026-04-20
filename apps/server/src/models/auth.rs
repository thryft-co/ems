use crate::models::person::PersonRole;
use regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 1))]
    pub password: String,

    pub tenant_subdomain: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 1, max = 50))]
    pub first_name: String,

    #[validate(length(min = 1, max = 50))]
    pub last_name: String,

    #[validate(length(min = 8, max = 128))]
    pub password: String,

    #[validate(length(min = 1, max = 50), regex(path = "SUBDOMAIN_REGEX"))]
    pub tenant_subdomain: String,

    #[validate(length(min = 1, max = 100))]
    pub tenant_name: String,
}

// OAuth-related enums and structs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OAuthProvider {
    #[serde(rename = "google")]
    Google,
    #[serde(rename = "microsoft")]
    Microsoft,
    #[serde(rename = "apple")]
    Apple,
}

impl std::fmt::Display for OAuthProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OAuthProvider::Google => write!(f, "google"),
            OAuthProvider::Microsoft => write!(f, "microsoft"),
            OAuthProvider::Apple => write!(f, "apple"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct OAuthLoginRequest {
    pub provider: OAuthProvider,
    #[validate(length(min = 1, max = 50), regex(path = "SUBDOMAIN_REGEX"))]
    pub tenant_subdomain: String,
    pub redirect_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthUrlResponse {
    pub auth_url: String,
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct OAuthCallbackRequest {
    pub provider: OAuthProvider,
    #[validate(length(min = 1))]
    pub code: String,
    #[validate(length(min = 1))]
    pub state: String,
    #[validate(length(min = 1, max = 50), regex(path = "SUBDOMAIN_REGEX"))]
    pub tenant_subdomain: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    pub id: String,
    pub email: String,
    pub name: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub picture: Option<String>,
    pub provider: OAuthProvider,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct InternalPersonOAuthRegisterRequest {
    pub provider: OAuthProvider,
    #[validate(length(min = 1))]
    pub code: String,
    #[validate(length(min = 1))]
    pub state: String,
    #[validate(length(min = 1, max = 50), regex(path = "SUBDOMAIN_REGEX"))]
    pub tenant_subdomain: String,
    #[validate(length(min = 1, max = 100))]
    pub tenant_name: String,
    // Optional internal person specific fields
    #[validate(length(max = 50))]
    pub department: Option<String>,
    #[validate(length(max = 100))]
    pub position: Option<String>,
    #[validate(length(max = 20))]
    pub employee_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: AuthUser,
    pub tenant: AuthTenant,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: PersonRole,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthTenant {
    pub id: Uuid,
    pub name: String,
    pub subdomain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User ID
    pub tenant_id: String,
    pub role: String,
    pub exp: usize, // Expiration time
    pub iat: usize, // Issued at
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RefreshTokenRequest {
    #[validate(length(min = 1))]
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LogoutRequest {
    #[validate(length(min = 1))]
    pub refresh_token: String,
}

lazy_static::lazy_static! {
    static ref SUBDOMAIN_REGEX: regex::Regex = regex::Regex::new(r"^[a-z0-9-]+$").unwrap();
}

// Person-only registration (without tenant creation)
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PersonOnlyRegisterRequest {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 1, max = 50))]
    pub first_name: String,

    #[validate(length(min = 1, max = 50))]
    pub last_name: String,

    #[validate(length(min = 8, max = 128))]
    pub password: String,
}

// Response for person-only registration (no tenant info)
#[derive(Debug, Serialize, Deserialize)]
pub struct PersonOnlyAuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub person: AuthPersonWithoutTenant,
}

// Person info without tenant context
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthPersonWithoutTenant {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}

// Request to associate user with existing tenant
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct JoinTenantRequest {
    #[validate(length(min = 1, max = 50), regex(path = "SUBDOMAIN_REGEX"))]
    pub tenant_subdomain: String,
}

// Request to create new tenant and associate user
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateAndJoinTenantRequest {
    #[validate(length(min = 1, max = 100))]
    pub tenant_name: String,

    #[validate(length(min = 1, max = 50), regex(path = "SUBDOMAIN_REGEX"))]
    pub tenant_subdomain: String,
}
