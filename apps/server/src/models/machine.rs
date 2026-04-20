use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::{Asset, Item, Person, Tenant};
use crate::schema::*;

// Core machine models

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations,
)]
#[diesel(belongs_to(Tenant, foreign_key = tenant_id))]
#[diesel(table_name = machines)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Machine {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub ip: String,
    pub port: i32,
    pub protocol: String,
    pub status: String,
    pub action: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = machines)]
pub struct NewMachine {
    pub tenant_id: Uuid,
    pub name: String,
    pub ip: String,
    pub port: i32,
    pub protocol: String,
    pub status: String,
    pub action: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

// Machine-Item relationship models

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations,
)]
#[diesel(belongs_to(Machine, foreign_key = machine_id))]
#[diesel(belongs_to(Item, foreign_key = item_id))]
#[diesel(table_name = machine_item_relationships)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MachineItemRelationship {
    pub id: Uuid,
    pub machine_id: Uuid,
    pub item_id: Uuid,
    pub relationship_type: String,
    pub notes: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = machine_item_relationships)]
pub struct NewMachineItemRelationship {
    pub machine_id: Uuid,
    pub item_id: Uuid,
    pub relationship_type: String,
    pub notes: Option<String>,
}

// Machine-Asset relationship models

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations,
)]
#[diesel(belongs_to(Machine, foreign_key = machine_id))]
#[diesel(belongs_to(Asset, foreign_key = asset_id))]
#[diesel(table_name = machine_asset_relationships)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MachineAssetRelationship {
    pub id: Uuid,
    pub machine_id: Uuid,
    pub asset_id: Uuid,
    pub relationship_type: String,
    pub notes: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = machine_asset_relationships)]
pub struct NewMachineAssetRelationship {
    pub machine_id: Uuid,
    pub asset_id: Uuid,
    pub relationship_type: String,
    pub notes: Option<String>,
}

// Machine-Person operator assignment models

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations,
)]
#[diesel(belongs_to(Machine, foreign_key = machine_id))]
#[diesel(belongs_to(Person, foreign_key = person_id))]
#[diesel(table_name = machine_operator_assignments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MachineOperatorAssignment {
    pub id: Uuid,
    pub machine_id: Uuid,
    pub person_id: Uuid,
    pub assignment_type: String,
    pub notes: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = machine_operator_assignments)]
pub struct NewMachineOperatorAssignment {
    pub machine_id: Uuid,
    pub person_id: Uuid,
    pub assignment_type: String,
    pub notes: Option<String>,
}

// Machine-Job assignment models

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations,
)]
#[diesel(belongs_to(Machine, foreign_key = machine_id))]
#[diesel(table_name = machine_job_assignments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MachineJobAssignment {
    pub id: Uuid,
    pub machine_id: Uuid,
    pub job_id: Uuid,
    pub status: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = machine_job_assignments)]
pub struct NewMachineJobAssignment {
    pub machine_id: Uuid,
    pub job_id: Uuid,
    pub status: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

// Enums for better type safety

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MachineProtocol {
    #[serde(rename = "http")]
    Http,
    #[serde(rename = "mqtt")]
    Mqtt,
    #[serde(rename = "graph")]
    Graph,
    #[serde(rename = "tcp")]
    Tcp,
    #[serde(rename = "udp")]
    Udp,
    #[serde(rename = "websocket")]
    WebSocket,
}

impl std::fmt::Display for MachineProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MachineProtocol::Http => write!(f, "http"),
            MachineProtocol::Mqtt => write!(f, "mqtt"),
            MachineProtocol::Graph => write!(f, "graph"),
            MachineProtocol::Tcp => write!(f, "tcp"),
            MachineProtocol::Udp => write!(f, "udp"),
            MachineProtocol::WebSocket => write!(f, "websocket"),
        }
    }
}

impl From<MachineProtocol> for String {
    fn from(protocol: MachineProtocol) -> Self {
        protocol.to_string()
    }
}

