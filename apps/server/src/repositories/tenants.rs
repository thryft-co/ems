use anyhow::Result;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::models::{CreateTenantRequest, NewTenant, Tenant, UpdateTenantRequest};
use crate::schema::tenants;
use crate::repositories::DatabaseService;

pub struct TenantService {
    database: DatabaseService,
}

impl TenantService {
    pub fn new(database: DatabaseService) -> Self {
        Self { database }
    }

    pub async fn create_tenant(&self, request: CreateTenantRequest) -> Result<Tenant> {
        let mut conn = self.database.get_connection().await?;

        let new_tenant = NewTenant {
            name: request.name,
            subdomain: request.subdomain,
            database_url: None,
            settings: request.settings,
            is_active: Some(true),
        };

        let tenant: Tenant = diesel::insert_into(tenants::table)
            .values(&new_tenant)
            .returning(Tenant::as_returning())
            .get_result(&mut conn)
            .await?;

        Ok(tenant)
    }

    pub async fn get_tenant_by_id(&self, tenant_id: Uuid) -> Result<Option<Tenant>> {
        let mut conn = self.database.get_connection().await?;

        let tenant = tenants::table
            .filter(tenants::id.eq(tenant_id))
            .select(Tenant::as_select())
            .first::<Tenant>(&mut conn)
            .await
            .optional()?;

        Ok(tenant)
    }

    pub async fn get_tenant_by_subdomain(&self, subdomain: &str) -> Result<Option<Tenant>> {
        let mut conn = self.database.get_connection().await?;

        let tenant = tenants::table
            .filter(tenants::subdomain.eq(subdomain))
            .select(Tenant::as_select())
            .first::<Tenant>(&mut conn)
            .await
            .optional()?;

        Ok(tenant)
    }

    pub async fn update_tenant(
        &self,
        tenant_id: Uuid,
        request: UpdateTenantRequest,
    ) -> Result<Tenant> {
        let mut conn = self.database.get_connection().await?;

        // Update fields individually to avoid Diesel type issues
        if let Some(name) = &request.name {
            diesel::update(tenants::table.filter(tenants::id.eq(tenant_id)))
                .set(tenants::name.eq(name))
                .execute(&mut conn)
                .await?;
        }
        if let Some(settings) = &request.settings {
            diesel::update(tenants::table.filter(tenants::id.eq(tenant_id)))
                .set(tenants::settings.eq(settings))
                .execute(&mut conn)
                .await?;
        }
        if let Some(is_active) = request.is_active {
            diesel::update(tenants::table.filter(tenants::id.eq(tenant_id)))
                .set(tenants::is_active.eq(is_active))
                .execute(&mut conn)
                .await?;
        }

        // Return the updated tenant
        let tenant = tenants::table
            .filter(tenants::id.eq(tenant_id))
            .select(Tenant::as_select())
            .first::<Tenant>(&mut conn)
            .await?;

        Ok(tenant)
    }

    pub async fn delete_tenant(&self, tenant_id: Uuid) -> Result<()> {
        let mut conn = self.database.get_connection().await?;

        diesel::delete(tenants::table.filter(tenants::id.eq(tenant_id)))
            .execute(&mut conn)
            .await?;

        Ok(())
    }

    pub async fn list_tenants(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Tenant>> {
        let mut conn = self.database.get_connection().await?;

        let mut query = tenants::table.into_boxed();

        if let Some(limit_val) = limit {
            query = query.limit(limit_val as i64);
        }
        if let Some(offset_val) = offset {
            query = query.offset(offset_val as i64);
        }

        let tenants = query
            .select(Tenant::as_select())
            .load::<Tenant>(&mut conn)
            .await?;
        Ok(tenants)
    }
}
