use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::schema::*;

// Core Order Models
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = orders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Order {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub order_number: String,
    pub order_type: String,
    pub external_entity_id: Uuid,
    pub external_entity_type: String,
    pub order_date: DateTime<Utc>,
    pub total_amount: f64,
    pub status: String,
    pub created_by_id: Uuid,
    pub notes: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = orders)]
pub struct NewOrder {
    pub tenant_id: Uuid,
    pub order_number: String,
    pub order_type: String,
    pub external_entity_id: Uuid,
    pub external_entity_type: String,
    pub order_date: DateTime<Utc>,
    pub total_amount: f64,
    pub status: String,
    pub created_by_id: Uuid,
    pub notes: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// Order Item Models
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = order_items)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub item_id: Option<Uuid>,
    pub item_name: String,
    pub item_description: Option<String>,
    pub quantity: i32,
    pub unit_price: f64,
    pub extended_price: f64,
    pub notes: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = order_items)]
pub struct NewOrderItem {
    pub order_id: Uuid,
    pub item_id: Option<Uuid>,
    pub item_name: String,
    pub item_description: Option<String>,
    pub quantity: i32,
    pub unit_price: f64,
    pub extended_price: f64,
    pub notes: Option<String>,
}

// Order History Models
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = order_history)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OrderHistory {
    pub id: Uuid,
    pub order_id: Uuid,
    pub tenant_id: Uuid,
    pub person_id: Option<Uuid>,
    pub action: String,
    pub previous_status: Option<String>,
    pub new_status: Option<String>,
    pub notes: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = order_history)]
pub struct NewOrderHistory {
    pub order_id: Uuid,
    pub tenant_id: Uuid,
    pub person_id: Option<Uuid>,
    pub action: String,
    pub previous_status: Option<String>,
    pub new_status: Option<String>,
    pub notes: Option<String>,
}

// Enums
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderType {
    #[serde(rename = "purchase_order")]
    PurchaseOrder,
    #[serde(rename = "customer_order")]
    CustomerOrder,
    #[serde(rename = "distributor_order")]
    DistributorOrder,
}

impl std::fmt::Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderType::PurchaseOrder => write!(f, "purchase_order"),
            OrderType::CustomerOrder => write!(f, "customer_order"),
            OrderType::DistributorOrder => write!(f, "distributor_order"),
        }
    }
}

impl From<OrderType> for String {
    fn from(order_type: OrderType) -> Self {
        order_type.to_string()
    }
}

impl TryFrom<String> for OrderType {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "purchase_order" => Ok(OrderType::PurchaseOrder),
            "customer_order" => Ok(OrderType::CustomerOrder),
            "distributor_order" => Ok(OrderType::DistributorOrder),
            _ => Err(format!("Invalid order type: {}", value)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderStatus {
    #[serde(rename = "draft")]
    Draft,
    #[serde(rename = "submitted")]
    Submitted,
    #[serde(rename = "approved")]
    Approved,
    #[serde(rename = "fulfilled")]
    Fulfilled,
    #[serde(rename = "partially_fulfilled")]
    PartiallyFulfilled,
    #[serde(rename = "cancelled")]
    Cancelled,
    #[serde(rename = "paid")]
    Paid,
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderStatus::Draft => write!(f, "draft"),
            OrderStatus::Submitted => write!(f, "submitted"),
            OrderStatus::Approved => write!(f, "approved"),
            OrderStatus::Fulfilled => write!(f, "fulfilled"),
            OrderStatus::PartiallyFulfilled => write!(f, "partially_fulfilled"),
            OrderStatus::Cancelled => write!(f, "cancelled"),
            OrderStatus::Paid => write!(f, "paid"),
        }
    }
}

impl From<OrderStatus> for String {
    fn from(status: OrderStatus) -> Self {
        status.to_string()
    }
}

