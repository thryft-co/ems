use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::token_blacklist;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = token_blacklist)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TokenBlacklist {
    pub id: Uuid,
    pub token_hash: String,
    pub token_type: String,
    pub person_id: Uuid,
    pub tenant_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub blacklisted_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = token_blacklist)]
pub struct NewTokenBlacklist {
    pub token_hash: String,
    pub token_type: String,
    pub person_id: Uuid,
    pub tenant_id: Uuid,
    pub expires_at: DateTime<Utc>,
}
