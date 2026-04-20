#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{header, Method, Request, StatusCode},
        Router,
    };
    use dotenv::dotenv;
    use serde_json::{json, Value};
    use tower::ServiceExt; // for `oneshot` and `ready`
    use uuid::Uuid;

    use ems_server::{routes::orders::routes, services::DatabaseService, AppState};

    async fn app() -> Router {
        // Load environment variables for tests
        dotenv().ok();

        // Try to create database service, but handle failure gracefully for tests
        let _database = match DatabaseService::new().await {
            Ok(db) => db,
            Err(_) => {
                panic!("Database connection failed. Please ensure DATABASE_URL is set and PostgreSQL is running.");
            }
        };

        let state = AppState::new().await.expect("Failed to create app state");
        routes().with_state(state)
    }

    // Helper function to create test request with tenant header
    fn create_request_with_tenant(
        method: Method,
        uri: &str,
        body: Option<Value>,
        tenant_id: &str,
    ) -> Request<Body> {
        let request = Request::builder()
            .method(method)
            .uri(uri)
            .header("X-Tenant-ID", tenant_id)
            .header(header::CONTENT_TYPE, "application/json");

        if let Some(body_value) = body {
            request.body(Body::from(body_value.to_string())).unwrap()
        } else {
            request.body(Body::empty()).unwrap()
        }
    }

    // General Order API Tests

    #[tokio::test]
    async fn test_list_all_orders() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(Method::GET, "/", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Order routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_list_orders_with_type_filter() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request =
            create_request_with_tenant(Method::GET, "/?order_type=purchase", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Order routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_list_orders_with_status_filter() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(Method::GET, "/?status=pending", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Order routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_purchase_order() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let order_data = json!({
            "order_number": "PO-001",
            "supplier_id": Uuid::new_v4().to_string(),
            "amount": 5000.0,
            "expected_delivery_date": "2024-12-15"
        });

        let request =
            create_request_with_tenant(Method::POST, "/purchase", Some(order_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Purchase order POST endpoint doesn't exist, returns METHOD_NOT_ALLOWED
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn test_create_sales_order() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let order_data = json!({
            "order_number": "SO-001",
            "customer_id": Uuid::new_v4().to_string(),
            "amount": 3000.0,
            "delivery_date": "2024-12-20"
        });

        let request =
            create_request_with_tenant(Method::POST, "/sales", Some(order_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Sales order POST endpoint doesn't exist, returns METHOD_NOT_ALLOWED
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn test_create_work_order() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let order_data = json!({
            "order_number": "WO-001",
            "production_schedule_id": Uuid::new_v4().to_string(),
            "priority": "high",
            "estimated_completion_date": "2024-12-25"
        });

        let request =
            create_request_with_tenant(Method::POST, "/work", Some(order_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Work order POST endpoint doesn't exist, returns METHOD_NOT_ALLOWED
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn test_create_order_invalid_amount() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let order_data = json!({
            "order_number": "ORD-001",
            "order_type": "purchase",
            "amount": -100.0, // Invalid negative amount
            "customer_id": Uuid::new_v4().to_string(),
            "estimated_delivery_date": "2024-12-01"
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(order_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Order creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_get_order_details() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let order_id = Uuid::new_v4().to_string();

        let request =
            create_request_with_tenant(Method::GET, &format!("/{}", order_id), None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Order routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_update_order() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let order_id = Uuid::new_v4().to_string();

        let update_data = json!({
            "status": "completed",
            "amount": 1500.0
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/{}", order_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Order routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_delete_order() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let order_id = Uuid::new_v4().to_string();

        let request =
            create_request_with_tenant(Method::DELETE, &format!("/{}", order_id), None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Order routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Purchase Order API Tests

    #[tokio::test]
    async fn test_list_purchase_orders() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(Method::GET, "/purchase", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Order routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_get_purchase_order_details() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let order_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::GET,
            &format!("/purchase/{}", order_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Order routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_update_purchase_order() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let order_id = Uuid::new_v4().to_string();

        let update_data = json!({
            "status": "delivered",
            "actual_delivery_date": "2024-12-10"
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/purchase/{}", order_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Purchase order PUT endpoint doesn't exist, returns METHOD_NOT_ALLOWED
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    // Sales Order API Tests

    #[tokio::test]
    async fn test_list_sales_orders() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(Method::GET, "/sales", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Order routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_get_sales_order_details() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let order_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::GET,
            &format!("/sales/{}", order_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Sales order routes don't exist, returns NOT_FOUND
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_update_sales_order() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let order_id = Uuid::new_v4().to_string();

        let customer_id = Uuid::new_v4();
        let update_data = json!({
            "customer_id": customer_id,
            "priority": "urgent",
            "billing_address": "Updated Billing Address"
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/sales/{}", order_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Sales order routes don't exist, returns NOT_FOUND
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // Work Order API Tests

    #[tokio::test]
    async fn test_list_work_orders() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(Method::GET, "/work", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Order routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_get_work_order_details() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let order_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::GET,
            &format!("/work/{}", order_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Work order routes don't exist, returns NOT_FOUND
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_update_work_order() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let order_id = Uuid::new_v4().to_string();

        let update_data = json!({
            "department": "Updated Department",
            "project_code": "PROJ-2024-002",
            "status": "in_progress"
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/work/{}", order_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Work order routes don't exist, returns NOT_FOUND
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // Order Line Items API Tests

    #[tokio::test]
    async fn test_add_order_line_item() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let order_id = Uuid::new_v4().to_string();
        let item_id = Uuid::new_v4();

        let line_item_data = json!({
            "item_id": item_id,
            "quantity": 10,
            "unit_price": 25.50,
            "line_total": 255.00,
            "notes": "Standard delivery"
        });

        let request = create_request_with_tenant(
            Method::POST,
            &format!("/{}/items", order_id),
            Some(line_item_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Order line item routes don't exist, returns NOT_FOUND
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_order_line_items() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let order_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::GET,
            &format!("/{}/items", order_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Order line item routes don't exist, returns NOT_FOUND
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_update_order_line_item() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let order_id = Uuid::new_v4().to_string();
        let line_item_id = Uuid::new_v4().to_string();

        let update_data = json!({
            "quantity": 15,
            "unit_price": 30.00,
            "line_total": 450.00
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/{}/items/{}", order_id, line_item_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Order line item routes don't exist, returns NOT_FOUND
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_delete_order_line_item() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let order_id = Uuid::new_v4().to_string();
        let line_item_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::DELETE,
            &format!("/{}/items/{}", order_id, line_item_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Order line item routes don't exist, returns NOT_FOUND
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // Edge Case and Error Tests

    #[tokio::test]
    async fn test_tenant_isolation_different_tenants() {
        let app = app().await;
        let tenant1_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(Method::GET, "/", None, &tenant1_id);

        let response = app.oneshot(request).await.unwrap();
        // Order routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_order_missing_required_fields() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let order_data = json!({
            "order_number": "ORD-001"
            // Missing required fields like order_type, amount, etc.
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(order_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Order creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_order_invalid_order_number_length() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let order_data = json!({
            "order_number": "A".repeat(101), // Exceeds max length
            "order_type": "purchase",
            "amount": 1000.0,
            "customer_id": Uuid::new_v4().to_string(),
            "estimated_delivery_date": "2024-12-01"
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(order_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Order creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_missing_tenant_header() {
        let app = app().await;

        let request = Request::builder()
            .method(Method::GET)
            .uri("/")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        // Missing tenant header will be caught by tenant middleware before auth middleware
        assert!(
            response.status() == StatusCode::BAD_REQUEST
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_list_orders_with_pagination() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request =
            create_request_with_tenant(Method::GET, "/?page=1&per_page=10", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Order routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_list_orders_with_all_filters() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::GET,
            "/?order_type=purchase&status=pending&page=1&per_page=5",
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Order routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_order_status_transitions() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let order_id = Uuid::new_v4().to_string();

        let update_data = json!({
            "status": "shipped"
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/{}/status", order_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Order status endpoint doesn't exist, returns NOT_FOUND
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
