use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::{Item, Person, Tenant};
use crate::schema::*;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = asset_types)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AssetType {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = asset_types)]
pub struct NewAssetType {
    pub name: String,
    pub description: Option<String>,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations,
)]
#[diesel(belongs_to(Tenant, foreign_key = tenant_id))]
#[diesel(belongs_to(Item, foreign_key = item_id))]
#[diesel(belongs_to(AssetType, foreign_key = asset_type_id))]
#[diesel(belongs_to(Person, foreign_key = created_by_id))]
#[diesel(table_name = assets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Asset {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub item_id: Uuid,
    pub asset_type_id: Uuid,
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub file_path: Option<String>,
    pub file_size: Option<i64>,
    pub file_type: Option<String>,
    pub checksum: Option<String>,
    pub is_active: Option<bool>,
    pub metadata: Option<serde_json::Value>,
    pub created_by_id: Uuid,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = assets)]
pub struct NewAsset {
    pub tenant_id: Uuid,
    pub item_id: Uuid,
    pub asset_type_id: Uuid,
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub file_path: Option<String>,
    pub file_size: Option<i64>,
    pub file_type: Option<String>,
    pub checksum: Option<String>,
    pub is_active: Option<bool>,
    pub metadata: Option<serde_json::Value>,
    pub created_by_id: Uuid,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations,
)]
#[diesel(belongs_to(Asset, foreign_key = asset_id))]
#[diesel(table_name = firmware_specific)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FirmwareSpecific {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub hardware_version: Option<String>,
    pub min_hardware_version: Option<String>,
    pub max_hardware_version: Option<String>,
    pub release_notes: Option<String>,
    pub is_beta: Option<bool>,
    pub is_critical: Option<bool>,
    pub requires_manual_update: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = firmware_specific)]
pub struct NewFirmwareSpecific {
    pub asset_id: Uuid,
    pub hardware_version: Option<String>,
    pub min_hardware_version: Option<String>,
    pub max_hardware_version: Option<String>,
    pub release_notes: Option<String>,
    pub is_beta: Option<bool>,
    pub is_critical: Option<bool>,
    pub requires_manual_update: Option<bool>,
}

// Asset Type Enum for common asset types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AssetTypeEnum {
    #[serde(rename = "invoice")]
    Invoice,
    #[serde(rename = "firmware")]
    Firmware,
    #[serde(rename = "report")]
    Report,
    #[serde(rename = "document")]
    Document,
    #[serde(rename = "certificate")]
    Certificate,
}

impl std::fmt::Display for AssetTypeEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetTypeEnum::Invoice => write!(f, "invoice"),
            AssetTypeEnum::Firmware => write!(f, "firmware"),
            AssetTypeEnum::Report => write!(f, "report"),
            AssetTypeEnum::Document => write!(f, "document"),
            AssetTypeEnum::Certificate => write!(f, "certificate"),
        }
    }
}

impl From<AssetTypeEnum> for String {
    fn from(asset_type: AssetTypeEnum) -> Self {
        asset_type.to_string()
    }
}

impl TryFrom<String> for AssetTypeEnum {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "invoice" => Ok(AssetTypeEnum::Invoice),
            "firmware" => Ok(AssetTypeEnum::Firmware),
            "report" => Ok(AssetTypeEnum::Report),
            "document" => Ok(AssetTypeEnum::Document),
            "certificate" => Ok(AssetTypeEnum::Certificate),
            _ => Err(format!("Invalid asset type: {}", value)),
        }
    }
}

// Request/Response DTOs
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateAssetRequest {
    pub item_id: Uuid,
    pub asset_type_id: Uuid,

    #[validate(length(min = 1, max = 100))]
    pub name: String,

    #[validate(length(max = 50))]
    pub version: Option<String>,

    #[validate(length(max = 1000))]
    pub description: Option<String>,

    #[validate(length(max = 500))]
    pub file_path: Option<String>,

    #[validate(range(min = 0))]
    pub file_size: Option<i64>,

    #[validate(length(max = 50))]
    pub file_type: Option<String>,

    #[validate(length(max = 64))]
    pub checksum: Option<String>,

    pub is_active: Option<bool>,
    pub metadata: Option<serde_json::Value>,

    // Firmware-specific fields (optional)
    pub firmware_details: Option<CreateFirmwareSpecificRequest>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateFirmwareSpecificRequest {
    #[validate(length(max = 50))]
    pub hardware_version: Option<String>,

    #[validate(length(max = 50))]
    pub min_hardware_version: Option<String>,

    #[validate(length(max = 50))]
    pub max_hardware_version: Option<String>,

    pub release_notes: Option<String>,
    pub is_beta: Option<bool>,
    pub is_critical: Option<bool>,
    pub requires_manual_update: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateAssetRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,

    #[validate(length(max = 50))]
    pub version: Option<String>,

    #[validate(length(max = 1000))]
    pub description: Option<String>,

    #[validate(length(max = 500))]
    pub file_path: Option<String>,

    #[validate(range(min = 0))]
    pub file_size: Option<i64>,

    #[validate(length(max = 50))]
    pub file_type: Option<String>,

    #[validate(length(max = 64))]
    pub checksum: Option<String>,

    pub is_active: Option<bool>,
    pub metadata: Option<serde_json::Value>,

    // Firmware-specific fields (optional)
    pub firmware_details: Option<UpdateFirmwareSpecificRequest>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateFirmwareSpecificRequest {
    #[validate(length(max = 50))]
    pub hardware_version: Option<String>,

    #[validate(length(max = 50))]
    pub min_hardware_version: Option<String>,

    #[validate(length(max = 50))]
    pub max_hardware_version: Option<String>,

    pub release_notes: Option<String>,
    pub is_beta: Option<bool>,
    pub is_critical: Option<bool>,
    pub requires_manual_update: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub item_id: Uuid,
    pub asset_type: AssetTypeResponse,
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub file_path: Option<String>,
    pub file_size: Option<i64>,
    pub file_type: Option<String>,
    pub checksum: Option<String>,
    pub is_active: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_by: PersonSummary,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Optional firmware-specific details
    pub firmware_details: Option<FirmwareSpecificResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetTypeResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FirmwareSpecificResponse {
    pub id: Uuid,
    pub hardware_version: Option<String>,
    pub min_hardware_version: Option<String>,
    pub max_hardware_version: Option<String>,
    pub release_notes: Option<String>,
    pub is_beta: bool,
    pub is_critical: bool,
    pub requires_manual_update: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PersonSummary {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetSummary {
    pub id: Uuid,
    pub name: String,
    pub version: Option<String>,
    pub asset_type: String,
    pub file_type: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAssetIdResponse {
    pub id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateAssetTypeRequest {
    #[validate(length(min = 1, max = 50))]
    pub name: String,

    #[validate(length(max = 1000))]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateAssetTypeRequest {
    #[validate(length(min = 1, max = 50))]
    pub name: Option<String>,

    #[validate(length(max = 1000))]
    pub description: Option<String>,
}
