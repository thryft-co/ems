use anyhow::Result;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl, SimpleAsyncConnection};
use uuid::Uuid;

use crate::models::{
    Asset, AssetResponse, AssetSummary, AssetType, AssetTypeResponse, CreateAssetIdResponse,
    CreateAssetRequest, CreateAssetTypeRequest, FirmwareSpecific, FirmwareSpecificResponse,
    NewAsset, NewAssetType, NewFirmwareSpecific, Person, PersonSummary, UpdateAssetRequest,
    UpdateAssetTypeRequest,
};
use crate::schema::*;
use crate::repositories::DatabaseService;

pub struct AssetService {
    database: DatabaseService,
}

impl AssetService {
    pub fn new(database: DatabaseService) -> Self {
        Self { database }
    }

    // Asset Type Management

    pub async fn create_asset_type(&self, request: CreateAssetTypeRequest) -> Result<AssetType> {
        let mut conn = self.database.get_connection().await?;

        let new_asset_type = NewAssetType {
            name: request.name,
            description: request.description,
        };

        let asset_type: AssetType = diesel::insert_into(asset_types::table)
            .values(&new_asset_type)
            .returning(AssetType::as_returning())
            .get_result(&mut conn)
            .await?;

        Ok(asset_type)
    }

    pub async fn get_asset_type_by_id(&self, asset_type_id: Uuid) -> Result<Option<AssetType>> {
        let mut conn = self.database.get_connection().await?;

        let asset_type = asset_types::table
            .filter(asset_types::id.eq(asset_type_id))
            .select(AssetType::as_select())
            .first::<AssetType>(&mut conn)
            .await
            .optional()?;

        Ok(asset_type)
    }

    pub async fn list_asset_types(&self) -> Result<Vec<AssetTypeResponse>> {
        let mut conn = self.database.get_connection().await?;

        let asset_types = asset_types::table
            .select(AssetType::as_select())
            .load::<AssetType>(&mut conn)
            .await?;

        let responses = asset_types
            .into_iter()
            .map(|at| AssetTypeResponse {
                id: at.id,
                name: at.name,
                description: at.description,
                created_at: at.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: at.updated_at.unwrap_or_else(|| Utc::now()),
            })
            .collect();

        Ok(responses)
    }

    pub async fn update_asset_type(
        &self,
        asset_type_id: Uuid,
        request: UpdateAssetTypeRequest,
    ) -> Result<AssetType> {
        let mut conn = self.database.get_connection().await?;

        // Update fields individually to avoid Diesel type issues
        if let Some(name) = &request.name {
            diesel::update(asset_types::table.filter(asset_types::id.eq(asset_type_id)))
                .set(asset_types::name.eq(name))
                .execute(&mut conn)
                .await?;
        }

        if let Some(description) = &request.description {
            diesel::update(asset_types::table.filter(asset_types::id.eq(asset_type_id)))
                .set(asset_types::description.eq(description))
                .execute(&mut conn)
                .await?;
        }

        // Always update the updated_at timestamp
        diesel::update(asset_types::table.filter(asset_types::id.eq(asset_type_id)))
            .set(asset_types::updated_at.eq(Utc::now()))
            .execute(&mut conn)
            .await?;

        // Return the updated asset type
        let asset_type = asset_types::table
            .filter(asset_types::id.eq(asset_type_id))
            .select(AssetType::as_select())
            .first::<AssetType>(&mut conn)
            .await?;

        Ok(asset_type)
    }

    pub async fn delete_asset_type(&self, asset_type_id: Uuid) -> Result<()> {
        let mut conn = self.database.get_connection().await?;

        diesel::delete(asset_types::table.filter(asset_types::id.eq(asset_type_id)))
            .execute(&mut conn)
            .await?;

        Ok(())
    }

    // Asset Management