impl TryFrom<String> for MachineProtocol {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "http" => Ok(MachineProtocol::Http),
            "mqtt" => Ok(MachineProtocol::Mqtt),
            "graph" => Ok(MachineProtocol::Graph),
            "tcp" => Ok(MachineProtocol::Tcp),
            "udp" => Ok(MachineProtocol::Udp),
            "websocket" => Ok(MachineProtocol::WebSocket),
            _ => Err(format!("Invalid machine protocol: {}", value)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MachineStatus {
    #[serde(rename = "offline")]
    Offline,
    #[serde(rename = "idle")]
    Idle,
    #[serde(rename = "busy")]
    Busy,
    #[serde(rename = "maintenance")]
    Maintenance,
    #[serde(rename = "error")]
    Error,
}

impl std::fmt::Display for MachineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MachineStatus::Offline => write!(f, "offline"),
            MachineStatus::Idle => write!(f, "idle"),
            MachineStatus::Busy => write!(f, "busy"),
            MachineStatus::Maintenance => write!(f, "maintenance"),
            MachineStatus::Error => write!(f, "error"),
        }
    }
}

impl From<MachineStatus> for String {
    fn from(status: MachineStatus) -> Self {
        status.to_string()
    }
}

impl TryFrom<String> for MachineStatus {
    type Error = String;

    fn try_from(value: String) -> Result<Self, <Self as TryFrom<String>>::Error> {
        match value.as_str() {
            "offline" => Ok(MachineStatus::Offline),
            "idle" => Ok(MachineStatus::Idle),
            "busy" => Ok(MachineStatus::Busy),
            "maintenance" => Ok(MachineStatus::Maintenance),
            "error" => Ok(MachineStatus::Error),
            _ => Err(format!("Invalid machine status: {}", value)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MachineAction {
    #[serde(rename = "run")]
    Run,
    #[serde(rename = "test")]
    Test,
    #[serde(rename = "calibrate")]
    Calibrate,
    #[serde(rename = "diagnostics")]
    Diagnostics,
    #[serde(rename = "emergency_stop")]
    EmergencyStop,
}

impl std::fmt::Display for MachineAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MachineAction::Run => write!(f, "run"),
            MachineAction::Test => write!(f, "test"),
            MachineAction::Calibrate => write!(f, "calibrate"),
            MachineAction::Diagnostics => write!(f, "diagnostics"),
            MachineAction::EmergencyStop => write!(f, "emergency_stop"),
        }
    }
}

impl From<MachineAction> for String {
    fn from(action: MachineAction) -> Self {
        action.to_string()
    }
}

impl TryFrom<String> for MachineAction {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "run" => Ok(MachineAction::Run),
            "test" => Ok(MachineAction::Test),
            "calibrate" => Ok(MachineAction::Calibrate),
            "diagnostics" => Ok(MachineAction::Diagnostics),
            "emergency_stop" => Ok(MachineAction::EmergencyStop),
            _ => Err(format!("Invalid machine action: {}", value)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ItemRelationshipType {
    #[serde(rename = "builds")]
    Builds,
    #[serde(rename = "tests")]
    Tests,
    #[serde(rename = "calibrates")]
    Calibrates,
}

impl std::fmt::Display for ItemRelationshipType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemRelationshipType::Builds => write!(f, "builds"),
            ItemRelationshipType::Tests => write!(f, "tests"),
            ItemRelationshipType::Calibrates => write!(f, "calibrates"),
        }
    }
}

impl From<ItemRelationshipType> for String {
    fn from(rel_type: ItemRelationshipType) -> Self {
        rel_type.to_string()
    }
}

impl TryFrom<String> for ItemRelationshipType {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "builds" => Ok(ItemRelationshipType::Builds),
            "tests" => Ok(ItemRelationshipType::Tests),
            "calibrates" => Ok(ItemRelationshipType::Calibrates),
            _ => Err(format!("Invalid item relationship type: {}", value)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AssetRelationshipType {
    #[serde(rename = "firmware")]
    Firmware,
    #[serde(rename = "configuration")]
    Configuration,
    #[serde(rename = "calibration_data")]
    CalibrationData,
}

impl std::fmt::Display for AssetRelationshipType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetRelationshipType::Firmware => write!(f, "firmware"),
            AssetRelationshipType::Configuration => write!(f, "configuration"),
            AssetRelationshipType::CalibrationData => write!(f, "calibration_data"),
        }
    }
}

impl From<AssetRelationshipType> for String {
    fn from(rel_type: AssetRelationshipType) -> Self {
        rel_type.to_string()
    }
}

