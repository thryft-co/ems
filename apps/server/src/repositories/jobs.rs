use anyhow::Result;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl, SimpleAsyncConnection};
use uuid::Uuid;

use crate::models::{
    CreateJobIdResponse, CreateJobRequest, Job, JobPriority, JobResponse, JobStatus, JobType,
    ManufacturingJob, ManufacturingJobData, ManufacturingJobResponse, NewJob, NewManufacturingJob,
    NewQaJob, NewServiceJob, QaJob, QaJobData, QaJobResponse, ServiceJob, ServiceJobData,
    ServiceJobResponse, UpdateJobRequest,
};
use crate::schema::*;
use crate::repositories::DatabaseService;

pub struct JobService {
    database: DatabaseService,
}

impl JobService {
    pub fn new(database: DatabaseService) -> Self {
        Self { database }
    }

    // General Job API methods

    pub async fn create_job(
        &self,
        tenant_id: Uuid,
        request: CreateJobRequest,
    ) -> Result<CreateJobIdResponse> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let job_id = conn
            .transaction::<_, diesel::result::Error, _>(|conn| {
                Box::pin(async move {
                    // Create job record
                    let new_job = NewJob {
                        tenant_id,
                        job_number: request.job_number.clone(),
                        item_id: request.item_id,
                        quantity: request.quantity,
                        assigned_person_id: request.assigned_person_id,
                        supervisor_id: request.supervisor_id,
                        customer_id: request.customer_id,
                        job_type: request.job_type.to_string(),
                        priority: request.priority.map(|p| p.to_string()),
                        start_date: request.start_date,
                        end_date: request.end_date,
                        due_date: request.due_date,
                        status: request.status.unwrap_or(JobStatus::Pending).to_string(),
                        comments: request.comments,
                        materials_consumed: request.materials_consumed,
                        labor_hours: request.labor_hours,
                        metadata: request.metadata,
                    };

                    let job: Job = diesel::insert_into(jobs::table)
                        .values(&new_job)
                        .returning(Job::as_returning())
                        .get_result(conn)
                        .await?;

                    // Create type-specific data based on job type
                    match request.job_type {
                        JobType::Manufacturing => {
                            let new_manufacturing = NewManufacturingJob {
                                job_id: job.id,
                                tenant_id,
                                work_order_number: request.work_order_number,
                                production_line: request.production_line,
                                machine_id: request.machine_id,
                                setup_time_hours: request.setup_time_hours,
                                cycle_time_minutes: request.cycle_time_minutes,
                                quality_check_required: request.quality_check_required,
                                batch_size: request.batch_size,
                                tool_requirements: request.tool_requirements,
                            };
                            diesel::insert_into(manufacturing_job::table)
                                .values(&new_manufacturing)
                                .execute(conn)
                                .await?;
                        }
                        JobType::Qa => {
                            let new_qa = NewQaJob {
                                job_id: job.id,
                                tenant_id,
                                inspection_type: request.inspection_type,
                                test_procedure_id: request.test_procedure_id,
                                acceptance_criteria: request.acceptance_criteria,
                                sampling_size: request.sampling_size,
                                test_equipment: request.test_equipment,
                                calibration_required: request.calibration_required,
                                environmental_conditions: request.environmental_conditions,
                            };
                            diesel::insert_into(qa_job::table)
                                .values(&new_qa)
                                .execute(conn)
                                .await?;
                        }
                        JobType::Service => {
                            let new_service = NewServiceJob {
                                job_id: job.id,
                                tenant_id,
                                service_type: request.service_type,
                                location: request.location,
                                equipment_serial_number: request.equipment_serial_number,
                                maintenance_type: request.maintenance_type,
                                parts_required: request.parts_required,
                                safety_requirements: request.safety_requirements,
                                travel_time_hours: request.travel_time_hours,
                            };
                            diesel::insert_into(service_job::table)
                                .values(&new_service)
                                .execute(conn)
                                .await?;
                        }
                    }

                    Ok(job.id)
                })
            })
            .await
            .map_err(|e| anyhow::anyhow!("Transaction failed: {}", e))?;

