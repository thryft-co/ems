use anyhow::Result;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl, SimpleAsyncConnection};
use uuid::Uuid;

use crate::models::{
    CreateOrderIdResponse, CreateOrderRequest, CustomerOrderResponse, DistributorOrderResponse,
    ExternalEntityType, NewOrder, NewOrderHistory, NewOrderItem, Order, OrderHistory, OrderItem,
    OrderItemResponse, OrderResponse, OrderStatus, OrderType, PurchaseOrderResponse,
    UpdateOrderRequest,
};
use crate::schema::*;
use crate::repositories::DatabaseService;

pub struct OrderService {
    database: DatabaseService,
}

impl OrderService {
    pub fn new(database: DatabaseService) -> Self {
        Self { database }
    }

    // General Order API methods

    pub async fn create_order(
        &self,
        tenant_id: Uuid,
        request: CreateOrderRequest,
    ) -> Result<CreateOrderIdResponse> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let order_id = conn
            .transaction::<_, diesel::result::Error, _>(|conn| {
                Box::pin(async move {
                    // Create order record
                    let new_order = NewOrder {
                        tenant_id,
                        order_number: request.order_number.clone(),
                        order_type: request.order_type.to_string(),
                        external_entity_id: request.external_entity_id,
                        external_entity_type: request.external_entity_type.to_string(),
                        order_date: request.order_date,
                        total_amount: request.total_amount,
                        status: request
                            .status
                            .clone()
                            .unwrap_or(OrderStatus::Draft)
                            .to_string(),
                        created_by_id: request.created_by_id,
                        notes: request.notes.clone(),
                        metadata: request.metadata.clone(),
                    };

                    let order: Order = diesel::insert_into(orders::table)
                        .values(&new_order)
                        .returning(Order::as_returning())
                        .get_result(conn)
                        .await?;

                    // Create order items
                    for item_request in &request.items {
                        let extended_price = item_request.quantity as f64 * item_request.unit_price;

                        let new_item = NewOrderItem {
                            order_id: order.id,
                            item_id: item_request.item_id,
                            item_name: item_request.item_name.clone(),
                            item_description: item_request.item_description.clone(),
                            quantity: item_request.quantity,
                            unit_price: item_request.unit_price,
                            extended_price: extended_price,
                            notes: item_request.notes.clone(),
                        };

                        diesel::insert_into(order_items::table)
                            .values(&new_item)
                            .execute(conn)
                            .await?;
                    }

                    // Log order creation
                    let order_history = NewOrderHistory {
                        order_id: order.id,
                        tenant_id,
                        person_id: Some(request.created_by_id),
                        action: "create".to_string(),
                        previous_status: None,
                        new_status: Some(order.status.clone()),
                        notes: Some("Order created".to_string()),
                    };

                    diesel::insert_into(order_history::table)
                        .values(&order_history)
                        .execute(conn)
                        .await?;

                    Ok(order.id)
                })
            })
            .await
            .map_err(|e| anyhow::anyhow!("Transaction failed: {}", e))?;