impl TryFrom<String> for AssetRelationshipType {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "firmware" => Ok(AssetRelationshipType::Firmware),
            "configuration" => Ok(AssetRelationshipType::Configuration),
            "calibration_data" => Ok(AssetRelationshipType::CalibrationData),
            _ => Err(format!("Invalid asset relationship type: {}", value)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperatorAssignmentType {
    #[serde(rename = "primary")]
    Primary,
    #[serde(rename = "backup")]
    Backup,
    #[serde(rename = "maintenance")]
    Maintenance,
}

impl std::fmt::Display for OperatorAssignmentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperatorAssignmentType::Primary => write!(f, "primary"),
            OperatorAssignmentType::Backup => write!(f, "backup"),
            OperatorAssignmentType::Maintenance => write!(f, "maintenance"),
        }
    }
}

impl From<OperatorAssignmentType> for String {
    fn from(assignment_type: OperatorAssignmentType) -> Self {
        assignment_type.to_string()
    }
}

impl TryFrom<String> for OperatorAssignmentType {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "primary" => Ok(OperatorAssignmentType::Primary),
            "backup" => Ok(OperatorAssignmentType::Backup),
            "maintenance" => Ok(OperatorAssignmentType::Maintenance),
            _ => Err(format!("Invalid operator assignment type: {}", value)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobAssignmentStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
}

impl std::fmt::Display for JobAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobAssignmentStatus::Pending => write!(f, "pending"),
            JobAssignmentStatus::InProgress => write!(f, "in_progress"),
            JobAssignmentStatus::Completed => write!(f, "completed"),
            JobAssignmentStatus::Failed => write!(f, "failed"),
        }
    }
}

impl From<JobAssignmentStatus> for String {
    fn from(status: JobAssignmentStatus) -> Self {
        status.to_string()
    }
}

impl TryFrom<String> for JobAssignmentStatus {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "pending" => Ok(JobAssignmentStatus::Pending),
            "in_progress" => Ok(JobAssignmentStatus::InProgress),
            "completed" => Ok(JobAssignmentStatus::Completed),
            "failed" => Ok(JobAssignmentStatus::Failed),
            _ => Err(format!("Invalid job assignment status: {}", value)),
        }
    }
}

// Request/Response DTOs

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateMachineRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    #[validate(length(min = 7, max = 45))] // IPv4 min: 1.1.1.1, IPv6 max
    pub ip: String,

    #[validate(range(min = 1, max = 65535))]
    pub port: i32,

    pub protocol: MachineProtocol,

    pub status: Option<MachineStatus>,

    pub action: Option<MachineAction>,

    pub payload: Option<serde_json::Value>,

    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateMachineRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,

    #[validate(length(min = 7, max = 45))]
    pub ip: Option<String>,

    #[validate(range(min = 1, max = 65535))]
    pub port: Option<i32>,

    pub protocol: Option<MachineProtocol>,

    pub status: Option<MachineStatus>,

    pub action: Option<MachineAction>,

    pub payload: Option<serde_json::Value>,

    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MachineResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub ip: String,
    pub port: i32,
    pub protocol: MachineProtocol,
    pub status: MachineStatus,
    pub action: Option<MachineAction>,
    pub payload: Option<serde_json::Value>,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateMachineItemRelationshipRequest {
    pub item_id: Uuid,
    pub relationship_type: ItemRelationshipType,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateMachineAssetRelationshipRequest {
    pub asset_id: Uuid,
    pub relationship_type: AssetRelationshipType,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateMachineOperatorAssignmentRequest {
    pub person_id: Uuid,
    pub assignment_type: OperatorAssignmentType,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateMachineJobAssignmentRequest {
    pub job_id: Uuid,
    pub status: Option<JobAssignmentStatus>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateMachineJobAssignmentRequest {
    pub status: Option<JobAssignmentStatus>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MachineItemRelationshipResponse {
    pub id: Uuid,
    pub machine_id: Uuid,
    pub item_id: Uuid,
    pub relationship_type: ItemRelationshipType,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MachineAssetRelationshipResponse {
    pub id: Uuid,
    pub machine_id: Uuid,
    pub asset_id: Uuid,
    pub relationship_type: AssetRelationshipType,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MachineOperatorAssignmentResponse {
    pub id: Uuid,
    pub machine_id: Uuid,
    pub person_id: Uuid,
    pub assignment_type: OperatorAssignmentType,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MachineJobAssignmentResponse {
    pub id: Uuid,
    pub machine_id: Uuid,
    pub job_id: Uuid,
    pub status: JobAssignmentStatus,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub status: MachineStatus,
    pub action: Option<MachineAction>,
    pub payload: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MachineCreateIdResponse {
    pub id: Uuid,
}
