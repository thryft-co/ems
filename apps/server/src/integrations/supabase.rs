use anyhow::Result;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use postgrest::Postgrest;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

use crate::models::{OAuthProvider, OAuthUserInfo};

#[derive(Debug, Serialize, Deserialize)]
pub struct SupabaseOAuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: SupabaseOAuthUser,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SupabaseOAuthUser {
    pub id: String,
    pub email: String,
    pub user_metadata: serde_json::Value,
    pub identities: Vec<SupabaseIdentity>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SupabaseIdentity {
    pub provider: String,
    pub identity_data: serde_json::Value,
}

#[derive(Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_url: String,
    pub scopes: Vec<String>,
}

#[derive(Clone)]
pub struct SupabaseService {
    pub client: Postgrest,
    pub url: String,
    pub api_key: String,
    pub http_client: Client,
    pub google_oauth: Option<OAuthConfig>,
    pub microsoft_oauth: Option<OAuthConfig>,
    pub apple_oauth: Option<OAuthConfig>,
}

impl SupabaseService {
    pub async fn new(url: &str, api_key: &str) -> Result<Self> {
        let client = Postgrest::new(format!("{}/rest/v1", url)).insert_header("apikey", api_key);
        let http_client = Client::new();

        // Initialize OAuth configurations
        let google_oauth = Self::init_google_oauth()?;
        let microsoft_oauth = Self::init_microsoft_oauth()?;
        let apple_oauth = Self::init_apple_oauth()?;

        Ok(Self {
            client,
            url: url.to_string(),
            api_key: api_key.to_string(),
            http_client,
            google_oauth,
            microsoft_oauth,
            apple_oauth,
        })
    }

    pub fn get_client(&self) -> Postgrest {
        self.client.clone()
    }

    pub fn get_client_with_tenant(&self, tenant_id: &str) -> Postgrest {
        self.client.clone().insert_header("X-Tenant-ID", tenant_id)
    }

    /// Initialize Google OAuth configuration
    fn init_google_oauth() -> Result<Option<OAuthConfig>> {
        if let (Ok(client_id), Ok(client_secret), Ok(redirect_url)) = (
            env::var("GOOGLE_CLIENT_ID"),
            env::var("GOOGLE_CLIENT_SECRET"),
            env::var("GOOGLE_REDIRECT_URL"),
        ) {
            Ok(Some(OAuthConfig {
                client_id,
                client_secret,
                auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
                token_url: "https://oauth2.googleapis.com/token".to_string(),
                redirect_url,
                scopes: vec![
                    "openid".to_string(),
                    "email".to_string(),
                    "profile".to_string(),
                ],
            }))
        } else {
            tracing::warn!("Google OAuth configuration not found. Set GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET, and GOOGLE_REDIRECT_URL environment variables.");
            Ok(None)
        }
    }

    /// Initialize Microsoft OAuth configuration
    fn init_microsoft_oauth() -> Result<Option<OAuthConfig>> {
        if let (Ok(client_id), Ok(client_secret), Ok(redirect_url)) = (
            env::var("MICROSOFT_CLIENT_ID"),
            env::var("MICROSOFT_CLIENT_SECRET"),
            env::var("MICROSOFT_REDIRECT_URL"),
        ) {
            let tenant_id =
                env::var("MICROSOFT_TENANT_ID").unwrap_or_else(|_| "common".to_string());
            Ok(Some(OAuthConfig {
                client_id,
                client_secret,
                auth_url: format!(
                    "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize",
                    tenant_id
                ),
                token_url: format!(
                    "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
                    tenant_id
                ),
                redirect_url,
                scopes: vec![
                    "openid".to_string(),
                    "email".to_string(),
                    "profile".to_string(),
                    "User.Read".to_string(),
                ],
            }))
        } else {
            tracing::warn!("Microsoft OAuth configuration not found. Set MICROSOFT_CLIENT_ID, MICROSOFT_CLIENT_SECRET, and MICROSOFT_REDIRECT_URL environment variables.");
            Ok(None)
        }
    }

