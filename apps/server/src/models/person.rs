use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::tenant::Tenant;
use crate::schema::*;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = person)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Person {
    pub id: Uuid,
    pub supabase_uid: Uuid,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub global_access: Option<Vec<Option<String>>>,
    pub is_active: Option<bool>,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = person)]
pub struct NewPerson {
    pub supabase_uid: Uuid,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub global_access: Option<Vec<Option<String>>>,
    pub is_active: Option<bool>,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations,
)]
#[diesel(belongs_to(Person, foreign_key = person_id))]
#[diesel(belongs_to(Tenant, foreign_key = tenant_id))]
#[diesel(table_name = tenant_person)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TenantPerson {
    pub id: Uuid,
    pub person_id: Uuid,
    pub tenant_id: Uuid,
    pub role: String, // We'll convert from PersonRole enum
    pub access_level: Option<Vec<Option<String>>>,
    pub is_primary: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = tenant_person)]
pub struct NewTenantPerson {
    pub person_id: Uuid,
    pub tenant_id: Uuid,
    pub role: String,
    pub access_level: Option<Vec<Option<String>>>,
    pub is_primary: Option<bool>,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations,
)]
#[diesel(belongs_to(Person, foreign_key = person_id))]
#[diesel(belongs_to(Tenant, foreign_key = tenant_id))]
#[diesel(table_name = internal_person)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InternalPerson {
    pub id: Uuid,
    pub person_id: Uuid,
    pub tenant_id: Uuid,
    pub department: Option<String>,
    pub position: Option<String>,
    pub employee_id: Option<String>,
    pub hire_date: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = internal_person)]
pub struct NewInternalPerson {
    pub person_id: Uuid,
    pub tenant_id: Uuid,
    pub department: Option<String>,
    pub position: Option<String>,
    pub employee_id: Option<String>,
    pub hire_date: Option<DateTime<Utc>>,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations,
)]
#[diesel(belongs_to(Person, foreign_key = person_id))]
#[diesel(belongs_to(Tenant, foreign_key = tenant_id))]
#[diesel(table_name = customer_person)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CustomerPerson {
    pub id: Uuid,
    pub person_id: Uuid,
    pub tenant_id: Uuid,
    pub company: Option<String>,
    pub industry: Option<String>,
    pub customer_since: Option<DateTime<Utc>>,
    pub account_manager_id: Option<Uuid>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = customer_person)]
pub struct NewCustomerPerson {
    pub person_id: Uuid,
    pub tenant_id: Uuid,
    pub company: Option<String>,
    pub industry: Option<String>,
    pub customer_since: Option<DateTime<Utc>>,
    pub account_manager_id: Option<Uuid>,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations,
)]
#[diesel(belongs_to(Person, foreign_key = person_id))]
#[diesel(belongs_to(Tenant, foreign_key = tenant_id))]
#[diesel(table_name = vendor_person)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct VendorPerson {
    pub id: Uuid,
    pub person_id: Uuid,
    pub tenant_id: Uuid,
    pub company: Option<String>,
    pub service_type: Option<String>,
    pub contract_start: Option<DateTime<Utc>>,
    pub contract_end: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = vendor_person)]
pub struct NewVendorPerson {
    pub person_id: Uuid,
    pub tenant_id: Uuid,
    pub company: Option<String>,
    pub service_type: Option<String>,
    pub contract_start: Option<DateTime<Utc>>,
    pub contract_end: Option<DateTime<Utc>>,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations,
)]
#[diesel(belongs_to(Person, foreign_key = person_id))]
#[diesel(belongs_to(Tenant, foreign_key = tenant_id))]
#[diesel(table_name = distributor_person)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DistributorPerson {
    pub id: Uuid,
    pub person_id: Uuid,
    pub tenant_id: Uuid,
    pub company: Option<String>,
    pub territory: Option<String>,
    pub distribution_tier: Option<String>,
    pub commission_rate: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = distributor_person)]
pub struct NewDistributorPerson {
    pub person_id: Uuid,
    pub tenant_id: Uuid,
    pub company: Option<String>,
    pub territory: Option<String>,
    pub distribution_tier: Option<String>,
    pub commission_rate: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PersonRole {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "internal")]
    Internal,
    #[serde(rename = "customer")]
    Customer,
    #[serde(rename = "vendor")]
    Vendor,
    #[serde(rename = "distributor")]
    Distributor,
}

impl std::fmt::Display for PersonRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersonRole::Pending => write!(f, "pending"),
            PersonRole::Internal => write!(f, "internal"),
            PersonRole::Customer => write!(f, "customer"),
            PersonRole::Vendor => write!(f, "vendor"),
            PersonRole::Distributor => write!(f, "distributor"),
        }
    }
}