    pub async fn create_asset(
        &self,
        tenant_id: Uuid,
        created_by_id: Uuid,
        request: CreateAssetRequest,
    ) -> Result<CreateAssetIdResponse> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let asset_id = conn
            .transaction::<_, diesel::result::Error, _>(|conn| {
                Box::pin(async move {
                    // Create asset record
                    let new_asset = NewAsset {
                        tenant_id,
                        item_id: request.item_id,
                        asset_type_id: request.asset_type_id,
                        name: request.name.clone(),
                        version: request.version,
                        description: request.description,
                        file_path: request.file_path,
                        file_size: request.file_size,
                        file_type: request.file_type,
                        checksum: request.checksum,
                        is_active: request.is_active.or(Some(true)),
                        metadata: request.metadata,
                        created_by_id,
                    };

                    let asset: Asset = diesel::insert_into(assets::table)
                        .values(&new_asset)
                        .returning(Asset::as_returning())
                        .get_result(conn)
                        .await?;

                    // If firmware-specific details are provided, create those too
                    if let Some(firmware_details) = request.firmware_details {
                        let new_firmware_specific = NewFirmwareSpecific {
                            asset_id: asset.id,
                            hardware_version: firmware_details.hardware_version,
                            min_hardware_version: firmware_details.min_hardware_version,
                            max_hardware_version: firmware_details.max_hardware_version,
                            release_notes: firmware_details.release_notes,
                            is_beta: firmware_details.is_beta,
                            is_critical: firmware_details.is_critical,
                            requires_manual_update: firmware_details.requires_manual_update,
                        };

                        diesel::insert_into(firmware_specific::table)
                            .values(&new_firmware_specific)
                            .execute(conn)
                            .await?;
                    }

                    Ok(asset.id)
                })
            })
            .await
            .map_err(|e| anyhow::anyhow!("Transaction failed: {}", e))?;