    /// Initialize Apple OAuth configuration
    fn init_apple_oauth() -> Result<Option<OAuthConfig>> {
        if let (Ok(client_id), Ok(client_secret), Ok(redirect_url)) = (
            env::var("APPLE_CLIENT_ID"),
            env::var("APPLE_CLIENT_SECRET"),
            env::var("APPLE_REDIRECT_URL"),
        ) {
            Ok(Some(OAuthConfig {
                client_id,
                client_secret,
                auth_url: "https://appleid.apple.com/auth/authorize".to_string(),
                token_url: "https://appleid.apple.com/auth/token".to_string(),
                redirect_url,
                scopes: vec![
                    "openid".to_string(),
                    "email".to_string(),
                    "name".to_string(),
                ],
            }))
        } else {
            tracing::warn!("Apple OAuth configuration not found. Set APPLE_CLIENT_ID, APPLE_CLIENT_SECRET, and APPLE_REDIRECT_URL environment variables.");
            Ok(None)
        }
    }

    /// Generate OAuth authorization URL
    pub fn generate_oauth_url(&self, provider: &OAuthProvider, state: &str) -> Result<String> {
        let config = match provider {
            OAuthProvider::Google => self.google_oauth.as_ref(),
            OAuthProvider::Microsoft => self.microsoft_oauth.as_ref(),
            OAuthProvider::Apple => self.apple_oauth.as_ref(),
        };

        let config = config
            .ok_or_else(|| anyhow::anyhow!("{} OAuth is not configured", provider.to_string()))?;

        let client = oauth2::basic::BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            AuthUrl::new(config.auth_url.clone())?,
            Some(TokenUrl::new(config.token_url.clone())?),
        )
        .set_redirect_uri(RedirectUrl::new(config.redirect_url.clone())?);

        let mut auth_request = client.authorize_url(|| CsrfToken::new(state.to_string()));

        // Add scopes
        for scope in &config.scopes {
            auth_request = auth_request.add_scope(Scope::new(scope.clone()));
        }

        // Add provider-specific parameters
        match provider {
            OAuthProvider::Google => {
                auth_request = auth_request.add_extra_param("access_type", "offline");
            }
            OAuthProvider::Microsoft => {
                auth_request = auth_request.add_extra_param("response_mode", "query");
            }
            OAuthProvider::Apple => {
                auth_request = auth_request.add_extra_param("response_mode", "form_post");
            }
        }