        Ok(CreateOrderIdResponse { id: order_id })
    }

    pub async fn get_order_by_id(
        &self,
        tenant_id: Uuid,
        order_id: Uuid,
    ) -> Result<Option<OrderResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        // Get order
        let order = orders::table
            .filter(orders::id.eq(order_id))
            .filter(orders::tenant_id.eq(tenant_id))
            .select(Order::as_select())
            .first::<Order>(&mut conn)
            .await
            .optional()?;

        if let Some(order) = order {
            // Get order items
            let items = order_items::table
                .filter(order_items::order_id.eq(order_id))
                .select(OrderItem::as_select())
                .load::<OrderItem>(&mut conn)
                .await?;

            let item_responses: Vec<OrderItemResponse> = items
                .into_iter()
                .map(|item| OrderItemResponse {
                    id: item.id,
                    item_id: item.item_id,
                    item_name: item.item_name,
                    item_description: item.item_description,
                    quantity: item.quantity,
                    unit_price: item.unit_price,
                    extended_price: item.extended_price,
                    notes: item.notes,
                    created_at: item.created_at.unwrap_or_else(|| Utc::now()),
                    updated_at: item.updated_at.unwrap_or_else(|| Utc::now()),
                })
                .collect();

            Ok(Some(OrderResponse {
                id: order.id,
                order_number: order.order_number,
                order_type: OrderType::try_from(order.order_type)
                    .map_err(|e| anyhow::anyhow!("Invalid order type: {}", e))?,
                external_entity_id: order.external_entity_id,
                external_entity_type: ExternalEntityType::try_from(order.external_entity_type)
                    .map_err(|e| anyhow::anyhow!("Invalid external entity type: {}", e))?,
                order_date: order.order_date,
                total_amount: order.total_amount,
                status: OrderStatus::try_from(order.status)
                    .map_err(|e| anyhow::anyhow!("Invalid order status: {}", e))?,
                created_by_id: order.created_by_id,
                notes: order.notes,
                metadata: order.metadata,
                created_at: order.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: order.updated_at.unwrap_or_else(|| Utc::now()),
                items: item_responses,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_order(
        &self,
        tenant_id: Uuid,
        order_id: Uuid,
        request: UpdateOrderRequest,
    ) -> Result<OrderResponse> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            Box::pin(async move {
                // Update order fields individually
                if let Some(order_number) = &request.order_number {
                    diesel::update(orders::table.filter(orders::id.eq(order_id)))
                        .set(orders::order_number.eq(order_number))
                        .execute(conn)
                        .await?;
                }
                if let Some(external_entity_id) = request.external_entity_id {
                    diesel::update(orders::table.filter(orders::id.eq(order_id)))
                        .set(orders::external_entity_id.eq(external_entity_id))
                        .execute(conn)
                        .await?;
                }
                if let Some(order_date) = request.order_date {
                    diesel::update(orders::table.filter(orders::id.eq(order_id)))
                        .set(orders::order_date.eq(order_date))
                        .execute(conn)
                        .await?;
                }
                if let Some(total_amount) = request.total_amount {
                    diesel::update(orders::table.filter(orders::id.eq(order_id)))
                        .set(orders::total_amount.eq(total_amount))
                        .execute(conn)
                        .await?;
                }
                if let Some(status) = &request.status {
                    diesel::update(orders::table.filter(orders::id.eq(order_id)))
                        .set(orders::status.eq(status.to_string()))
                        .execute(conn)
                        .await?;
                }
                if let Some(notes) = &request.notes {
                    diesel::update(orders::table.filter(orders::id.eq(order_id)))
                        .set(orders::notes.eq(notes))
                        .execute(conn)
                        .await?;
                }
                if let Some(metadata) = &request.metadata {
                    diesel::update(orders::table.filter(orders::id.eq(order_id)))
                        .set(orders::metadata.eq(metadata))
                        .execute(conn)
                        .await?;
                }

                Ok(())
            })
        })
        .await
        .map_err(|e| anyhow::anyhow!("Update transaction failed: {}", e))?;

        // Return updated order
        self.get_order_by_id(tenant_id, order_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Order not found after update"))
    }

    pub async fn delete_order(&self, tenant_id: Uuid, order_id: Uuid) -> Result<()> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        diesel::delete(
            orders::table
                .filter(orders::id.eq(order_id))
                .filter(orders::tenant_id.eq(tenant_id)),
        )
        .execute(&mut conn)
        .await?;

        Ok(())
    }

    pub async fn list_orders(
        &self,
        tenant_id: Uuid,
        order_type: Option<OrderType>,
        status: Option<OrderStatus>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<OrderResponse>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let mut query = orders::table
            .filter(orders::tenant_id.eq(tenant_id))
            .into_boxed();

        // Filter by order type if specified
        if let Some(order_type) = &order_type {
            query = query.filter(orders::order_type.eq(order_type.to_string()));
        }

        // Filter by status if specified
        if let Some(status) = &status {
            query = query.filter(orders::status.eq(status.to_string()));
        }

        // Apply pagination
        if let Some(limit_val) = limit {
            query = query.limit(limit_val as i64);
        }
        if let Some(offset_val) = offset {
            query = query.offset(offset_val as i64);
        }

        let orders = query
            .select(Order::as_select())
            .load::<Order>(&mut conn)
            .await?;

        let mut order_responses = Vec::new();
        for order in orders {
            // Get order items for each order
            let items = order_items::table
                .filter(order_items::order_id.eq(order.id))
                .select(OrderItem::as_select())
                .load::<OrderItem>(&mut conn)
                .await?;

            let item_responses: Vec<OrderItemResponse> = items
                .into_iter()
                .map(|item| OrderItemResponse {
                    id: item.id,
                    item_id: item.item_id,
                    item_name: item.item_name,
                    item_description: item.item_description,
                    quantity: item.quantity,
                    unit_price: item.unit_price,
                    extended_price: item.extended_price,
                    notes: item.notes,
                    created_at: item.created_at.unwrap_or_else(|| Utc::now()),
                    updated_at: item.updated_at.unwrap_or_else(|| Utc::now()),
                })
                .collect();

            order_responses.push(OrderResponse {
                id: order.id,
                order_number: order.order_number,
                order_type: OrderType::try_from(order.order_type)
                    .map_err(|e| anyhow::anyhow!("Invalid order type: {}", e))?,
                external_entity_id: order.external_entity_id,
                external_entity_type: ExternalEntityType::try_from(order.external_entity_type)
                    .map_err(|e| anyhow::anyhow!("Invalid external entity type: {}", e))?,
                order_date: order.order_date,
                total_amount: order.total_amount,
                status: OrderStatus::try_from(order.status)
                    .map_err(|e| anyhow::anyhow!("Invalid order status: {}", e))?,
                created_by_id: order.created_by_id,
                notes: order.notes,
                metadata: order.metadata,
                created_at: order.created_at.unwrap_or_else(|| Utc::now()),
                updated_at: order.updated_at.unwrap_or_else(|| Utc::now()),
                items: item_responses,
            });
        }

        Ok(order_responses)
    }

    // Type-specific order methods

    pub async fn list_purchase_orders(
        &self,
        tenant_id: Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<PurchaseOrderResponse>> {
        let orders = self
            .list_orders(
                tenant_id,
                Some(OrderType::PurchaseOrder),
                None,
                limit,
                offset,
            )
            .await?;

        let purchase_orders = orders
            .into_iter()
            .map(|order| PurchaseOrderResponse {
                id: order.id,
                order_number: order.order_number,
                vendor_id: order.external_entity_id,
                order_date: order.order_date,
                total_amount: order.total_amount,
                status: order.status,
                created_by_id: order.created_by_id,
                notes: order.notes,
                created_at: order.created_at,
                updated_at: order.updated_at,
                items: order.items,
            })
            .collect();

        Ok(purchase_orders)
    }

    pub async fn get_purchase_order_by_id(
        &self,
        tenant_id: Uuid,
        order_id: Uuid,
    ) -> Result<Option<PurchaseOrderResponse>> {
        if let Some(order) = self.get_order_by_id(tenant_id, order_id).await? {
            // Verify it's a purchase order
            if order.order_type == OrderType::PurchaseOrder {
                Ok(Some(PurchaseOrderResponse {
                    id: order.id,
                    order_number: order.order_number,
                    vendor_id: order.external_entity_id,
                    order_date: order.order_date,
                    total_amount: order.total_amount,
                    status: order.status,
                    created_by_id: order.created_by_id,
                    notes: order.notes,
                    created_at: order.created_at,
                    updated_at: order.updated_at,
                    items: order.items,
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub async fn list_customer_orders(
        &self,
        tenant_id: Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<CustomerOrderResponse>> {
        let orders = self
            .list_orders(
                tenant_id,
                Some(OrderType::CustomerOrder),
                None,
                limit,
                offset,
            )
            .await?;

        let customer_orders = orders
            .into_iter()
            .map(|order| CustomerOrderResponse {
                id: order.id,
                order_number: order.order_number,
                customer_id: order.external_entity_id,
                order_date: order.order_date,
                total_amount: order.total_amount,
                status: order.status,
                created_by_id: order.created_by_id,
                notes: order.notes,
                created_at: order.created_at,
                updated_at: order.updated_at,
                items: order.items,
            })
            .collect();

        Ok(customer_orders)
    }

    pub async fn get_customer_order_by_id(
        &self,
        tenant_id: Uuid,
        order_id: Uuid,
    ) -> Result<Option<CustomerOrderResponse>> {
        if let Some(order) = self.get_order_by_id(tenant_id, order_id).await? {
            // Verify it's a customer order
            if order.order_type == OrderType::CustomerOrder {
                Ok(Some(CustomerOrderResponse {
                    id: order.id,
                    order_number: order.order_number,
                    customer_id: order.external_entity_id,
                    order_date: order.order_date,
                    total_amount: order.total_amount,
                    status: order.status,
                    created_by_id: order.created_by_id,
                    notes: order.notes,
                    created_at: order.created_at,
                    updated_at: order.updated_at,
                    items: order.items,
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub async fn list_distributor_orders(
        &self,
        tenant_id: Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<DistributorOrderResponse>> {
        let orders = self
            .list_orders(
                tenant_id,
                Some(OrderType::DistributorOrder),
                None,
                limit,
                offset,
            )
            .await?;

        let distributor_orders = orders
            .into_iter()
            .map(|order| DistributorOrderResponse {
                id: order.id,
                order_number: order.order_number,
                distributor_id: order.external_entity_id,
                order_date: order.order_date,
                total_amount: order.total_amount,
                status: order.status,
                created_by_id: order.created_by_id,
                notes: order.notes,
                created_at: order.created_at,
                updated_at: order.updated_at,
                items: order.items,
            })
            .collect();

        Ok(distributor_orders)
    }

    pub async fn get_distributor_order_by_id(
        &self,
        tenant_id: Uuid,
        order_id: Uuid,
    ) -> Result<Option<DistributorOrderResponse>> {
        if let Some(order) = self.get_order_by_id(tenant_id, order_id).await? {
            // Verify it's a distributor order
            if order.order_type == OrderType::DistributorOrder {
                Ok(Some(DistributorOrderResponse {
                    id: order.id,
                    order_number: order.order_number,
                    distributor_id: order.external_entity_id,
                    order_date: order.order_date,
                    total_amount: order.total_amount,
                    status: order.status,
                    created_by_id: order.created_by_id,
                    notes: order.notes,
                    created_at: order.created_at,
                    updated_at: order.updated_at,
                    items: order.items,
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub async fn get_order_history(
        &self,
        tenant_id: Uuid,
        order_id: Uuid,
    ) -> Result<Vec<OrderHistory>> {
        let mut conn = self.database.get_connection().await?;

        // Set tenant context for RLS
        conn.batch_execute(&format!("SET app.current_tenant_id = '{}'", tenant_id))
            .await?;

        let history = order_history::table
            .filter(order_history::order_id.eq(order_id))
            .filter(order_history::tenant_id.eq(tenant_id))
            .order(order_history::created_at.desc())
            .select(OrderHistory::as_select())
            .load::<OrderHistory>(&mut conn)
            .await?;

        Ok(history)
    }
}
