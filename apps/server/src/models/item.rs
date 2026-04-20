use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::tenant::Tenant;
use crate::schema::*;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = items)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Item {
    pub id: Uuid,
    pub internal_part_number: String,
    pub mfr_part_number: Option<String>,
    pub manufacturer: String,
    pub datasheet: Option<String>,
    pub lifecycle: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub linked_resources: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = items)]
pub struct NewItem {
    pub internal_part_number: String,
    pub mfr_part_number: Option<String>,
    pub manufacturer: String,
    pub datasheet: Option<String>,
    pub lifecycle: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub linked_resources: Option<serde_json::Value>,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations,
)]
#[diesel(belongs_to(Item, foreign_key = item_id))]
#[diesel(belongs_to(Tenant, foreign_key = tenant_id))]
#[diesel(table_name = inventory_items)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InventoryItem {
    pub id: Uuid,
    pub item_id: Uuid,
    pub tenant_id: Uuid,
    pub context: String,
    pub quantity: Option<i32>,
    pub location: Option<String>,
    pub pricing: Option<serde_json::Value>,
    pub lead_time: Option<i32>,
    pub min_stock_level: Option<i32>,
    pub max_stock_level: Option<i32>,
    pub reorder_point: Option<i32>,
    pub vendor_id: Option<Uuid>,
    pub last_received_date: Option<DateTime<Utc>>,
    pub status: Option<String>,
    pub notes: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = inventory_items)]
pub struct NewInventoryItem {
    pub item_id: Uuid,
    pub tenant_id: Uuid,
    pub context: String,
    pub quantity: Option<i32>,
    pub location: Option<String>,
    pub pricing: Option<serde_json::Value>,
    pub lead_time: Option<i32>,
    pub min_stock_level: Option<i32>,
    pub max_stock_level: Option<i32>,
    pub reorder_point: Option<i32>,
    pub vendor_id: Option<Uuid>,
    pub last_received_date: Option<DateTime<Utc>>,
    pub status: Option<String>,
    pub notes: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations,
)]
#[diesel(belongs_to(Tenant, foreign_key = tenant_id))]
#[diesel(table_name = item_bom)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ItemBom {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub parent_item_id: Uuid,
    pub component_item_id: Uuid,
    pub quantity: Option<i32>,
    pub notes: Option<String>,
    pub is_optional: Option<bool>,
    pub substitutes: Option<Vec<Option<Uuid>>>,
    pub assembly_order: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = item_bom)]
pub struct NewItemBom {
    pub tenant_id: Uuid,
    pub parent_item_id: Uuid,
    pub component_item_id: Uuid,
    pub quantity: Option<i32>,
    pub notes: Option<String>,
    pub is_optional: Option<bool>,
    pub substitutes: Option<Vec<Option<Uuid>>>,
    pub assembly_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ItemContext {
    #[serde(rename = "finished_goods")]
    FinishedGoods,
    #[serde(rename = "store")]
    Store,
    #[serde(rename = "vendor")]
    Vendor,
}

impl std::fmt::Display for ItemContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemContext::FinishedGoods => write!(f, "finished_goods"),
            ItemContext::Store => write!(f, "store"),
            ItemContext::Vendor => write!(f, "vendor"),
        }
    }
}

impl From<ItemContext> for String {
    fn from(context: ItemContext) -> Self {
        context.to_string()
    }
}

impl TryFrom<String> for ItemContext {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "finished_goods" => Ok(ItemContext::FinishedGoods),
            "store" => Ok(ItemContext::Store),
            "vendor" => Ok(ItemContext::Vendor),
            _ => Err(format!("Invalid item context: {}", value)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ItemLifecycle {
    #[serde(rename = "production")]
    Production,
    #[serde(rename = "prototype")]
    Prototype,
    #[serde(rename = "obsolete")]
    Obsolete,
    #[serde(rename = "nrfnd")]
    Nrfnd,
}

impl std::fmt::Display for ItemLifecycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemLifecycle::Production => write!(f, "production"),
            ItemLifecycle::Prototype => write!(f, "prototype"),
            ItemLifecycle::Obsolete => write!(f, "obsolete"),
            ItemLifecycle::Nrfnd => write!(f, "nrfnd"),
        }
    }
}

impl From<ItemLifecycle> for String {
    fn from(lifecycle: ItemLifecycle) -> Self {
        lifecycle.to_string()
    }
}

impl TryFrom<String> for ItemLifecycle {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "production" => Ok(ItemLifecycle::Production),
            "prototype" => Ok(ItemLifecycle::Prototype),
            "obsolete" => Ok(ItemLifecycle::Obsolete),
            "nrfnd" => Ok(ItemLifecycle::Nrfnd),
            _ => Err(format!("Invalid item lifecycle: {}", value)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ItemStatus {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "inactive")]
    Inactive,
    #[serde(rename = "discontinued")]
    Discontinued,
}

impl std::fmt::Display for ItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemStatus::Active => write!(f, "active"),
            ItemStatus::Inactive => write!(f, "inactive"),
            ItemStatus::Discontinued => write!(f, "discontinued"),
        }
    }
}

impl From<ItemStatus> for String {
    fn from(status: ItemStatus) -> Self {
        status.to_string()
    }
}