        let (auth_url, _csrf_token) = auth_request.url();
        Ok(auth_url.to_string())
    }

    /// Exchange authorization code for access token and get user info
    pub async fn handle_oauth_callback(
        &self,
        provider: &OAuthProvider,
        code: &str,
    ) -> Result<OAuthUserInfo> {
        let config = match provider {
            OAuthProvider::Google => self.google_oauth.as_ref(),
            OAuthProvider::Microsoft => self.microsoft_oauth.as_ref(),
            OAuthProvider::Apple => self.apple_oauth.as_ref(),
        };

        let config = config
            .ok_or_else(|| anyhow::anyhow!("{} OAuth is not configured", provider.to_string()))?;

        let client = oauth2::basic::BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            AuthUrl::new(config.auth_url.clone())?,
            Some(TokenUrl::new(config.token_url.clone())?),
        )
        .set_redirect_uri(RedirectUrl::new(config.redirect_url.clone())?);

        // Exchange code for token
        let token_result = client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to exchange code for token: {:?}", e))?;

        let access_token = token_result.access_token().secret();

        // Get user info based on provider
        let user_info = match provider {
            OAuthProvider::Google => self.get_google_user_info(access_token).await?,
            OAuthProvider::Microsoft => self.get_microsoft_user_info(access_token).await?,
            OAuthProvider::Apple => self.get_apple_user_info(access_token).await?,
        };

        Ok(user_info)
    }

    /// Get Google user information
    async fn get_google_user_info(&self, access_token: &str) -> Result<OAuthUserInfo> {
        let response = self
            .http_client
            .get("https://www.googleapis.com/oauth2/v2/userinfo")
            .bearer_auth(access_token)
            .send()
            .await?;

        if response.status().is_success() {
            let user_data: serde_json::Value = response.json().await?;

            Ok(OAuthUserInfo {
                id: user_data["id"].as_str().unwrap_or_default().to_string(),
                email: user_data["email"].as_str().unwrap_or_default().to_string(),
                name: user_data["name"].as_str().unwrap_or_default().to_string(),
                first_name: user_data["given_name"].as_str().map(|s| s.to_string()),
                last_name: user_data["family_name"].as_str().map(|s| s.to_string()),
                picture: user_data["picture"].as_str().map(|s| s.to_string()),
                provider: OAuthProvider::Google,
            })
        } else {
            Err(anyhow::anyhow!(
                "Failed to get Google user info: {}",
                response.status()
            ))
        }
    }

    /// Get Microsoft user information
    async fn get_microsoft_user_info(&self, access_token: &str) -> Result<OAuthUserInfo> {
        let response = self
            .http_client
            .get("https://graph.microsoft.com/v1.0/me")
            .bearer_auth(access_token)
            .send()
            .await?;

        if response.status().is_success() {
            let user_data: serde_json::Value = response.json().await?;

            Ok(OAuthUserInfo {
                id: user_data["id"].as_str().unwrap_or_default().to_string(),
                email: user_data["mail"]
                    .as_str()
                    .or_else(|| user_data["userPrincipalName"].as_str())
                    .unwrap_or_default()
                    .to_string(),
                name: user_data["displayName"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                first_name: user_data["givenName"].as_str().map(|s| s.to_string()),
                last_name: user_data["surname"].as_str().map(|s| s.to_string()),
                picture: None, // Microsoft Graph requires separate call for photo
                provider: OAuthProvider::Microsoft,
            })
        } else {
            Err(anyhow::anyhow!(
                "Failed to get Microsoft user info: {}",
                response.status()
            ))
        }
    }

    /// Get Apple user information
    async fn get_apple_user_info(&self, _access_token: &str) -> Result<OAuthUserInfo> {
        // TODO: Implement Apple ID token decoding
        // Apple returns user info in the ID token, not via a separate API call
        // For now, return a placeholder
        Err(anyhow::anyhow!(
            "Apple user info extraction not yet implemented"
        ))
    }

    /// Authenticate user with Supabase using OAuth
    pub async fn oauth_sign_in(
        &self,
        provider: &OAuthProvider,
        access_token: &str,
    ) -> Result<SupabaseOAuthResponse> {
        let auth_url = format!("{}/auth/v1/token?grant_type=id_token", self.url);

        let response = self
            .http_client
            .post(&auth_url)
            .header("apikey", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "id_token": access_token,
                "provider": provider.to_string()
            }))
            .send()
            .await?;

        if response.status().is_success() {
            let oauth_response: SupabaseOAuthResponse = response.json().await?;
            Ok(oauth_response)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(anyhow::anyhow!(
                "Supabase OAuth sign-in failed: {}",
                error_text
            ))
        }
    }

    /// Create user in Supabase using admin API
    pub async fn create_user(
        &self,
        email: &str,
        password: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        // Use service role key for admin operations
        let service_role_key = std::env::var("SUPABASE_SERVICE_ROLE_KEY")
            .map_err(|_| anyhow::anyhow!("SUPABASE_SERVICE_ROLE_KEY not set"))?;

        let admin_url = format!("{}/auth/v1/admin/users", self.url);

        let mut payload = serde_json::json!({
            "email": email,
            "password": password,
            "email_confirm": true
        });

        if let Some(meta) = metadata {
            payload["user_metadata"] = meta;
        }

        let response = self
            .http_client
            .post(&admin_url)
            .header("apikey", &service_role_key)
            .header("Authorization", format!("Bearer {}", service_role_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let user: serde_json::Value = response.json().await?;
            Ok(user)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(anyhow::anyhow!("User creation failed: {}", error_text))
        }
    }

    /// Authenticate user with Supabase
    pub async fn authenticate_user(
        &self,
        email: &str,
        password: &str,
    ) -> Result<serde_json::Value> {
        let auth_url = format!("{}/auth/v1/token?grant_type=password", self.url);

        let response = self
            .http_client
            .post(&auth_url)
            .header("apikey", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "email": email,
                "password": password
            }))
            .send()
            .await?;

        if response.status().is_success() {
            let session: serde_json::Value = response.json().await?;
            Ok(session)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(anyhow::anyhow!("Authentication failed: {}", error_text))
        }
    }

    /// Verify if user exists in Supabase
    pub async fn verify_user_exists(&self, email: &str) -> Result<bool> {
        // Use service role key for admin operations
        let service_role_key = std::env::var("SUPABASE_SERVICE_ROLE_KEY")
            .map_err(|_| anyhow::anyhow!("SUPABASE_SERVICE_ROLE_KEY not set"))?;

        let admin_url = format!("{}/auth/v1/admin/users", self.url);

        let response = self
            .http_client
            .get(&admin_url)
            .header("apikey", &service_role_key)
            .header("Authorization", format!("Bearer {}", service_role_key))
            .header("Content-Type", "application/json")
            .query(&[("email", email)])
            .send()
            .await?;

        if response.status().is_success() {
            let users: serde_json::Value = response.json().await?;

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
}
