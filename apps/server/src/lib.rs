pub mod core;
pub mod integrations;
pub mod middleware;
pub mod models;
pub mod repositories;
pub mod routes;
pub mod schema;
pub mod services;

use anyhow::Result;
use integrations::SupabaseService;
use repositories::DatabaseService;
use std::env;

#[derive(Clone)]
pub struct AppState {
    pub database: DatabaseService,
    pub supabase: SupabaseService,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        let database = DatabaseService::new().await?;

        // Initialize Supabase service
        let supabase_url = env::var("SUPABASE_URL")
            .map_err(|_| anyhow::anyhow!("SUPABASE_URL environment variable is required"))?;
        let supabase_key = env::var("SUPABASE_ANON_KEY")
            .map_err(|_| anyhow::anyhow!("SUPABASE_ANON_KEY environment variable is required"))?;

        let supabase = SupabaseService::new(&supabase_url, &supabase_key).await?;

        Ok(Self { database, supabase })
    }
}