impl TryFrom<String> for ItemStatus {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "active" => Ok(ItemStatus::Active),
            "inactive" => Ok(ItemStatus::Inactive),
            "discontinued" => Ok(ItemStatus::Discontinued),
            _ => Err(format!("Invalid item status: {}", value)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateItemRequest {
    #[validate(length(min = 1, max = 50))]
    pub internal_part_number: String,

    #[validate(length(max = 50))]
    pub mfr_part_number: Option<String>,

    #[validate(length(min = 1, max = 100))]
    pub manufacturer: String,

    #[validate(length(max = 500))]
    pub datasheet: Option<String>,

    pub lifecycle: Option<ItemLifecycle>,

    pub description: Option<String>,

    #[validate(length(max = 50))]
    pub category: Option<String>,

    pub metadata: Option<serde_json::Value>,

    pub linked_resources: Option<serde_json::Value>,

    // Inventory context data
    pub context: ItemContext,

    #[validate(range(min = 0))]
    pub quantity: Option<i32>,

    #[validate(length(max = 100))]
    pub location: Option<String>,

    pub pricing: Option<serde_json::Value>,

    #[validate(range(min = 0))]
    pub lead_time: Option<i32>,

    #[validate(range(min = 0))]
    pub min_stock_level: Option<i32>,

    #[validate(range(min = 0))]
    pub max_stock_level: Option<i32>,

    #[validate(range(min = 0))]
    pub reorder_point: Option<i32>,

    pub vendor_id: Option<Uuid>,

    pub last_received_date: Option<DateTime<Utc>>,

    pub status: Option<ItemStatus>,

    pub notes: Option<String>,

    pub inventory_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateItemRequest {
    #[validate(length(max = 50))]
    pub mfr_part_number: Option<String>,

    #[validate(length(min = 1, max = 100))]
    pub manufacturer: Option<String>,

    #[validate(length(max = 500))]
    pub datasheet: Option<String>,

    pub lifecycle: Option<ItemLifecycle>,

    pub description: Option<String>,

    #[validate(length(max = 50))]
    pub category: Option<String>,

    pub metadata: Option<serde_json::Value>,

    pub linked_resources: Option<serde_json::Value>,

    // Inventory context data
    #[validate(range(min = 0))]
    pub quantity: Option<i32>,

    #[validate(length(max = 100))]
    pub location: Option<String>,

    pub pricing: Option<serde_json::Value>,

    #[validate(range(min = 0))]
    pub lead_time: Option<i32>,

    #[validate(range(min = 0))]
    pub min_stock_level: Option<i32>,

    #[validate(range(min = 0))]
    pub max_stock_level: Option<i32>,

    #[validate(range(min = 0))]
    pub reorder_point: Option<i32>,

    pub vendor_id: Option<Uuid>,

    pub last_received_date: Option<DateTime<Utc>>,

    pub status: Option<ItemStatus>,

    pub notes: Option<String>,

    pub inventory_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemResponse {
    pub id: Uuid,
    pub internal_part_number: String,
    pub mfr_part_number: Option<String>,
    pub manufacturer: String,
    pub datasheet: Option<String>,
    pub lifecycle: ItemLifecycle,
    pub description: Option<String>,
    pub category: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub linked_resources: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Inventory context data
    pub context: ItemContext,
    pub quantity: i32,
    pub location: Option<String>,
    pub pricing: Option<serde_json::Value>,
    pub lead_time: Option<i32>,
    pub min_stock_level: i32,
    pub max_stock_level: Option<i32>,
    pub reorder_point: Option<i32>,
    pub vendor_id: Option<Uuid>,
    pub last_received_date: Option<DateTime<Utc>>,
    pub status: ItemStatus,
    pub notes: Option<String>,
    pub inventory_metadata: Option<serde_json::Value>,
    pub inventory_created_at: DateTime<Utc>,
    pub inventory_updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinishedGoodsItemResponse {
    pub id: Uuid,
    pub internal_part_number: String,
    pub mfr_part_number: Option<String>,
    pub manufacturer: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub lifecycle: ItemLifecycle,
    pub quantity: i32,
    pub location: Option<String>,
    pub pricing: Option<serde_json::Value>,
    pub min_stock_level: i32,
    pub max_stock_level: Option<i32>,
    pub reorder_point: Option<i32>,
    pub status: ItemStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreItemResponse {
    pub id: Uuid,
    pub internal_part_number: String,
    pub mfr_part_number: Option<String>,
    pub manufacturer: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub lifecycle: ItemLifecycle,
    pub quantity: i32,
    pub location: Option<String>,
    pub min_stock_level: i32,
    pub max_stock_level: Option<i32>,
    pub reorder_point: Option<i32>,
    pub status: ItemStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VendorItemResponse {
    pub id: Uuid,
    pub internal_part_number: String,
    pub mfr_part_number: Option<String>,
    pub manufacturer: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub lifecycle: ItemLifecycle,
    pub quantity: i32,
    pub location: Option<String>,
    pub pricing: Option<serde_json::Value>,
    pub lead_time: Option<i32>,
    pub vendor_id: Option<Uuid>,
    pub last_received_date: Option<DateTime<Utc>>,
    pub status: ItemStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateBomItemRequest {
    pub parent_item_id: Uuid,
    pub component_item_id: Uuid,

    #[validate(range(min = 1))]
    pub quantity: Option<i32>,

    pub notes: Option<String>,
    pub is_optional: Option<bool>,
    pub substitutes: Option<Vec<Uuid>>,
    pub assembly_order: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateBomItemRequest {
    #[validate(range(min = 1))]
    pub quantity: Option<i32>,

    pub notes: Option<String>,
    pub is_optional: Option<bool>,
    pub substitutes: Option<Vec<Uuid>>,
    pub assembly_order: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BomItemResponse {
    pub id: Uuid,
    pub parent_item: ItemSummary,
    pub component_item: ItemSummary,
    pub quantity: i32,
    pub notes: Option<String>,
    pub is_optional: bool,
    pub substitutes: Vec<Uuid>,
    pub assembly_order: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemSummary {
    pub id: Uuid,
    pub internal_part_number: String,
    pub mfr_part_number: Option<String>,
    pub manufacturer: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateItemIdResponse {
    pub id: Uuid,
}
