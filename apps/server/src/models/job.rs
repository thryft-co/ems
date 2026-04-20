use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::tenant::Tenant;
use crate::schema::*;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = jobs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Job {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub job_number: String,
    pub item_id: Option<Uuid>,
    pub quantity: i32,
    pub assigned_person_id: Option<Uuid>,
    pub supervisor_id: Option<Uuid>,
    pub customer_id: Option<Uuid>,
    pub job_type: String,
    pub priority: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub status: String,
    pub comments: Option<String>,
    pub materials_consumed: Option<serde_json::Value>,
    pub labor_hours: Option<f64>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = jobs)]
pub struct NewJob {
    pub tenant_id: Uuid,
    pub job_number: String,
    pub item_id: Option<Uuid>,
    pub quantity: i32,
    pub assigned_person_id: Option<Uuid>,
    pub supervisor_id: Option<Uuid>,
    pub customer_id: Option<Uuid>,
    pub job_type: String,
    pub priority: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub status: String,
    pub comments: Option<String>,
    pub materials_consumed: Option<serde_json::Value>,
    pub labor_hours: Option<f64>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations,
)]
#[diesel(belongs_to(Job, foreign_key = job_id))]
#[diesel(belongs_to(Tenant, foreign_key = tenant_id))]
#[diesel(table_name = manufacturing_job)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ManufacturingJob {
    pub id: Uuid,
    pub job_id: Uuid,
    pub tenant_id: Uuid,
    pub work_order_number: Option<String>,
    pub production_line: Option<String>,
    pub machine_id: Option<String>,
    pub setup_time_hours: Option<f64>,
    pub cycle_time_minutes: Option<f64>,
    pub quality_check_required: Option<bool>,
    pub batch_size: Option<i32>,
    pub tool_requirements: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = manufacturing_job)]
pub struct NewManufacturingJob {
    pub job_id: Uuid,
    pub tenant_id: Uuid,
    pub work_order_number: Option<String>,
    pub production_line: Option<String>,
    pub machine_id: Option<String>,
    pub setup_time_hours: Option<f64>,
    pub cycle_time_minutes: Option<f64>,
    pub quality_check_required: Option<bool>,
    pub batch_size: Option<i32>,
    pub tool_requirements: Option<serde_json::Value>,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations,
)]
#[diesel(belongs_to(Job, foreign_key = job_id))]
#[diesel(belongs_to(Tenant, foreign_key = tenant_id))]
#[diesel(table_name = qa_job)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct QaJob {
    pub id: Uuid,
    pub job_id: Uuid,
    pub tenant_id: Uuid,
    pub inspection_type: Option<String>,
    pub test_procedure_id: Option<String>,
    pub acceptance_criteria: Option<String>,
    pub sampling_size: Option<i32>,
    pub test_equipment: Option<serde_json::Value>,
    pub calibration_required: Option<bool>,
    pub environmental_conditions: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = qa_job)]
pub struct NewQaJob {
    pub job_id: Uuid,
    pub tenant_id: Uuid,
    pub inspection_type: Option<String>,
    pub test_procedure_id: Option<String>,
    pub acceptance_criteria: Option<String>,
    pub sampling_size: Option<i32>,
    pub test_equipment: Option<serde_json::Value>,
    pub calibration_required: Option<bool>,
    pub environmental_conditions: Option<serde_json::Value>,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations,
)]
#[diesel(belongs_to(Job, foreign_key = job_id))]
#[diesel(belongs_to(Tenant, foreign_key = tenant_id))]
#[diesel(table_name = service_job)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ServiceJob {
    pub id: Uuid,
    pub job_id: Uuid,
    pub tenant_id: Uuid,
    pub service_type: Option<String>,
    pub location: Option<String>,
    pub equipment_serial_number: Option<String>,
    pub maintenance_type: Option<String>,
    pub parts_required: Option<serde_json::Value>,
    pub safety_requirements: Option<serde_json::Value>,
    pub travel_time_hours: Option<f64>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = service_job)]
pub struct NewServiceJob {
    pub job_id: Uuid,
    pub tenant_id: Uuid,
    pub service_type: Option<String>,
    pub location: Option<String>,
    pub equipment_serial_number: Option<String>,
    pub maintenance_type: Option<String>,
    pub parts_required: Option<serde_json::Value>,
    pub safety_requirements: Option<serde_json::Value>,
    pub travel_time_hours: Option<f64>,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations,
)]
#[diesel(belongs_to(Job, foreign_key = job_id))]
#[diesel(belongs_to(Tenant, foreign_key = tenant_id))]
#[diesel(table_name = job_history)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct JobHistory {
    pub id: Uuid,
    pub job_id: Uuid,
    pub tenant_id: Uuid,
    pub person_id: Option<Uuid>,
    pub action: String,
    pub previous_status: Option<String>,
    pub new_status: Option<String>,
    pub notes: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = job_history)]