impl From<PersonRole> for String {
    fn from(role: PersonRole) -> Self {
        role.to_string()
    }
}

impl TryFrom<String> for PersonRole {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "pending" => Ok(PersonRole::Pending),
            "internal" => Ok(PersonRole::Internal),
            "customer" => Ok(PersonRole::Customer),
            "vendor" => Ok(PersonRole::Vendor),
            "distributor" => Ok(PersonRole::Distributor),
            _ => Err(format!("Invalid person role: {}", value)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreatePersonRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    #[validate(email)]
    pub email: String,

    #[validate(length(max = 20))]
    pub phone: Option<String>,

    pub role: PersonRole,

    pub person_type: PersonRole, // For compatibility with Postman collection

    // Internal person specific fields
    #[validate(length(max = 50))]
    pub department: Option<String>,

    #[validate(length(max = 100))]
    pub position: Option<String>,

    #[validate(length(max = 20))]
    pub employee_id: Option<String>,

    pub hire_date: Option<DateTime<Utc>>,

    // Customer person specific fields
    #[validate(length(max = 100))]
    pub company: Option<String>,

    #[validate(length(max = 50))]
    pub industry: Option<String>,

    pub customer_since: Option<DateTime<Utc>>,

    pub account_manager_id: Option<Uuid>,

    // Vendor person specific fields
    #[validate(length(max = 100))]
    pub service_type: Option<String>,

    pub contract_start: Option<DateTime<Utc>>,

    pub contract_end: Option<DateTime<Utc>>,

    // Distributor person specific fields
    #[validate(length(max = 100))]
    pub territory: Option<String>,

    #[validate(length(max = 50))]
    pub distribution_tier: Option<String>,

    #[validate(length(max = 10))]
    pub commission_rate: Option<String>,

    pub global_access: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdatePersonRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,

    #[validate(length(max = 20))]
    pub phone: Option<String>,

    pub role: Option<PersonRole>,

    pub global_access: Option<Vec<String>>,

    pub is_active: Option<bool>,

    // Internal person specific fields
    #[validate(length(max = 50))]
    pub department: Option<String>,

    #[validate(length(max = 100))]
    pub position: Option<String>,

    #[validate(length(max = 20))]
    pub employee_id: Option<String>,

    pub hire_date: Option<DateTime<Utc>>,

    // Customer person specific fields
    #[validate(length(max = 100))]
    pub company: Option<String>,

    #[validate(length(max = 50))]
    pub industry: Option<String>,

    pub customer_since: Option<DateTime<Utc>>,

    pub account_manager_id: Option<Uuid>,

    // Vendor person specific fields
    #[validate(length(max = 100))]
    pub service_type: Option<String>,

    pub contract_start: Option<DateTime<Utc>>,

    pub contract_end: Option<DateTime<Utc>>,

    // Distributor person specific fields
    #[validate(length(max = 100))]
    pub territory: Option<String>,

    #[validate(length(max = 50))]
    pub distribution_tier: Option<String>,

    #[validate(length(max = 10))]
    pub commission_rate: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PersonResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub person_type: PersonRole,
    pub global_access: Vec<String>,
    pub is_active: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Type-specific data
    pub internal: Option<InternalPersonData>,
    pub customer: Option<CustomerPersonData>,
    pub vendor: Option<VendorPersonData>,
    pub distributor: Option<DistributorPersonData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalPersonResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub department: Option<String>,
    pub position: Option<String>,
    pub employee_id: Option<String>,
    pub hire_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerPersonResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub industry: Option<String>,
    pub customer_since: Option<DateTime<Utc>>,
    pub account_manager_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VendorPersonResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub service_type: Option<String>,
    pub contract_start: Option<DateTime<Utc>>,
    pub contract_end: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DistributorPersonResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub territory: Option<String>,
    pub distribution_tier: Option<String>,
    pub commission_rate: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalPersonData {
    pub department: Option<String>,
    pub position: Option<String>,
    pub employee_id: Option<String>,
    pub hire_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerPersonData {
    pub company: Option<String>,
    pub industry: Option<String>,
    pub customer_since: Option<DateTime<Utc>>,
    pub account_manager_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VendorPersonData {
    pub company: Option<String>,
    pub service_type: Option<String>,
    pub contract_start: Option<DateTime<Utc>>,
    pub contract_end: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DistributorPersonData {
    pub company: Option<String>,
    pub territory: Option<String>,
    pub distribution_tier: Option<String>,
    pub commission_rate: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePersonIdResponse {
    pub id: Uuid,
}
