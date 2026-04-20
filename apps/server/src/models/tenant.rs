use chrono::{DateTime, Utc};
use diesel::prelude::*;
use regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::schema::tenants;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = tenants)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub subdomain: String,
    pub database_url: Option<String>,
    pub settings: Option<serde_json::Value>,
    pub is_active: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = tenants)]
pub struct NewTenant {
    pub name: String,
    pub subdomain: String,
    pub database_url: Option<String>,
    pub settings: Option<serde_json::Value>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateTenantRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    #[validate(length(min = 1, max = 50), regex(path = "SUBDOMAIN_REGEX"))]
    pub subdomain: String,

    pub settings: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateTenantRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,

    pub settings: Option<serde_json::Value>,

    pub is_active: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct TenantContext {
    pub tenant_id: Uuid,
    pub tenant: Tenant,
}

lazy_static::lazy_static! {
    static ref SUBDOMAIN_REGEX: regex::Regex = regex::Regex::new(r"^[a-z0-9-]+$").unwrap();
}
