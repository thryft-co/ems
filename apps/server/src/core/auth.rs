use anyhow::Result;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::env;
use uuid::Uuid;

use crate::models::{AuthTenant, AuthUser, Claims, PersonRole};

#[derive(Debug, Serialize, Deserialize)]
pub struct SupabaseUser {
    pub id: String,
    pub email: String,
    pub user_metadata: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SupabaseSession {
    pub access_token: String,
    pub refresh_token: String,
    pub user: SupabaseUser,
}

pub struct AuthUtils;

impl AuthUtils {
    /// Hash a password using bcrypt
    pub fn hash_password(password: &str) -> Result<String> {
        hash(password, DEFAULT_COST).map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))
    }

    /// Verify a password against a hash
    pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
        verify(password, hash).map_err(|e| anyhow::anyhow!("Failed to verify password: {}", e))
    }

    /// Hash a token for storage in blacklist (using SHA256)
    pub fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Generate JWT access token
    pub fn generate_access_token(
        user_id: Uuid,
        tenant_id: Uuid,
        role: &PersonRole,
    ) -> Result<String> {
        let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret".to_string());
        let now = Utc::now();
        let exp = now + Duration::hours(1); // 1 hour expiration

        let claims = Claims {
            sub: user_id.to_string(),
            tenant_id: tenant_id.to_string(),
            role: role.to_string(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .map_err(|e| anyhow::anyhow!("Failed to generate access token: {}", e))
    }

    /// Generate JWT refresh token
    pub fn generate_refresh_token(user_id: Uuid, tenant_id: Uuid) -> Result<String> {
        let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret".to_string());
        let now = Utc::now();
        let exp = now + Duration::days(30); // 30 days expiration

        let claims = Claims {
            sub: user_id.to_string(),
            tenant_id: tenant_id.to_string(),
            role: "refresh".to_string(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .map_err(|e| anyhow::anyhow!("Failed to generate refresh token: {}", e))
    }

    /// Generate temporary JWT access token without tenant context (for user-only registration)
    pub fn generate_temporary_access_token(user_id: Uuid) -> Result<String> {
        let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret".to_string());
        let now = Utc::now();
        let exp = now + Duration::hours(1); // 1 hour expiration

        let claims = Claims {
            sub: user_id.to_string(),
            tenant_id: "".to_string(),   // Empty tenant_id for temp tokens
            role: "pending".to_string(), // Special role for users without tenants
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .map_err(|e| anyhow::anyhow!("Failed to generate temporary access token: {}", e))
    }

    /// Generate temporary JWT refresh token without tenant context
    pub fn generate_temporary_refresh_token(user_id: Uuid) -> Result<String> {
        let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret".to_string());
        let now = Utc::now();
        let exp = now + Duration::days(7); // Shorter expiration for temp tokens (7 days)

        let claims = Claims {
            sub: user_id.to_string(),
            tenant_id: "".to_string(),   // Empty tenant_id for temp tokens
            role: "pending".to_string(), // Special role for users without tenants
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .map_err(|e| anyhow::anyhow!("Failed to generate temporary refresh token: {}", e))
    }

    /// Validate and decode JWT token
    pub fn validate_token(token: &str) -> Result<TokenData<Claims>> {
        let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret".to_string());

        decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|e| anyhow::anyhow!("Failed to validate token: {}", e))
    }

    /// Verify JWT token (alias for validate_token for compatibility)
    pub fn verify_jwt_token(token: &str) -> Result<Claims> {
        Self::validate_token(token).map(|token_data| token_data.claims)
    }

    /// Create AuthUser from person data
    pub fn create_auth_user(id: Uuid, email: String, name: String, role: PersonRole) -> AuthUser {
        // Split name into first and last name (basic implementation)
        let name_parts: Vec<&str> = name.split_whitespace().collect();
        let (first_name, last_name) = if name_parts.len() >= 2 {
            (name_parts[0].to_string(), name_parts[1..].join(" "))
        } else {
            (name.clone(), "".to_string())
        };

        AuthUser {
            id,
            email,
            first_name,
            last_name,
            role,
        }
    }

    /// Create AuthTenant from tenant data
    pub fn create_auth_tenant(id: Uuid, name: String, subdomain: String) -> AuthTenant {
        AuthTenant {
            id,
            name,
            subdomain,
        }
    }

    /// Authenticate with Supabase using direct HTTP requests
    pub async fn authenticate_with_supabase(
        email: &str,
        password: &str,
    ) -> Result<SupabaseSession> {
        let supabase_url =
            env::var("SUPABASE_URL").map_err(|_| anyhow::anyhow!("SUPABASE_URL not set"))?;
        let supabase_anon_key = env::var("SUPABASE_ANON_KEY")
            .map_err(|_| anyhow::anyhow!("SUPABASE_ANON_KEY not set"))?;

        let client = reqwest::Client::new();
        let auth_url = format!("{}/auth/v1/token?grant_type=password", supabase_url);

        let response = client
            .post(&auth_url)
            .header("apikey", &supabase_anon_key)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "email": email,
                "password": password
            }))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send auth request: {}", e))?;

        if response.status().is_success() {
            let session: SupabaseSession = response
                .json()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to parse auth response: {}", e))?;
            Ok(session)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(anyhow::anyhow!("Authentication failed: {}", error_text))
        }
    }

    /// Register with Supabase using direct HTTP requests
    pub async fn register_with_supabase(
        email: &str,
        password: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<SupabaseSession> {
        let supabase_url =
            env::var("SUPABASE_URL").map_err(|_| anyhow::anyhow!("SUPABASE_URL not set"))?;
        let supabase_anon_key = env::var("SUPABASE_ANON_KEY")
            .map_err(|_| anyhow::anyhow!("SUPABASE_ANON_KEY not set"))?;

        let client = reqwest::Client::new();
        let signup_url = format!("{}/auth/v1/signup", supabase_url);

        let mut payload = serde_json::json!({
            "email": email,
            "password": password
        });

        if let Some(meta) = metadata {
            payload["data"] = meta;
        }

        let response = client
            .post(&signup_url)
            .header("apikey", &supabase_anon_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send signup request: {}", e))?;

        if response.status().is_success() {
            let session: SupabaseSession = response
                .json()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to parse signup response: {}", e))?;
            Ok(session)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(anyhow::anyhow!("Registration failed: {}", error_text))
        }
    }

    /// Verify Supabase user exists using direct HTTP requests
    pub async fn verify_supabase_user(email: &str) -> Result<bool> {
        let supabase_url =
            env::var("SUPABASE_URL").map_err(|_| anyhow::anyhow!("SUPABASE_URL not set"))?;
        let supabase_service_key = env::var("SUPABASE_SERVICE_ROLE_KEY")
            .map_err(|_| anyhow::anyhow!("SUPABASE_SERVICE_ROLE_KEY not set"))?;

        let client = reqwest::Client::new();
        let admin_url = format!("{}/auth/v1/admin/users", supabase_url);

        let response = client
            .get(&admin_url)
            .header("apikey", &supabase_service_key)
            .header("Authorization", format!("Bearer {}", supabase_service_key))
            .header("Content-Type", "application/json")
            .query(&[("email", email)])
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to verify user: {}", e))?;

        if response.status().is_success() {
            let users: serde_json::Value = response
                .json()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to parse user response: {}", e))?;

            // Check if any users were returned
            if let Some(users_array) = users["users"].as_array() {
                Ok(!users_array.is_empty())
            } else {
                Ok(false)
            }
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(anyhow::anyhow!("Failed to verify user: {}", error_text))
        }
    }

    /// Create Supabase user using admin API (for cases where we need to create without email confirmation)
    pub async fn create_supabase_user(
        email: &str,
        password: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<SupabaseUser> {
        let supabase_url =
            env::var("SUPABASE_URL").map_err(|_| anyhow::anyhow!("SUPABASE_URL not set"))?;
        let supabase_service_key = env::var("SUPABASE_SERVICE_ROLE_KEY")
            .map_err(|_| anyhow::anyhow!("SUPABASE_SERVICE_ROLE_KEY not set"))?;

        let client = reqwest::Client::new();
        let admin_url = format!("{}/auth/v1/admin/users", supabase_url);

        let mut payload = serde_json::json!({
            "email": email,
            "password": password,
            "email_confirm": true
        });

        if let Some(meta) = metadata {
            payload["user_metadata"] = meta;
        }

        let response = client
            .post(&admin_url)
            .header("apikey", &supabase_service_key)
            .header("Authorization", format!("Bearer {}", supabase_service_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create user: {}", e))?;

        if response.status().is_success() {
            let user: SupabaseUser = response
                .json()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to parse user creation response: {}", e))?;
            Ok(user)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(anyhow::anyhow!("User creation failed: {}", error_text))
        }
    }
}