pub struct NewJobHistory {
    pub job_id: Uuid,
    pub tenant_id: Uuid,
    pub person_id: Option<Uuid>,
    pub action: String,
    pub previous_status: Option<String>,
    pub new_status: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobType {
    #[serde(rename = "manufacturing")]
    Manufacturing,
    #[serde(rename = "qa")]
    Qa,
    #[serde(rename = "service")]
    Service,
}

impl std::fmt::Display for JobType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobType::Manufacturing => write!(f, "manufacturing"),
            JobType::Qa => write!(f, "qa"),
            JobType::Service => write!(f, "service"),
        }
    }
}

impl From<JobType> for String {
    fn from(job_type: JobType) -> Self {
        job_type.to_string()
    }
}

impl TryFrom<String> for JobType {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "manufacturing" => Ok(JobType::Manufacturing),
            "qa" => Ok(JobType::Qa),
            "service" => Ok(JobType::Service),
            _ => Err(format!("Invalid job type: {}", value)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "on_hold")]
    OnHold,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "cancelled")]
    Cancelled,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "pending"),
            JobStatus::InProgress => write!(f, "in_progress"),
            JobStatus::OnHold => write!(f, "on_hold"),
            JobStatus::Completed => write!(f, "completed"),
            JobStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl From<JobStatus> for String {
    fn from(status: JobStatus) -> Self {
        status.to_string()
    }
}

impl TryFrom<String> for JobStatus {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "pending" => Ok(JobStatus::Pending),
            "in_progress" => Ok(JobStatus::InProgress),
            "on_hold" => Ok(JobStatus::OnHold),
            "completed" => Ok(JobStatus::Completed),
            "cancelled" => Ok(JobStatus::Cancelled),
            _ => Err(format!("Invalid job status: {}", value)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobPriority {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "urgent")]
    Urgent,
}

impl std::fmt::Display for JobPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobPriority::Low => write!(f, "low"),
            JobPriority::Normal => write!(f, "normal"),
            JobPriority::High => write!(f, "high"),
            JobPriority::Urgent => write!(f, "urgent"),
        }
    }
}

impl From<JobPriority> for String {
    fn from(priority: JobPriority) -> Self {
        priority.to_string()
    }
}

