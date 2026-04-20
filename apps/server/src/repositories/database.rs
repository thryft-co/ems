use anyhow::Result;
use diesel_async::pooled_connection::bb8::Pool as AsyncPool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncConnection, AsyncPgConnection, SimpleAsyncConnection};
use std::env;

pub type DbPool = AsyncPool<AsyncPgConnection>;
pub type DbConnection<'a> =
    bb8::PooledConnection<'a, AsyncDieselConnectionManager<AsyncPgConnection>>;

#[derive(Clone)]
pub struct DatabaseService {
    pub pool: DbPool,
}

impl DatabaseService {
    pub async fn new() -> Result<Self> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let config = AsyncPgConnection::establish(&database_url).await?;
        drop(config); // Test connection and drop it

        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&database_url);
        let pool = AsyncPool::builder().max_size(10).build(manager).await?;

        Ok(Self { pool })
    }

    pub async fn get_connection(&self) -> Result<DbConnection<'_>> {
        self.pool
            .get()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get connection: {}", e))
    }

    pub async fn execute_with_tenant_context<T, F>(&self, tenant_id: uuid::Uuid, f: F) -> Result<T>
    where
        F: FnOnce(DbConnection<'_>) -> Result<T>,
    {
        let mut conn = self.get_connection().await?;

        // Set tenant context for RLS (Row Level Security)
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        f(conn)
    }
}