        Ok(CreateAssetIdResponse { id: asset_id })
    }

    pub async fn get_asset_by_id(
        &self,
        tenant_id: Uuid,
        asset_id: Uuid,
    ) -> Result<Option<AssetResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        // Get the asset with related data
        let result = assets::table
            .inner_join(asset_types::table.on(asset_types::id.eq(assets::asset_type_id)))
            .inner_join(person::table.on(person::id.eq(assets::created_by_id)))
            .filter(assets::id.eq(asset_id))
            .filter(assets::tenant_id.eq(tenant_id))
            .select((
                Asset::as_select(),
                AssetType::as_select(),
                Person::as_select(),
            ))
            .first::<(Asset, AssetType, Person)>(&mut conn)
            .await
            .optional()?;

        if let Some((asset, asset_type, person)) = result {
            // Get firmware-specific details if this is a firmware asset
            let firmware_details = if asset_type.name == "firmware" {
                firmware_specific::table
                    .filter(firmware_specific::asset_id.eq(asset.id))
                    .select(FirmwareSpecific::as_select())
                    .first::<FirmwareSpecific>(&mut conn)
                    .await
                    .optional()?
                    .map(|fs| FirmwareSpecificResponse {
                        id: fs.id,
                        hardware_version: fs.hardware_version,
                        min_hardware_version: fs.min_hardware_version,
                        max_hardware_version: fs.max_hardware_version,
                        release_notes: fs.release_notes,
                        is_beta: fs.is_beta.unwrap_or(false),
                        is_critical: fs.is_critical.unwrap_or(false),
                        requires_manual_update: fs.requires_manual_update.unwrap_or(false),
                        created_at: fs.created_at.unwrap_or_else(|| Utc::now()),
                        updated_at: fs.updated_at.unwrap_or_else(|| Utc::now()),
                    })
            } else {
                None
            };

            Ok(Some(AssetResponse {
                id: asset.id,
                tenant_id: asset.tenant_id,
                item_id: asset.item_id,
                asset_type: AssetTypeResponse {
                    id: asset_type.id,
                    name: asset_type.name,
                    description: asset_type.description,
                    created_at: asset_type.created_at.unwrap_or_else(|| Utc::now()),
                    updated_at: asset_type.updated_at.unwrap_or_else(|| Utc::now()),
                },
                name: asset.name,
                version: asset.version,
                description: asset.description,
                file_path: asset.file_path,
                file_size: asset.file_size,
                file_type: asset.file_type,
                checksum: asset.checksum,
                is_active: asset.is_active.unwrap_or(true),
                metadata: asset.metadata,
                created_by: PersonSummary {
                    id: person.id,
                    name: person.name,
                    email: person.email,
                },
                created_at: asset.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: asset.updated_at.unwrap_or_else(|| Utc::now()),
                firmware_details,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_assets(
        &self,
        tenant_id: Uuid,
        asset_type_id: Option<Uuid>,
        item_id: Option<Uuid>,
        is_active: Option<bool>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<AssetSummary>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let mut query = assets::table
            .inner_join(asset_types::table.on(asset_types::id.eq(assets::asset_type_id)))
            .filter(assets::tenant_id.eq(tenant_id))
            .into_boxed();

        // Apply filters
        if let Some(type_id) = asset_type_id {
            query = query.filter(assets::asset_type_id.eq(type_id));
        }

        if let Some(item_filter) = item_id {
            query = query.filter(assets::item_id.eq(item_filter));
        }

        if let Some(active_filter) = is_active {
            query = query.filter(assets::is_active.eq(active_filter));
        }

        // Apply pagination
        if let Some(limit_val) = limit {
            query = query.limit(limit_val as i64);
        }
        if let Some(offset_val) = offset {
            query = query.offset(offset_val as i64);
        }

        let results = query
            .select((Asset::as_select(), AssetType::as_select()))
            .load::<(Asset, AssetType)>(&mut conn)
            .await?;

        let summaries = results
            .into_iter()
            .map(|(asset, asset_type)| AssetSummary {
                id: asset.id,
                name: asset.name,
                version: asset.version,
                asset_type: asset_type.name,
                file_type: asset.file_type,
                is_active: asset.is_active.unwrap_or(true),
                created_at: asset.created_at.unwrap_or_else(|| Utc::now()),
            })
            .collect();

        Ok(summaries)
    }

    pub async fn update_asset(
        &self,
        tenant_id: Uuid,
        asset_id: Uuid,
        request: UpdateAssetRequest,
    ) -> Result<AssetResponse> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            Box::pin(async move {
                // Update asset fields individually
                if let Some(name) = &request.name {
                    diesel::update(assets::table.filter(assets::id.eq(asset_id)))
                        .set(assets::name.eq(name))
                        .execute(conn)
                        .await?;
                }

                if let Some(version) = &request.version {
                    diesel::update(assets::table.filter(assets::id.eq(asset_id)))
                        .set(assets::version.eq(version))
                        .execute(conn)
                        .await?;
                }

                if let Some(description) = &request.description {
                    diesel::update(assets::table.filter(assets::id.eq(asset_id)))
                        .set(assets::description.eq(description))
                        .execute(conn)
                        .await?;
                }

                if let Some(file_path) = &request.file_path {
                    diesel::update(assets::table.filter(assets::id.eq(asset_id)))
                        .set(assets::file_path.eq(file_path))
                        .execute(conn)
                        .await?;
                }

                if let Some(file_size) = request.file_size {
                    diesel::update(assets::table.filter(assets::id.eq(asset_id)))
                        .set(assets::file_size.eq(file_size))
                        .execute(conn)
                        .await?;
                }

                if let Some(file_type) = &request.file_type {
                    diesel::update(assets::table.filter(assets::id.eq(asset_id)))
                        .set(assets::file_type.eq(file_type))
                        .execute(conn)
                        .await?;
                }

                if let Some(checksum) = &request.checksum {
                    diesel::update(assets::table.filter(assets::id.eq(asset_id)))
                        .set(assets::checksum.eq(checksum))
                        .execute(conn)
                        .await?;
                }

                if let Some(is_active) = request.is_active {
                    diesel::update(assets::table.filter(assets::id.eq(asset_id)))
                        .set(assets::is_active.eq(is_active))
                        .execute(conn)
                        .await?;
                }

                if let Some(metadata) = &request.metadata {
                    diesel::update(assets::table.filter(assets::id.eq(asset_id)))
                        .set(assets::metadata.eq(metadata))
                        .execute(conn)
                        .await?;
                }

                // Always update the updated_at timestamp
                diesel::update(assets::table.filter(assets::id.eq(asset_id)))
                    .set(assets::updated_at.eq(Utc::now()))
                    .execute(conn)
                    .await?;

                // Update firmware-specific details if provided
                if let Some(firmware_details) = request.firmware_details {
                    if let Some(hardware_version) = &firmware_details.hardware_version {
                        diesel::update(
                            firmware_specific::table
                                .filter(firmware_specific::asset_id.eq(asset_id)),
                        )
                        .set(firmware_specific::hardware_version.eq(hardware_version))
                        .execute(conn)
                        .await?;
                    }

                    if let Some(min_hardware_version) = &firmware_details.min_hardware_version {
                        diesel::update(
                            firmware_specific::table
                                .filter(firmware_specific::asset_id.eq(asset_id)),
                        )
                        .set(firmware_specific::min_hardware_version.eq(min_hardware_version))
                        .execute(conn)
                        .await?;
                    }

                    if let Some(max_hardware_version) = &firmware_details.max_hardware_version {
                        diesel::update(
                            firmware_specific::table
                                .filter(firmware_specific::asset_id.eq(asset_id)),
                        )
                        .set(firmware_specific::max_hardware_version.eq(max_hardware_version))
                        .execute(conn)
                        .await?;
                    }

                    if let Some(release_notes) = &firmware_details.release_notes {
                        diesel::update(
                            firmware_specific::table
                                .filter(firmware_specific::asset_id.eq(asset_id)),
                        )
                        .set(firmware_specific::release_notes.eq(release_notes))
                        .execute(conn)
                        .await?;
                    }

                    if let Some(is_beta) = firmware_details.is_beta {
                        diesel::update(
                            firmware_specific::table
                                .filter(firmware_specific::asset_id.eq(asset_id)),
                        )
                        .set(firmware_specific::is_beta.eq(is_beta))
                        .execute(conn)
                        .await?;
                    }

                    if let Some(is_critical) = firmware_details.is_critical {
                        diesel::update(
                            firmware_specific::table
                                .filter(firmware_specific::asset_id.eq(asset_id)),
                        )
                        .set(firmware_specific::is_critical.eq(is_critical))
                        .execute(conn)
                        .await?;
                    }

                    if let Some(requires_manual_update) = firmware_details.requires_manual_update {
                        diesel::update(
                            firmware_specific::table
                                .filter(firmware_specific::asset_id.eq(asset_id)),
                        )
                        .set(firmware_specific::requires_manual_update.eq(requires_manual_update))
                        .execute(conn)
                        .await?;
                    }

                    // Always update the updated_at timestamp for firmware
                    diesel::update(
                        firmware_specific::table.filter(firmware_specific::asset_id.eq(asset_id)),
                    )
                    .set(firmware_specific::updated_at.eq(Utc::now()))
                    .execute(conn)
                    .await?;
                }

                Ok(())
            })
        })
        .await
        .map_err(|e| anyhow::anyhow!("Transaction failed: {}", e))?;

        // Return the updated asset
        self.get_asset_by_id(tenant_id, asset_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Asset not found after update"))
    }

    pub async fn delete_asset(&self, tenant_id: Uuid, asset_id: Uuid) -> Result<()> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            Box::pin(async move {
                // Delete firmware-specific records first (if any)
                diesel::delete(
                    firmware_specific::table.filter(firmware_specific::asset_id.eq(asset_id)),
                )
                .execute(conn)
                .await?;

                // Delete the asset
                diesel::delete(
                    assets::table
                        .filter(assets::id.eq(asset_id))
                        .filter(assets::tenant_id.eq(tenant_id)),
                )
                .execute(conn)
                .await?;

                Ok(())
            })
        })
        .await
        .map_err(|e| anyhow::anyhow!("Transaction failed: {}", e))?;

        Ok(())
    }

    // Get assets by item ID
    pub async fn get_assets_by_item_id(
        &self,
        tenant_id: Uuid,
        item_id: Uuid,
    ) -> Result<Vec<AssetSummary>> {
        self.list_assets(tenant_id, None, Some(item_id), None, None, None)
            .await
    }

    // Get assets by type
    pub async fn get_assets_by_type(
        &self,
        tenant_id: Uuid,
        asset_type_id: Uuid,
    ) -> Result<Vec<AssetSummary>> {
        self.list_assets(tenant_id, Some(asset_type_id), None, None, None, None)
            .await
    }
}
