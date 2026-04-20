use anyhow::Result;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl, SimpleAsyncConnection};
use uuid::Uuid;

use crate::models::{
    BomItemResponse, CreateBomItemRequest, CreateItemIdResponse, CreateItemRequest,
    FinishedGoodsItemResponse, InventoryItem, Item, ItemBom, ItemContext, ItemLifecycle,
    ItemResponse, ItemStatus, ItemSummary, NewInventoryItem, NewItem, NewItemBom,
    StoreItemResponse, UpdateBomItemRequest, UpdateItemRequest, VendorItemResponse,
};
use crate::schema::*;
use crate::repositories::DatabaseService;

pub struct ItemService {
    database: DatabaseService,
}

impl ItemService {
    pub fn new(database: DatabaseService) -> Self {
        Self { database }
    }

    // General Item API methods

    pub async fn create_item(
        &self,
        tenant_id: Uuid,
        request: CreateItemRequest,
    ) -> Result<CreateItemIdResponse> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let item_id = conn
            .transaction::<_, diesel::result::Error, _>(|conn| {
                Box::pin(async move {
                    // Create item record
                    let new_item = NewItem {
                        internal_part_number: request.internal_part_number.clone(),
                        mfr_part_number: request.mfr_part_number,
                        manufacturer: request.manufacturer,
                        datasheet: request.datasheet,
                        lifecycle: request.lifecycle.map(|l| l.to_string()),
                        description: request.description,
                        category: request.category,
                        metadata: request.metadata,
                        linked_resources: request.linked_resources,
                    };

                    let item: Item = diesel::insert_into(items::table)
                        .values(&new_item)
                        .returning(Item::as_returning())
                        .get_result(conn)
                        .await?;

                    // Create inventory item record
                    let new_inventory_item = NewInventoryItem {
                        item_id: item.id,
                        tenant_id,
                        context: request.context.to_string(),
                        quantity: request.quantity,
                        location: request.location,
                        pricing: request.pricing,
                        lead_time: request.lead_time,
                        min_stock_level: request.min_stock_level,
                        max_stock_level: request.max_stock_level,
                        reorder_point: request.reorder_point,
                        vendor_id: request.vendor_id,
                        last_received_date: request.last_received_date,
                        status: request.status.map(|s| s.to_string()),
                        notes: request.notes,
                        metadata: request.inventory_metadata,
                    };

                    diesel::insert_into(inventory_items::table)
                        .values(&new_inventory_item)
                        .execute(conn)
                        .await?;

                    Ok(item.id)
                })
            })
            .await
            .map_err(|e| anyhow::anyhow!("Transaction failed: {}", e))?;