impl TryFrom<String> for JobPriority {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "low" => Ok(JobPriority::Low),
            "normal" => Ok(JobPriority::Normal),
            "high" => Ok(JobPriority::High),
            "urgent" => Ok(JobPriority::Urgent),
            _ => Err(format!("Invalid job priority: {}", value)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateJobRequest {
    #[validate(length(min = 1, max = 50))]
    pub job_number: String,

    pub item_id: Option<Uuid>,

    #[validate(range(min = 1))]
    pub quantity: i32,

    pub assigned_person_id: Option<Uuid>,

    pub supervisor_id: Option<Uuid>,

    pub customer_id: Option<Uuid>,

    pub job_type: JobType,

    pub priority: Option<JobPriority>,

    pub start_date: Option<DateTime<Utc>>,

    pub end_date: Option<DateTime<Utc>>,

    pub due_date: Option<DateTime<Utc>>,

    pub status: Option<JobStatus>,

    pub comments: Option<String>,

    pub materials_consumed: Option<serde_json::Value>,

    pub labor_hours: Option<f64>,

    pub metadata: Option<serde_json::Value>,

    // Manufacturing job specific fields
    #[validate(length(max = 50))]
    pub work_order_number: Option<String>,

    #[validate(length(max = 100))]
    pub production_line: Option<String>,

    #[validate(length(max = 50))]
    pub machine_id: Option<String>,

    pub setup_time_hours: Option<f64>,

    pub cycle_time_minutes: Option<f64>,

    pub quality_check_required: Option<bool>,

    pub batch_size: Option<i32>,

    pub tool_requirements: Option<serde_json::Value>,

    // QA job specific fields
    #[validate(length(max = 50))]
    pub inspection_type: Option<String>,

    #[validate(length(max = 50))]
    pub test_procedure_id: Option<String>,

    pub acceptance_criteria: Option<String>,

    pub sampling_size: Option<i32>,

    pub test_equipment: Option<serde_json::Value>,

    pub calibration_required: Option<bool>,

    pub environmental_conditions: Option<serde_json::Value>,

    // Service job specific fields
    #[validate(length(max = 50))]
    pub service_type: Option<String>,

    #[validate(length(max = 200))]
    pub location: Option<String>,

    #[validate(length(max = 100))]
    pub equipment_serial_number: Option<String>,

    #[validate(length(max = 50))]
    pub maintenance_type: Option<String>,

    pub parts_required: Option<serde_json::Value>,

    pub safety_requirements: Option<serde_json::Value>,

    pub travel_time_hours: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateJobRequest {
    #[validate(range(min = 1))]
    pub quantity: Option<i32>,

    pub assigned_person_id: Option<Uuid>,

    pub supervisor_id: Option<Uuid>,

    pub customer_id: Option<Uuid>,

    pub priority: Option<JobPriority>,

    pub start_date: Option<DateTime<Utc>>,

    pub end_date: Option<DateTime<Utc>>,

    pub due_date: Option<DateTime<Utc>>,

    pub status: Option<JobStatus>,

    pub comments: Option<String>,

    pub materials_consumed: Option<serde_json::Value>,

    pub labor_hours: Option<f64>,

    pub metadata: Option<serde_json::Value>,

    // Manufacturing job specific fields
    #[validate(length(max = 50))]
    pub work_order_number: Option<String>,

    #[validate(length(max = 100))]
    pub production_line: Option<String>,

    #[validate(length(max = 50))]
    pub machine_id: Option<String>,

    pub setup_time_hours: Option<f64>,

    pub cycle_time_minutes: Option<f64>,

    pub quality_check_required: Option<bool>,

    pub batch_size: Option<i32>,

    pub tool_requirements: Option<serde_json::Value>,

    // QA job specific fields
    #[validate(length(max = 50))]
    pub inspection_type: Option<String>,

    #[validate(length(max = 50))]
    pub test_procedure_id: Option<String>,

    pub acceptance_criteria: Option<String>,

    pub sampling_size: Option<i32>,

    pub test_equipment: Option<serde_json::Value>,

    pub calibration_required: Option<bool>,

    pub environmental_conditions: Option<serde_json::Value>,

    // Service job specific fields
    #[validate(length(max = 50))]
    pub service_type: Option<String>,

    #[validate(length(max = 200))]
    pub location: Option<String>,

    #[validate(length(max = 100))]
    pub equipment_serial_number: Option<String>,

    #[validate(length(max = 50))]
    pub maintenance_type: Option<String>,

    pub parts_required: Option<serde_json::Value>,

    pub safety_requirements: Option<serde_json::Value>,

    pub travel_time_hours: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobResponse {
    pub id: Uuid,
    pub job_number: String,
    pub item_id: Option<Uuid>,
    pub quantity: i32,
    pub assigned_person_id: Option<Uuid>,
    pub supervisor_id: Option<Uuid>,
    pub customer_id: Option<Uuid>,
    pub job_type: JobType,
    pub priority: JobPriority,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub status: JobStatus,
    pub comments: Option<String>,
    pub materials_consumed: Option<serde_json::Value>,
    pub labor_hours: Option<f64>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Type-specific data
    pub manufacturing: Option<ManufacturingJobData>,
    pub qa: Option<QaJobData>,
    pub service: Option<ServiceJobData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManufacturingJobResponse {
    pub id: Uuid,
    pub job_number: String,
    pub quantity: i32,
    pub assigned_person_id: Option<Uuid>,
    pub supervisor_id: Option<Uuid>,
    pub customer_id: Option<Uuid>,
    pub priority: JobPriority,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub status: JobStatus,
    pub work_order_number: Option<String>,
    pub production_line: Option<String>,
    pub machine_id: Option<String>,
    pub setup_time_hours: Option<f64>,
    pub cycle_time_minutes: Option<f64>,
    pub quality_check_required: Option<bool>,
    pub batch_size: Option<i32>,
    pub tool_requirements: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QaJobResponse {
    pub id: Uuid,
    pub job_number: String,
    pub quantity: i32,
    pub assigned_person_id: Option<Uuid>,
    pub supervisor_id: Option<Uuid>,
    pub customer_id: Option<Uuid>,
    pub priority: JobPriority,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub status: JobStatus,
    pub inspection_type: Option<String>,
    pub test_procedure_id: Option<String>,
    pub acceptance_criteria: Option<String>,
    pub sampling_size: Option<i32>,
    pub test_equipment: Option<serde_json::Value>,
    pub calibration_required: Option<bool>,
    pub environmental_conditions: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceJobResponse {
    pub id: Uuid,
    pub job_number: String,
    pub quantity: i32,
    pub assigned_person_id: Option<Uuid>,
    pub supervisor_id: Option<Uuid>,
    pub customer_id: Option<Uuid>,
    pub priority: JobPriority,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub status: JobStatus,
    pub service_type: Option<String>,
    pub location: Option<String>,
    pub equipment_serial_number: Option<String>,
    pub maintenance_type: Option<String>,
    pub parts_required: Option<serde_json::Value>,
    pub safety_requirements: Option<serde_json::Value>,
    pub travel_time_hours: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManufacturingJobData {
    pub work_order_number: Option<String>,
    pub production_line: Option<String>,
    pub machine_id: Option<String>,
    pub setup_time_hours: Option<f64>,
    pub cycle_time_minutes: Option<f64>,
    pub quality_check_required: Option<bool>,
    pub batch_size: Option<i32>,
    pub tool_requirements: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QaJobData {
    pub inspection_type: Option<String>,
    pub test_procedure_id: Option<String>,
    pub acceptance_criteria: Option<String>,
    pub sampling_size: Option<i32>,
    pub test_equipment: Option<serde_json::Value>,
    pub calibration_required: Option<bool>,
    pub environmental_conditions: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceJobData {
    pub service_type: Option<String>,
    pub location: Option<String>,
    pub equipment_serial_number: Option<String>,
    pub maintenance_type: Option<String>,
    pub parts_required: Option<serde_json::Value>,
    pub safety_requirements: Option<serde_json::Value>,
    pub travel_time_hours: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateJobIdResponse {
    pub id: Uuid,
}
