use anyhow::Result;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::{RunQueryDsl, SimpleAsyncConnection};
use uuid::Uuid;

use crate::models::{
    AssetRelationshipType, CreateMachineAssetRelationshipRequest,
    CreateMachineItemRelationshipRequest, CreateMachineJobAssignmentRequest,
    CreateMachineOperatorAssignmentRequest, CreateMachineRequest, HeartbeatRequest,
    ItemRelationshipType, JobAssignmentStatus, Machine, MachineAction, MachineAssetRelationship,
    MachineAssetRelationshipResponse, MachineCreateIdResponse, MachineItemRelationship,
    MachineItemRelationshipResponse, MachineJobAssignment, MachineJobAssignmentResponse,
    MachineOperatorAssignment, MachineOperatorAssignmentResponse, MachineProtocol, MachineResponse,
    MachineStatus, NewMachine, NewMachineAssetRelationship, NewMachineItemRelationship,
    NewMachineJobAssignment, NewMachineOperatorAssignment, OperatorAssignmentType,
    UpdateMachineJobAssignmentRequest, UpdateMachineRequest,
};
use crate::schema::*;
use crate::repositories::DatabaseService;

pub struct MachineService {
    database: DatabaseService,
}

impl MachineService {
    pub fn new(database: DatabaseService) -> Self {
        Self { database }
    }

    // Machine CRUD operations

    pub async fn create_machine(
        &self,
        tenant_id: Uuid,
        request: CreateMachineRequest,
    ) -> Result<MachineCreateIdResponse> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let new_machine = NewMachine {
            tenant_id,
            name: request.name,
            ip: request.ip,
            port: request.port,
            protocol: request.protocol.to_string(),
            status: request.status.unwrap_or(MachineStatus::Offline).to_string(),
            action: request.action.map(|a| a.to_string()),
            payload: request.payload,
            last_heartbeat: None,
            metadata: request.metadata,
        };

        let machine: Machine = diesel::insert_into(machines::table)
            .values(&new_machine)
            .returning(Machine::as_returning())
            .get_result(&mut conn)
            .await?;