        Ok(CreateItemIdResponse { id: item_id })
    }

    pub async fn get_item_by_id(
        &self,
        tenant_id: Uuid,
        item_id: Uuid,
        context: ItemContext,
    ) -> Result<Option<ItemResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let result = items::table
            .inner_join(inventory_items::table.on(inventory_items::item_id.eq(items::id)))
            .filter(items::id.eq(item_id))
            .filter(inventory_items::tenant_id.eq(tenant_id))
            .filter(inventory_items::context.eq(context.to_string()))
            .select((Item::as_select(), InventoryItem::as_select()))
            .first::<(Item, InventoryItem)>(&mut conn)
            .await
            .optional()?;

        if let Some((item, inventory)) = result {
            Ok(Some(ItemResponse {
                id: item.id,
                internal_part_number: item.internal_part_number,
                mfr_part_number: item.mfr_part_number,
                manufacturer: item.manufacturer,
                datasheet: item.datasheet,
                lifecycle: ItemLifecycle::try_from(
                    item.lifecycle.unwrap_or_else(|| "production".to_string()),
                )
                .unwrap_or(ItemLifecycle::Production),
                description: item.description,
                category: item.category,
                metadata: item.metadata,
                linked_resources: item.linked_resources,
                created_at: item.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: item.updated_at.unwrap_or_else(|| Utc::now()),
                context: ItemContext::try_from(inventory.context).unwrap_or(ItemContext::Store),
                quantity: inventory.quantity.unwrap_or(0),
                location: inventory.location,
                pricing: inventory.pricing,
                lead_time: inventory.lead_time,
                min_stock_level: inventory.min_stock_level.unwrap_or(0),
                max_stock_level: inventory.max_stock_level,
                reorder_point: inventory.reorder_point,
                vendor_id: inventory.vendor_id,
                last_received_date: inventory.last_received_date,
                status: ItemStatus::try_from(
                    inventory.status.unwrap_or_else(|| "active".to_string()),
                )
                .unwrap_or(ItemStatus::Active),
                notes: inventory.notes,
                inventory_metadata: inventory.metadata,
                inventory_created_at: inventory.created_at.unwrap_or_else(|| Utc::now()),
                inventory_updated_at: inventory.updated_at.unwrap_or_else(|| Utc::now()),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_item(
        &self,
        tenant_id: Uuid,
        item_id: Uuid,
        context: ItemContext,
    ) -> Result<()> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        // Delete inventory item first (due to foreign key constraints)
        diesel::delete(
            inventory_items::table
                .filter(inventory_items::item_id.eq(item_id))
                .filter(inventory_items::tenant_id.eq(tenant_id))
                .filter(inventory_items::context.eq(context.to_string())),
        )
        .execute(&mut conn)
        .await?;

        // Check if there are other inventory items for this item
        let inventory_count: i64 = inventory_items::table
            .filter(inventory_items::item_id.eq(item_id))
            .count()
            .get_result(&mut conn)
            .await?;

        // If no other inventory items exist, delete the base item
        if inventory_count == 0 {
            diesel::delete(items::table.filter(items::id.eq(item_id)))
                .execute(&mut conn)
                .await?;
        }

        Ok(())
    }

    pub async fn list_items(
        &self,
        tenant_id: Uuid,
        context: Option<ItemContext>,
        category: Option<String>,
        lifecycle: Option<ItemLifecycle>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<ItemResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let mut query = items::table
            .inner_join(inventory_items::table.on(inventory_items::item_id.eq(items::id)))
            .filter(inventory_items::tenant_id.eq(tenant_id))
            .into_boxed();

        // Filter by context if specified
        if let Some(context_filter) = &context {
            query = query.filter(inventory_items::context.eq(context_filter.to_string()));
        }

        // Filter by category if specified
        if let Some(category_filter) = &category {
            query = query.filter(items::category.eq(category_filter));
        }

        // Filter by lifecycle if specified
        if let Some(lifecycle_filter) = &lifecycle {
            query = query.filter(items::lifecycle.eq(lifecycle_filter.to_string()));
        }

        // Apply pagination
        if let Some(limit_val) = limit {
            query = query.limit(limit_val as i64);
        }
        if let Some(offset_val) = offset {
            query = query.offset(offset_val as i64);
        }

        let results = query
            .select((Item::as_select(), InventoryItem::as_select()))
            .load::<(Item, InventoryItem)>(&mut conn)
            .await?;

        let mut item_responses = Vec::new();
        for (item, inventory) in results {
            item_responses.push(ItemResponse {
                id: item.id,
                internal_part_number: item.internal_part_number,
                mfr_part_number: item.mfr_part_number,
                manufacturer: item.manufacturer,
                datasheet: item.datasheet,
                lifecycle: ItemLifecycle::try_from(
                    item.lifecycle.unwrap_or_else(|| "production".to_string()),
                )
                .unwrap_or(ItemLifecycle::Production),
                description: item.description,
                category: item.category,
                metadata: item.metadata,
                linked_resources: item.linked_resources,
                created_at: item.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: item.updated_at.unwrap_or_else(|| Utc::now()),
                context: ItemContext::try_from(inventory.context).unwrap_or(ItemContext::Store),
                quantity: inventory.quantity.unwrap_or(0),
                location: inventory.location,
                pricing: inventory.pricing,
                lead_time: inventory.lead_time,
                min_stock_level: inventory.min_stock_level.unwrap_or(0),
                max_stock_level: inventory.max_stock_level,
                reorder_point: inventory.reorder_point,
                vendor_id: inventory.vendor_id,
                last_received_date: inventory.last_received_date,
                status: ItemStatus::try_from(
                    inventory.status.unwrap_or_else(|| "active".to_string()),
                )
                .unwrap_or(ItemStatus::Active),
                notes: inventory.notes,
                inventory_metadata: inventory.metadata,
                inventory_created_at: inventory.created_at.unwrap_or_else(|| Utc::now()),
                inventory_updated_at: inventory.updated_at.unwrap_or_else(|| Utc::now()),
            });
        }

        Ok(item_responses)
    }

    pub async fn update_item(
        &self,
        tenant_id: Uuid,
        item_id: Uuid,
        context: ItemContext,
        _request: UpdateItemRequest,
    ) -> Result<ItemResponse> {
        // For now, just return the existing item
        // In a real implementation, this would update the item fields
        self.get_item_by_id(tenant_id, item_id, context)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Item not found"))
    }

    // Context-specific implementations

    pub async fn list_finished_goods_items(
        &self,
        tenant_id: Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<FinishedGoodsItemResponse>> {
        let items = self
            .list_items(
                tenant_id,
                Some(ItemContext::FinishedGoods),
                None,
                None,
                limit,
                offset,
            )
            .await?;

        Ok(items
            .into_iter()
            .map(|item| FinishedGoodsItemResponse {
                id: item.id,
                internal_part_number: item.internal_part_number,
                mfr_part_number: item.mfr_part_number,
                manufacturer: item.manufacturer,
                description: item.description,
                category: item.category,
                lifecycle: item.lifecycle,
                quantity: item.quantity,
                location: item.location,
                pricing: item.pricing,
                min_stock_level: item.min_stock_level,
                max_stock_level: item.max_stock_level,
                reorder_point: item.reorder_point,
                status: item.status,
                created_at: item.created_at,
                updated_at: item.updated_at,
            })
            .collect())
    }

    pub async fn get_finished_goods_item_by_id(
        &self,
        tenant_id: Uuid,
        item_id: Uuid,
    ) -> Result<Option<FinishedGoodsItemResponse>> {
        if let Some(item) = self
            .get_item_by_id(tenant_id, item_id, ItemContext::FinishedGoods)
            .await?
        {
            Ok(Some(FinishedGoodsItemResponse {
                id: item.id,
                internal_part_number: item.internal_part_number,
                mfr_part_number: item.mfr_part_number,
                manufacturer: item.manufacturer,
                description: item.description,
                category: item.category,
                lifecycle: item.lifecycle,
                quantity: item.quantity,
                location: item.location,
                pricing: item.pricing,
                min_stock_level: item.min_stock_level,
                max_stock_level: item.max_stock_level,
                reorder_point: item.reorder_point,
                status: item.status,
                created_at: item.created_at,
                updated_at: item.updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_store_items(
        &self,
        tenant_id: Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<StoreItemResponse>> {
        let items = self
            .list_items(
                tenant_id,
                Some(ItemContext::Store),
                None,
                None,
                limit,
                offset,
            )
            .await?;

        Ok(items
            .into_iter()
            .map(|item| StoreItemResponse {
                id: item.id,
                internal_part_number: item.internal_part_number,
                mfr_part_number: item.mfr_part_number,
                manufacturer: item.manufacturer,
                description: item.description,
                category: item.category,
                lifecycle: item.lifecycle,
                quantity: item.quantity,
                location: item.location,
                min_stock_level: item.min_stock_level,
                max_stock_level: item.max_stock_level,
                reorder_point: item.reorder_point,
                status: item.status,
                created_at: item.created_at,
                updated_at: item.updated_at,
            })
            .collect())
    }

    pub async fn get_store_item_by_id(
        &self,
        tenant_id: Uuid,
        item_id: Uuid,
    ) -> Result<Option<StoreItemResponse>> {
        if let Some(item) = self
            .get_item_by_id(tenant_id, item_id, ItemContext::Store)
            .await?
        {
            Ok(Some(StoreItemResponse {
                id: item.id,
                internal_part_number: item.internal_part_number,
                mfr_part_number: item.mfr_part_number,
                manufacturer: item.manufacturer,
                description: item.description,
                category: item.category,
                lifecycle: item.lifecycle,
                quantity: item.quantity,
                location: item.location,
                min_stock_level: item.min_stock_level,
                max_stock_level: item.max_stock_level,
                reorder_point: item.reorder_point,
                status: item.status,
                created_at: item.created_at,
                updated_at: item.updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_vendor_items(
        &self,
        tenant_id: Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<VendorItemResponse>> {
        let items = self
            .list_items(
                tenant_id,
                Some(ItemContext::Vendor),
                None,
                None,
                limit,
                offset,
            )
            .await?;

        Ok(items
            .into_iter()
            .map(|item| VendorItemResponse {
                id: item.id,
                internal_part_number: item.internal_part_number,
                mfr_part_number: item.mfr_part_number,
                manufacturer: item.manufacturer,
                description: item.description,
                category: item.category,
                lifecycle: item.lifecycle,
                quantity: item.quantity,
                location: item.location,
                pricing: item.pricing,
                lead_time: item.lead_time,
                vendor_id: item.vendor_id,
                last_received_date: item.last_received_date,
                status: item.status,
                created_at: item.created_at,
                updated_at: item.updated_at,
            })
            .collect())
    }

    pub async fn get_vendor_item_by_id(
        &self,
        tenant_id: Uuid,
        item_id: Uuid,
    ) -> Result<Option<VendorItemResponse>> {
        if let Some(item) = self
            .get_item_by_id(tenant_id, item_id, ItemContext::Vendor)
            .await?
        {
            Ok(Some(VendorItemResponse {
                id: item.id,
                internal_part_number: item.internal_part_number,
                mfr_part_number: item.mfr_part_number,
                manufacturer: item.manufacturer,
                description: item.description,
                category: item.category,
                lifecycle: item.lifecycle,
                quantity: item.quantity,
                location: item.location,
                pricing: item.pricing,
                lead_time: item.lead_time,
                vendor_id: item.vendor_id,
                last_received_date: item.last_received_date,
                status: item.status,
                created_at: item.created_at,
                updated_at: item.updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    // BOM (Bill of Materials) methods

    pub async fn create_bom_item(
        &self,
        tenant_id: Uuid,
        request: CreateBomItemRequest,
    ) -> Result<Uuid> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let new_bom_item = NewItemBom {
            tenant_id,
            parent_item_id: request.parent_item_id,
            component_item_id: request.component_item_id,
            quantity: request.quantity,
            notes: request.notes,
            is_optional: request.is_optional,
            substitutes: request
                .substitutes
                .map(|s| s.into_iter().map(Some).collect()),
            assembly_order: request.assembly_order,
        };

        let bom_item: ItemBom = diesel::insert_into(item_bom::table)
            .values(&new_bom_item)
            .returning(ItemBom::as_returning())
            .get_result(&mut conn)
            .await?;

        Ok(bom_item.id)
    }

    pub async fn get_bom_by_parent_item(
        &self,
        tenant_id: Uuid,
        parent_item_id: Uuid,
    ) -> Result<Vec<BomItemResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        // First, get all BOM entries for the parent item
        let bom_entries = item_bom::table
            .filter(item_bom::tenant_id.eq(tenant_id))
            .filter(item_bom::parent_item_id.eq(parent_item_id))
            .select(ItemBom::as_select())
            .load::<ItemBom>(&mut conn)
            .await?;

        // Get the parent item details
        let parent_item = items::table
            .filter(items::id.eq(parent_item_id))
            .select(Item::as_select())
            .first::<Item>(&mut conn)
            .await?;

        let mut bom_responses = Vec::new();

        // For each BOM entry, get the component item details
        for bom in bom_entries {
            let component_item = items::table
                .filter(items::id.eq(bom.component_item_id))
                .select(Item::as_select())
                .first::<Item>(&mut conn)
                .await?;

            bom_responses.push(BomItemResponse {
                id: bom.id,
                parent_item: ItemSummary {
                    id: parent_item.id,
                    internal_part_number: parent_item.internal_part_number.clone(),
                    mfr_part_number: parent_item.mfr_part_number.clone(),
                    manufacturer: parent_item.manufacturer.clone(),
                    description: parent_item.description.clone(),
                },
                component_item: ItemSummary {
                    id: component_item.id,
                    internal_part_number: component_item.internal_part_number,
                    mfr_part_number: component_item.mfr_part_number,
                    manufacturer: component_item.manufacturer,
                    description: component_item.description,
                },
                quantity: bom.quantity.unwrap_or(1),
                notes: bom.notes,
                is_optional: bom.is_optional.unwrap_or(false),
                substitutes: bom
                    .substitutes
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|id| id)
                    .collect(),
                assembly_order: bom.assembly_order,
                created_at: bom.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: bom.updated_at.unwrap_or_else(|| Utc::now()),
            });
        }

        Ok(bom_responses)
    }

    pub async fn update_bom_item(
        &self,
        tenant_id: Uuid,
        bom_id: Uuid,
        _request: UpdateBomItemRequest,
    ) -> Result<()> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        // For now, just verify the BOM item exists
        let _bom_item = item_bom::table
            .filter(item_bom::id.eq(bom_id))
            .filter(item_bom::tenant_id.eq(tenant_id))
            .select(ItemBom::as_select())
            .first::<ItemBom>(&mut conn)
            .await?;

        // In a real implementation, this would update the BOM item fields
        Ok(())
    }

    pub async fn delete_bom_item(&self, tenant_id: Uuid, bom_id: Uuid) -> Result<()> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        diesel::delete(
            item_bom::table
                .filter(item_bom::id.eq(bom_id))
                .filter(item_bom::tenant_id.eq(tenant_id)),
        )
        .execute(&mut conn)
        .await?;

        Ok(())
    }
}