        Ok(CreateJobIdResponse { id: job_id })
    }

    pub async fn get_job_by_id(
        &self,
        tenant_id: Uuid,
        job_id: Uuid,
    ) -> Result<Option<JobResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        // Get job
        let job = jobs::table
            .filter(jobs::id.eq(job_id))
            .filter(jobs::tenant_id.eq(tenant_id))
            .select(Job::as_select())
            .first::<Job>(&mut conn)
            .await
            .optional()?;

        if let Some(job) = job {
            let job_type = JobType::try_from(job.job_type.clone())
                .map_err(|e| anyhow::anyhow!("Invalid job type: {}", e))?;

            // Get type-specific data based on job type
            let (manufacturing, qa, service) = match job_type {
                JobType::Manufacturing => {
                    let manufacturing_data = manufacturing_job::table
                        .filter(manufacturing_job::job_id.eq(job_id))
                        .filter(manufacturing_job::tenant_id.eq(tenant_id))
                        .select(ManufacturingJob::as_select())
                        .first::<ManufacturingJob>(&mut conn)
                        .await
                        .optional()?;

                    let manufacturing = manufacturing_data.map(|m| ManufacturingJobData {
                        work_order_number: m.work_order_number,
                        production_line: m.production_line,
                        machine_id: m.machine_id,
                        setup_time_hours: m
                            .setup_time_hours
                            .map(|bd| bd.to_string().parse().unwrap_or(0.0)),
                        cycle_time_minutes: m
                            .cycle_time_minutes
                            .map(|bd| bd.to_string().parse().unwrap_or(0.0)),
                        quality_check_required: m.quality_check_required,
                        batch_size: m.batch_size,
                        tool_requirements: m.tool_requirements,
                    });

                    (manufacturing, None, None)
                }
                JobType::Qa => {
                    let qa_data = qa_job::table
                        .filter(qa_job::job_id.eq(job_id))
                        .filter(qa_job::tenant_id.eq(tenant_id))
                        .select(QaJob::as_select())
                        .first::<QaJob>(&mut conn)
                        .await
                        .optional()?;

                    let qa = qa_data.map(|q| QaJobData {
                        inspection_type: q.inspection_type,
                        test_procedure_id: q.test_procedure_id,
                        acceptance_criteria: q.acceptance_criteria,
                        sampling_size: q.sampling_size,
                        test_equipment: q.test_equipment,
                        calibration_required: q.calibration_required,
                        environmental_conditions: q.environmental_conditions,
                    });

                    (None, qa, None)
                }
                JobType::Service => {
                    let service_data = service_job::table
                        .filter(service_job::job_id.eq(job_id))
                        .filter(service_job::tenant_id.eq(tenant_id))
                        .select(ServiceJob::as_select())
                        .first::<ServiceJob>(&mut conn)
                        .await
                        .optional()?;

                    let service = service_data.map(|s| ServiceJobData {
                        service_type: s.service_type,
                        location: s.location,
                        equipment_serial_number: s.equipment_serial_number,
                        maintenance_type: s.maintenance_type,
                        parts_required: s.parts_required,
                        safety_requirements: s.safety_requirements,
                        travel_time_hours: s
                            .travel_time_hours
                            .map(|bd| bd.to_string().parse().unwrap_or(0.0)),
                    });

                    (None, None, service)
                }
            };

            Ok(Some(JobResponse {
                id: job.id,
                job_number: job.job_number,
                item_id: job.item_id,
                quantity: job.quantity,
                assigned_person_id: job.assigned_person_id,
                supervisor_id: job.supervisor_id,
                customer_id: job.customer_id,
                job_type,
                priority: JobPriority::try_from(
                    job.priority.unwrap_or_else(|| "normal".to_string()),
                )
                .unwrap_or(JobPriority::Normal),
                start_date: job.start_date,
                end_date: job.end_date,
                due_date: job.due_date,
                status: JobStatus::try_from(job.status).unwrap_or(JobStatus::Pending),
                comments: job.comments,
                materials_consumed: job.materials_consumed,
                labor_hours: job
                    .labor_hours
                    .map(|bd| bd.to_string().parse().unwrap_or(0.0)),
                metadata: job.metadata,
                created_at: job.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: job.updated_at.unwrap_or_else(|| Utc::now()),
                manufacturing,
                qa,
                service,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_job(&self, tenant_id: Uuid, job_id: Uuid) -> Result<()> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        // Delete job (cascades will handle the rest)
        diesel::delete(
            jobs::table
                .filter(jobs::id.eq(job_id))
                .filter(jobs::tenant_id.eq(tenant_id)),
        )
        .execute(&mut conn)
        .await?;

        Ok(())
    }

    pub async fn list_jobs(
        &self,
        tenant_id: Uuid,
        job_type: Option<JobType>,
        status: Option<JobStatus>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<JobResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let mut query = jobs::table
            .filter(jobs::tenant_id.eq(tenant_id))
            .into_boxed();

        // Filter by job type if specified
        if let Some(job_type_filter) = &job_type {
            query = query.filter(jobs::job_type.eq(job_type_filter.to_string()));
        }

        // Filter by status if specified
        if let Some(status_filter) = &status {
            query = query.filter(jobs::status.eq(status_filter.to_string()));
        }

        // Apply pagination
        if let Some(limit_val) = limit {
            query = query.limit(limit_val as i64);
        }
        if let Some(offset_val) = offset {
            query = query.offset(offset_val as i64);
        }

        let jobs_data = query
            .select(Job::as_select())
            .load::<Job>(&mut conn)
            .await?;

        let mut job_responses = Vec::new();
        for job in jobs_data {
            if let Some(job_response) = self.get_job_by_id(tenant_id, job.id).await? {
                job_responses.push(job_response);
            }
        }

        Ok(job_responses)
    }

    // Update method simplified for brevity
    pub async fn update_job(
        &self,
        tenant_id: Uuid,
        job_id: Uuid,
        _request: UpdateJobRequest,
    ) -> Result<JobResponse> {
        // For now, just return the existing job
        // In a real implementation, this would update the job fields
        self.get_job_by_id(tenant_id, job_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Job not found"))
    }

    // Type-specific implementations (simplified for brevity)

    pub async fn list_manufacturing_jobs(
        &self,
        tenant_id: Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<ManufacturingJobResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let mut query = manufacturing_job::table
            .inner_join(jobs::table.on(manufacturing_job::job_id.eq(jobs::id)))
            .filter(manufacturing_job::tenant_id.eq(tenant_id))
            .into_boxed();

        // Apply pagination
        if let Some(limit_val) = limit {
            query = query.limit(limit_val as i64);
        }
        if let Some(offset_val) = offset {
            query = query.offset(offset_val as i64);
        }

        let results = query
            .select((ManufacturingJob::as_select(), Job::as_select()))
            .load::<(ManufacturingJob, Job)>(&mut conn)
            .await?;

        let manufacturing_jobs = results
            .into_iter()
            .map(|(manufacturing, job)| ManufacturingJobResponse {
                id: job.id,
                job_number: job.job_number,
                quantity: job.quantity,
                assigned_person_id: job.assigned_person_id,
                supervisor_id: job.supervisor_id,
                customer_id: job.customer_id,
                priority: JobPriority::try_from(
                    job.priority.unwrap_or_else(|| "normal".to_string()),
                )
                .unwrap_or(JobPriority::Normal),
                start_date: job.start_date,
                end_date: job.end_date,
                due_date: job.due_date,
                status: JobStatus::try_from(job.status).unwrap_or(JobStatus::Pending),
                work_order_number: manufacturing.work_order_number,
                production_line: manufacturing.production_line,
                machine_id: manufacturing.machine_id,
                setup_time_hours: manufacturing
                    .setup_time_hours
                    .map(|bd| bd.to_string().parse().unwrap_or(0.0)),
                cycle_time_minutes: manufacturing
                    .cycle_time_minutes
                    .map(|bd| bd.to_string().parse().unwrap_or(0.0)),
                quality_check_required: manufacturing.quality_check_required,
                batch_size: manufacturing.batch_size,
                tool_requirements: manufacturing.tool_requirements,
                created_at: job.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: job.updated_at.unwrap_or_else(|| Utc::now()),
            })
            .collect();

        Ok(manufacturing_jobs)
    }

    pub async fn get_manufacturing_job_by_id(
        &self,
        tenant_id: Uuid,
        job_id: Uuid,
    ) -> Result<Option<ManufacturingJobResponse>> {
        // Simplified implementation
        let jobs = self.list_manufacturing_jobs(tenant_id, None, None).await?;
        Ok(jobs.into_iter().find(|j| j.id == job_id))
    }

    pub async fn list_qa_jobs(
        &self,
        tenant_id: Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<QaJobResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let mut query = qa_job::table
            .inner_join(jobs::table.on(qa_job::job_id.eq(jobs::id)))
            .filter(qa_job::tenant_id.eq(tenant_id))
            .into_boxed();

        // Apply pagination
        if let Some(limit_val) = limit {
            query = query.limit(limit_val as i64);
        }
        if let Some(offset_val) = offset {
            query = query.offset(offset_val as i64);
        }

        let results = query
            .select((QaJob::as_select(), Job::as_select()))
            .load::<(QaJob, Job)>(&mut conn)
            .await?;

        let qa_jobs = results
            .into_iter()
            .map(|(qa, job)| QaJobResponse {
                id: job.id,
                job_number: job.job_number,
                quantity: job.quantity,
                assigned_person_id: job.assigned_person_id,
                supervisor_id: job.supervisor_id,
                customer_id: job.customer_id,
                priority: JobPriority::try_from(
                    job.priority.unwrap_or_else(|| "normal".to_string()),
                )
                .unwrap_or(JobPriority::Normal),
                start_date: job.start_date,
                end_date: job.end_date,
                due_date: job.due_date,
                status: JobStatus::try_from(job.status).unwrap_or(JobStatus::Pending),
                inspection_type: qa.inspection_type,
                test_procedure_id: qa.test_procedure_id,
                acceptance_criteria: qa.acceptance_criteria,
                sampling_size: qa.sampling_size,
                test_equipment: qa.test_equipment,
                calibration_required: qa.calibration_required,
                environmental_conditions: qa.environmental_conditions,
                created_at: job.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: job.updated_at.unwrap_or_else(|| Utc::now()),
            })
            .collect();

        Ok(qa_jobs)
    }

    pub async fn get_qa_job_by_id(
        &self,
        tenant_id: Uuid,
        job_id: Uuid,
    ) -> Result<Option<QaJobResponse>> {
        // Simplified implementation
        let jobs = self.list_qa_jobs(tenant_id, None, None).await?;
        Ok(jobs.into_iter().find(|j| j.id == job_id))
    }

    pub async fn list_service_jobs(
        &self,
        tenant_id: Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<ServiceJobResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let mut query = service_job::table
            .inner_join(jobs::table.on(service_job::job_id.eq(jobs::id)))
            .filter(service_job::tenant_id.eq(tenant_id))
            .into_boxed();

        // Apply pagination
        if let Some(limit_val) = limit {
            query = query.limit(limit_val as i64);
        }
        if let Some(offset_val) = offset {
            query = query.offset(offset_val as i64);
        }

        let results = query
            .select((ServiceJob::as_select(), Job::as_select()))
            .load::<(ServiceJob, Job)>(&mut conn)
            .await?;

        let service_jobs = results
            .into_iter()
            .map(|(service, job)| ServiceJobResponse {
                id: job.id,
                job_number: job.job_number,
                quantity: job.quantity,
                assigned_person_id: job.assigned_person_id,
                supervisor_id: job.supervisor_id,
                customer_id: job.customer_id,
                priority: JobPriority::try_from(
                    job.priority.unwrap_or_else(|| "normal".to_string()),
                )
                .unwrap_or(JobPriority::Normal),
                start_date: job.start_date,
                end_date: job.end_date,
                due_date: job.due_date,
                status: JobStatus::try_from(job.status).unwrap_or(JobStatus::Pending),
                service_type: service.service_type,
                location: service.location,
                equipment_serial_number: service.equipment_serial_number,
                maintenance_type: service.maintenance_type,
                parts_required: service.parts_required,
                safety_requirements: service.safety_requirements,
                travel_time_hours: service
                    .travel_time_hours
                    .map(|bd| bd.to_string().parse().unwrap_or(0.0)),
                created_at: job.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: job.updated_at.unwrap_or_else(|| Utc::now()),
            })
            .collect();

        Ok(service_jobs)
    }

    pub async fn get_service_job_by_id(
        &self,
        tenant_id: Uuid,
        job_id: Uuid,
    ) -> Result<Option<ServiceJobResponse>> {
        // Simplified implementation
        let jobs = self.list_service_jobs(tenant_id, None, None).await?;
        Ok(jobs.into_iter().find(|j| j.id == job_id))
    }
}