        Ok(MachineCreateIdResponse { id: machine.id })
    }

    pub async fn get_machine_by_id(
        &self,
        tenant_id: Uuid,
        machine_id: Uuid,
    ) -> Result<Option<MachineResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let machine = machines::table
            .filter(machines::id.eq(machine_id))
            .filter(machines::tenant_id.eq(tenant_id))
            .select(Machine::as_select())
            .first::<Machine>(&mut conn)
            .await
            .optional()?;

        if let Some(machine) = machine {
            Ok(Some(MachineResponse {
                id: machine.id,
                tenant_id: machine.tenant_id,
                name: machine.name,
                ip: machine.ip,
                port: machine.port,
                protocol: MachineProtocol::try_from(machine.protocol)
                    .unwrap_or(MachineProtocol::Http),
                status: MachineStatus::try_from(machine.status).unwrap_or(MachineStatus::Offline),
                action: machine.action.and_then(|a| MachineAction::try_from(a).ok()),
                payload: machine.payload,
                last_heartbeat: machine.last_heartbeat,
                metadata: machine.metadata,
                created_at: machine.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: machine.updated_at.unwrap_or_else(|| Utc::now()),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_machines(
        &self,
        tenant_id: Uuid,
        status: Option<MachineStatus>,
        protocol: Option<MachineProtocol>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<MachineResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let mut query = machines::table
            .filter(machines::tenant_id.eq(tenant_id))
            .into_boxed();

        if let Some(status_filter) = &status {
            query = query.filter(machines::status.eq(status_filter.to_string()));
        }

        if let Some(protocol_filter) = &protocol {
            query = query.filter(machines::protocol.eq(protocol_filter.to_string()));
        }

        if let Some(limit_val) = limit {
            query = query.limit(limit_val as i64);
        }
        if let Some(offset_val) = offset {
            query = query.offset(offset_val as i64);
        }

        let machines = query
            .select(Machine::as_select())
            .load::<Machine>(&mut conn)
            .await?;

        let mut machine_responses = Vec::new();
        for machine in machines {
            machine_responses.push(MachineResponse {
                id: machine.id,
                tenant_id: machine.tenant_id,
                name: machine.name,
                ip: machine.ip,
                port: machine.port,
                protocol: MachineProtocol::try_from(machine.protocol)
                    .unwrap_or(MachineProtocol::Http),
                status: MachineStatus::try_from(machine.status).unwrap_or(MachineStatus::Offline),
                action: machine.action.and_then(|a| MachineAction::try_from(a).ok()),
                payload: machine.payload,
                last_heartbeat: machine.last_heartbeat,
                metadata: machine.metadata,
                created_at: machine.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: machine.updated_at.unwrap_or_else(|| Utc::now()),
            });
        }

        Ok(machine_responses)
    }

    pub async fn update_machine(
        &self,
        tenant_id: Uuid,
        machine_id: Uuid,
        request: UpdateMachineRequest,
    ) -> Result<MachineResponse> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        // Update fields individually to avoid Diesel type issues
        if let Some(name) = &request.name {
            diesel::update(
                machines::table
                    .filter(machines::id.eq(machine_id))
                    .filter(machines::tenant_id.eq(tenant_id)),
            )
            .set(machines::name.eq(name))
            .execute(&mut conn)
            .await?;
        }

        if let Some(ip) = &request.ip {
            diesel::update(
                machines::table
                    .filter(machines::id.eq(machine_id))
                    .filter(machines::tenant_id.eq(tenant_id)),
            )
            .set(machines::ip.eq(ip))
            .execute(&mut conn)
            .await?;
        }

        if let Some(port) = request.port {
            diesel::update(
                machines::table
                    .filter(machines::id.eq(machine_id))
                    .filter(machines::tenant_id.eq(tenant_id)),
            )
            .set(machines::port.eq(port))
            .execute(&mut conn)
            .await?;
        }

        if let Some(protocol) = &request.protocol {
            diesel::update(
                machines::table
                    .filter(machines::id.eq(machine_id))
                    .filter(machines::tenant_id.eq(tenant_id)),
            )
            .set(machines::protocol.eq(protocol.to_string()))
            .execute(&mut conn)
            .await?;
        }

        if let Some(status) = &request.status {
            diesel::update(
                machines::table
                    .filter(machines::id.eq(machine_id))
                    .filter(machines::tenant_id.eq(tenant_id)),
            )
            .set(machines::status.eq(status.to_string()))
            .execute(&mut conn)
            .await?;
        }

        if let Some(action) = &request.action {
            diesel::update(
                machines::table
                    .filter(machines::id.eq(machine_id))
                    .filter(machines::tenant_id.eq(tenant_id)),
            )
            .set(machines::action.eq(action.to_string()))
            .execute(&mut conn)
            .await?;
        }

        if let Some(payload) = &request.payload {
            diesel::update(
                machines::table
                    .filter(machines::id.eq(machine_id))
                    .filter(machines::tenant_id.eq(tenant_id)),
            )
            .set(machines::payload.eq(payload))
            .execute(&mut conn)
            .await?;
        }

        if let Some(metadata) = &request.metadata {
            diesel::update(
                machines::table
                    .filter(machines::id.eq(machine_id))
                    .filter(machines::tenant_id.eq(tenant_id)),
            )
            .set(machines::metadata.eq(metadata))
            .execute(&mut conn)
            .await?;
        }

        // Return the updated machine
        self.get_machine_by_id(tenant_id, machine_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Machine not found"))
    }

    pub async fn delete_machine(&self, tenant_id: Uuid, machine_id: Uuid) -> Result<()> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        diesel::delete(
            machines::table
                .filter(machines::id.eq(machine_id))
                .filter(machines::tenant_id.eq(tenant_id)),
        )
        .execute(&mut conn)
        .await?;

        Ok(())
    }

    // Heartbeat functionality

    pub async fn update_heartbeat(
        &self,
        tenant_id: Uuid,
        machine_id: Uuid,
        request: HeartbeatRequest,
    ) -> Result<()> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let now = Utc::now();

        diesel::update(
            machines::table
                .filter(machines::id.eq(machine_id))
                .filter(machines::tenant_id.eq(tenant_id)),
        )
        .set((
            machines::status.eq(request.status.to_string()),
            machines::action.eq(request.action.map(|a| a.to_string())),
            machines::payload.eq(request.payload),
            machines::metadata.eq(request.metadata),
            machines::last_heartbeat.eq(now),
        ))
        .execute(&mut conn)
        .await?;

        Ok(())
    }

    // Machine-Item relationship operations

    pub async fn create_machine_item_relationship(
        &self,
        tenant_id: Uuid,
        machine_id: Uuid,
        request: CreateMachineItemRelationshipRequest,
    ) -> Result<Uuid> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let new_relationship = NewMachineItemRelationship {
            machine_id,
            item_id: request.item_id,
            relationship_type: request.relationship_type.to_string(),
            notes: request.notes,
        };

        let relationship: MachineItemRelationship =
            diesel::insert_into(machine_item_relationships::table)
                .values(&new_relationship)
                .returning(MachineItemRelationship::as_returning())
                .get_result(&mut conn)
                .await?;

        Ok(relationship.id)
    }

    pub async fn list_machine_item_relationships(
        &self,
        tenant_id: Uuid,
        machine_id: Uuid,
    ) -> Result<Vec<MachineItemRelationshipResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let relationships = machine_item_relationships::table
            .filter(machine_item_relationships::machine_id.eq(machine_id))
            .select(MachineItemRelationship::as_select())
            .load::<MachineItemRelationship>(&mut conn)
            .await?;

        Ok(relationships
            .into_iter()
            .map(|rel| MachineItemRelationshipResponse {
                id: rel.id,
                machine_id: rel.machine_id,
                item_id: rel.item_id,
                relationship_type: ItemRelationshipType::try_from(rel.relationship_type)
                    .unwrap_or(ItemRelationshipType::Builds),
                notes: rel.notes,
                created_at: rel.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: rel.updated_at.unwrap_or_else(|| Utc::now()),
            })
            .collect())
    }

    pub async fn delete_machine_item_relationship(
        &self,
        tenant_id: Uuid,
        relationship_id: Uuid,
    ) -> Result<()> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        diesel::delete(
            machine_item_relationships::table
                .filter(machine_item_relationships::id.eq(relationship_id)),
        )
        .execute(&mut conn)
        .await?;

        Ok(())
    }

    // Machine-Asset relationship operations

    pub async fn create_machine_asset_relationship(
        &self,
        tenant_id: Uuid,
        machine_id: Uuid,
        request: CreateMachineAssetRelationshipRequest,
    ) -> Result<Uuid> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let new_relationship = NewMachineAssetRelationship {
            machine_id,
            asset_id: request.asset_id,
            relationship_type: request.relationship_type.to_string(),
            notes: request.notes,
        };

        let relationship: MachineAssetRelationship =
            diesel::insert_into(machine_asset_relationships::table)
                .values(&new_relationship)
                .returning(MachineAssetRelationship::as_returning())
                .get_result(&mut conn)
                .await?;

        Ok(relationship.id)
    }

    pub async fn list_machine_asset_relationships(
        &self,
        tenant_id: Uuid,
        machine_id: Uuid,
    ) -> Result<Vec<MachineAssetRelationshipResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let relationships = machine_asset_relationships::table
            .filter(machine_asset_relationships::machine_id.eq(machine_id))
            .select(MachineAssetRelationship::as_select())
            .load::<MachineAssetRelationship>(&mut conn)
            .await?;

        Ok(relationships
            .into_iter()
            .map(|rel| MachineAssetRelationshipResponse {
                id: rel.id,
                machine_id: rel.machine_id,
                asset_id: rel.asset_id,
                relationship_type: AssetRelationshipType::try_from(rel.relationship_type)
                    .unwrap_or(AssetRelationshipType::Firmware),
                notes: rel.notes,
                created_at: rel.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: rel.updated_at.unwrap_or_else(|| Utc::now()),
            })
            .collect())
    }

    pub async fn delete_machine_asset_relationship(
        &self,
        tenant_id: Uuid,
        relationship_id: Uuid,
    ) -> Result<()> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        diesel::delete(
            machine_asset_relationships::table
                .filter(machine_asset_relationships::id.eq(relationship_id)),
        )
        .execute(&mut conn)
        .await?;

        Ok(())
    }

    // Machine-Operator assignment operations

    pub async fn create_machine_operator_assignment(
        &self,
        tenant_id: Uuid,
        machine_id: Uuid,
        request: CreateMachineOperatorAssignmentRequest,
    ) -> Result<Uuid> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let new_assignment = NewMachineOperatorAssignment {
            machine_id,
            person_id: request.person_id,
            assignment_type: request.assignment_type.to_string(),
            notes: request.notes,
        };

        let assignment: MachineOperatorAssignment =
            diesel::insert_into(machine_operator_assignments::table)
                .values(&new_assignment)
                .returning(MachineOperatorAssignment::as_returning())
                .get_result(&mut conn)
                .await?;

        Ok(assignment.id)
    }

    pub async fn list_machine_operator_assignments(
        &self,
        tenant_id: Uuid,
        machine_id: Uuid,
    ) -> Result<Vec<MachineOperatorAssignmentResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let assignments = machine_operator_assignments::table
            .filter(machine_operator_assignments::machine_id.eq(machine_id))
            .select(MachineOperatorAssignment::as_select())
            .load::<MachineOperatorAssignment>(&mut conn)
            .await?;

        Ok(assignments
            .into_iter()
            .map(|assignment| MachineOperatorAssignmentResponse {
                id: assignment.id,
                machine_id: assignment.machine_id,
                person_id: assignment.person_id,
                assignment_type: OperatorAssignmentType::try_from(assignment.assignment_type)
                    .unwrap_or(OperatorAssignmentType::Primary),
                notes: assignment.notes,
                created_at: assignment.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: assignment.updated_at.unwrap_or_else(|| Utc::now()),
            })
            .collect())
    }

    pub async fn delete_machine_operator_assignment(
        &self,
        tenant_id: Uuid,
        assignment_id: Uuid,
    ) -> Result<()> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        diesel::delete(
            machine_operator_assignments::table
                .filter(machine_operator_assignments::id.eq(assignment_id)),
        )
        .execute(&mut conn)
        .await?;

        Ok(())
    }

    // Machine-Job assignment operations

    pub async fn create_machine_job_assignment(
        &self,
        tenant_id: Uuid,
        machine_id: Uuid,
        request: CreateMachineJobAssignmentRequest,
    ) -> Result<Uuid> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let new_assignment = NewMachineJobAssignment {
            machine_id,
            job_id: request.job_id,
            status: request
                .status
                .unwrap_or(JobAssignmentStatus::Pending)
                .to_string(),
            start_time: request.start_time,
            end_time: request.end_time,
            notes: request.notes,
        };

        let assignment: MachineJobAssignment = diesel::insert_into(machine_job_assignments::table)
            .values(&new_assignment)
            .returning(MachineJobAssignment::as_returning())
            .get_result(&mut conn)
            .await?;

        Ok(assignment.id)
    }

    pub async fn list_machine_job_assignments(
        &self,
        tenant_id: Uuid,
        machine_id: Uuid,
    ) -> Result<Vec<MachineJobAssignmentResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let assignments = machine_job_assignments::table
            .filter(machine_job_assignments::machine_id.eq(machine_id))
            .select(MachineJobAssignment::as_select())
            .load::<MachineJobAssignment>(&mut conn)
            .await?;

        Ok(assignments
            .into_iter()
            .map(|assignment| MachineJobAssignmentResponse {
                id: assignment.id,
                machine_id: assignment.machine_id,
                job_id: assignment.job_id,
                status: JobAssignmentStatus::try_from(assignment.status)
                    .unwrap_or(JobAssignmentStatus::Pending),
                start_time: assignment.start_time,
                end_time: assignment.end_time,
                notes: assignment.notes,
                created_at: assignment.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: assignment.updated_at.unwrap_or_else(|| Utc::now()),
            })
            .collect())
    }

    pub async fn update_machine_job_assignment(
        &self,
        tenant_id: Uuid,
        assignment_id: Uuid,
        request: UpdateMachineJobAssignmentRequest,
    ) -> Result<MachineJobAssignmentResponse> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        // Update fields individually
        if let Some(status) = &request.status {
            diesel::update(
                machine_job_assignments::table
                    .filter(machine_job_assignments::id.eq(assignment_id)),
            )
            .set(machine_job_assignments::status.eq(status.to_string()))
            .execute(&mut conn)
            .await?;
        }

        if let Some(start_time) = request.start_time {
            diesel::update(
                machine_job_assignments::table
                    .filter(machine_job_assignments::id.eq(assignment_id)),
            )
            .set(machine_job_assignments::start_time.eq(start_time))
            .execute(&mut conn)
            .await?;
        }

        if let Some(end_time) = request.end_time {
            diesel::update(
                machine_job_assignments::table
                    .filter(machine_job_assignments::id.eq(assignment_id)),
            )
            .set(machine_job_assignments::end_time.eq(end_time))
            .execute(&mut conn)
            .await?;
        }

        if let Some(notes) = &request.notes {
            diesel::update(
                machine_job_assignments::table
                    .filter(machine_job_assignments::id.eq(assignment_id)),
            )
            .set(machine_job_assignments::notes.eq(notes))
            .execute(&mut conn)
            .await?;
        }

        // Return the updated assignment
        let assignment = machine_job_assignments::table
            .filter(machine_job_assignments::id.eq(assignment_id))
            .select(MachineJobAssignment::as_select())
            .first::<MachineJobAssignment>(&mut conn)
            .await?;

        Ok(MachineJobAssignmentResponse {
            id: assignment.id,
            machine_id: assignment.machine_id,
            job_id: assignment.job_id,
            status: JobAssignmentStatus::try_from(assignment.status)
                .unwrap_or(JobAssignmentStatus::Pending),
            start_time: assignment.start_time,
            end_time: assignment.end_time,
            notes: assignment.notes,
            created_at: assignment.created_at.unwrap_or_else(|| Utc::now()),
            updated_at: assignment.updated_at.unwrap_or_else(|| Utc::now()),
        })
    }

    pub async fn delete_machine_job_assignment(
        &self,
        tenant_id: Uuid,
        assignment_id: Uuid,
    ) -> Result<()> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        diesel::delete(
            machine_job_assignments::table.filter(machine_job_assignments::id.eq(assignment_id)),
        )
        .execute(&mut conn)
        .await?;

        Ok(())
    }

    // Utility methods

    pub async fn get_machines_by_item_id(
        &self,
        tenant_id: Uuid,
        item_id: Uuid,
        relationship_type: Option<ItemRelationshipType>,
    ) -> Result<Vec<MachineResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let mut query = machines::table
            .inner_join(
                machine_item_relationships::table
                    .on(machine_item_relationships::machine_id.eq(machines::id)),
            )
            .filter(machine_item_relationships::item_id.eq(item_id))
            .filter(machines::tenant_id.eq(tenant_id))
            .into_boxed();

        if let Some(rel_type) = &relationship_type {
            query = query
                .filter(machine_item_relationships::relationship_type.eq(rel_type.to_string()));
        }

        let machines = query
            .select(Machine::as_select())
            .load::<Machine>(&mut conn)
            .await?;

        Ok(machines
            .into_iter()
            .map(|machine| MachineResponse {
                id: machine.id,
                tenant_id: machine.tenant_id,
                name: machine.name,
                ip: machine.ip,
                port: machine.port,
                protocol: MachineProtocol::try_from(machine.protocol)
                    .unwrap_or(MachineProtocol::Http),
                status: MachineStatus::try_from(machine.status).unwrap_or(MachineStatus::Offline),
                action: machine.action.and_then(|a| MachineAction::try_from(a).ok()),
                payload: machine.payload,
                last_heartbeat: machine.last_heartbeat,
                metadata: machine.metadata,
                created_at: machine.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: machine.updated_at.unwrap_or_else(|| Utc::now()),
            })
            .collect())
    }

    pub async fn get_machines_by_job_id(
        &self,
        tenant_id: Uuid,
        job_id: Uuid,
        status: Option<JobAssignmentStatus>,
    ) -> Result<Vec<MachineResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let mut query = machines::table
            .inner_join(
                machine_job_assignments::table
                    .on(machine_job_assignments::machine_id.eq(machines::id)),
            )
            .filter(machine_job_assignments::job_id.eq(job_id))
            .filter(machines::tenant_id.eq(tenant_id))
            .into_boxed();

        if let Some(status_filter) = &status {
            query = query.filter(machine_job_assignments::status.eq(status_filter.to_string()));
        }

        let machines = query
            .select(Machine::as_select())
            .load::<Machine>(&mut conn)
            .await?;

        Ok(machines
            .into_iter()
            .map(|machine| MachineResponse {
                id: machine.id,
                tenant_id: machine.tenant_id,
                name: machine.name,
                ip: machine.ip,
                port: machine.port,
                protocol: MachineProtocol::try_from(machine.protocol)
                    .unwrap_or(MachineProtocol::Http),
                status: MachineStatus::try_from(machine.status).unwrap_or(MachineStatus::Offline),
                action: machine.action.and_then(|a| MachineAction::try_from(a).ok()),
                payload: machine.payload,
                last_heartbeat: machine.last_heartbeat,
                metadata: machine.metadata,
                created_at: machine.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: machine.updated_at.unwrap_or_else(|| Utc::now()),
            })
            .collect())
    }
}