impl TryFrom<String> for OrderStatus {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "draft" => Ok(OrderStatus::Draft),
            "submitted" => Ok(OrderStatus::Submitted),
            "approved" => Ok(OrderStatus::Approved),
            "fulfilled" => Ok(OrderStatus::Fulfilled),
            "partially_fulfilled" => Ok(OrderStatus::PartiallyFulfilled),
            "cancelled" => Ok(OrderStatus::Cancelled),
            "paid" => Ok(OrderStatus::Paid),
            _ => Err(format!("Invalid order status: {}", value)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExternalEntityType {
    #[serde(rename = "vendor")]
    Vendor,
    #[serde(rename = "customer")]
    Customer,
    #[serde(rename = "distributor")]
    Distributor,
}

impl std::fmt::Display for ExternalEntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExternalEntityType::Vendor => write!(f, "vendor"),
            ExternalEntityType::Customer => write!(f, "customer"),
            ExternalEntityType::Distributor => write!(f, "distributor"),
        }
    }
}

impl From<ExternalEntityType> for String {
    fn from(entity_type: ExternalEntityType) -> Self {
        entity_type.to_string()
    }
}

impl TryFrom<String> for ExternalEntityType {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "vendor" => Ok(ExternalEntityType::Vendor),
            "customer" => Ok(ExternalEntityType::Customer),
            "distributor" => Ok(ExternalEntityType::Distributor),
            _ => Err(format!("Invalid external entity type: {}", value)),
        }
    }
}

// Request/Response Models
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateOrderRequest {
    #[validate(length(min = 1, max = 50))]
    pub order_number: String,

    pub order_type: OrderType,

    pub external_entity_id: Uuid,

    pub external_entity_type: ExternalEntityType,

    pub order_date: DateTime<Utc>,

    #[validate(range(min = 0.0))]
    pub total_amount: f64,

    pub status: Option<OrderStatus>,

    pub created_by_id: Uuid,

    #[validate(length(max = 1000))]
    pub notes: Option<String>,

    pub metadata: Option<serde_json::Value>,

    pub items: Vec<CreateOrderItemRequest>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateOrderItemRequest {
    pub item_id: Option<Uuid>,

    #[validate(length(min = 1, max = 200))]
    pub item_name: String,

    #[validate(length(max = 1000))]
    pub item_description: Option<String>,

    #[validate(range(min = 1))]
    pub quantity: i32,

    #[validate(range(min = 0.0))]
    pub unit_price: f64,

    #[validate(length(max = 500))]
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateOrderRequest {
    #[validate(length(min = 1, max = 50))]
    pub order_number: Option<String>,

    pub external_entity_id: Option<Uuid>,

    pub order_date: Option<DateTime<Utc>>,

    #[validate(range(min = 0.0))]
    pub total_amount: Option<f64>,

    pub status: Option<OrderStatus>,

    #[validate(length(max = 1000))]
    pub notes: Option<String>,

    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderResponse {
    pub id: Uuid,
    pub order_number: String,
    pub order_type: OrderType,
    pub external_entity_id: Uuid,
    pub external_entity_type: ExternalEntityType,
    pub order_date: DateTime<Utc>,
    pub total_amount: f64,
    pub status: OrderStatus,
    pub created_by_id: Uuid,
    pub notes: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub items: Vec<OrderItemResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderItemResponse {
    pub id: Uuid,
    pub item_id: Option<Uuid>,
    pub item_name: String,
    pub item_description: Option<String>,
    pub quantity: i32,
    pub unit_price: f64,
    pub extended_price: f64,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PurchaseOrderResponse {
    pub id: Uuid,
    pub order_number: String,
    pub vendor_id: Uuid,
    pub order_date: DateTime<Utc>,
    pub total_amount: f64,
    pub status: OrderStatus,
    pub created_by_id: Uuid,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub items: Vec<OrderItemResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerOrderResponse {
    pub id: Uuid,
    pub order_number: String,
    pub customer_id: Uuid,
    pub order_date: DateTime<Utc>,
    pub total_amount: f64,
    pub status: OrderStatus,
    pub created_by_id: Uuid,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub items: Vec<OrderItemResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DistributorOrderResponse {
    pub id: Uuid,
    pub order_number: String,
    pub distributor_id: Uuid,
    pub order_date: DateTime<Utc>,
    pub total_amount: f64,
    pub status: OrderStatus,
    pub created_by_id: Uuid,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub items: Vec<OrderItemResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderIdResponse {
    pub id: Uuid,
}
