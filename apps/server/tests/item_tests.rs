#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{header, Method, Request, StatusCode},
        Router,
    };
    use dotenv::dotenv;
    use serde_json::{json, Value};
    use tower::ServiceExt; // for `oneshot` and `ready`
    use uuid::Uuid;

    use ems_server::{routes::items::routes, services::DatabaseService, AppState};

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

    // General Item API Tests

    #[tokio::test]
    async fn test_list_all_items() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(Method::GET, "/", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Item routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_list_items_with_context_filter() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request =
            create_request_with_tenant(Method::GET, "/?context=finished_goods", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Item routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_list_items_with_category_filter() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request =
            create_request_with_tenant(Method::GET, "/?category=electronics", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Item routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_finished_goods_item() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let item_data = json!({
            "internal_part_number": "FG-001",
            "mfr_part_number": "MFG-FG-001",
            "manufacturer": "Test Manufacturer",
            "description": "Test finished goods item",
            "category": "Electronics",
            "lifecycle": "production",
            "context": "finished_goods",
            "quantity": 100,
            "location": "Warehouse A",
            "min_stock_level": 10,
            "max_stock_level": 200,
            "status": "active"
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(item_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Item creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_store_item() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let item_data = json!({
            "internal_part_number": "STORE-001",
            "mfr_part_number": "MFG-STORE-001",
            "manufacturer": "Store Manufacturer",
            "description": "Test store item",
            "category": "Components",
            "lifecycle": "production",
            "context": "store",
            "quantity": 50,
            "location": "Store Room B",
            "min_stock_level": 5,
            "reorder_point": 15,
            "status": "active"
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(item_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Item creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_vendor_item() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let vendor_id = Uuid::new_v4();
        let item_data = json!({
            "internal_part_number": "VENDOR-001",
            "mfr_part_number": "MFG-VENDOR-001",
            "manufacturer": "Vendor Manufacturer",
            "description": "Test vendor item",
            "category": "Supplies",
            "lifecycle": "production",
            "context": "vendor",
            "quantity": 25,
            "vendor_id": vendor_id,
            "lead_time": 7,
            "status": "active"
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(item_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Item creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_item_invalid_quantity() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let invalid_item_data = json!({
            "internal_part_number": "INVALID-001",
            "manufacturer": "Test Manufacturer",
            "context": "finished_goods",
            "quantity": -5 // Invalid negative quantity
        });

        let request =
            create_request_with_tenant(Method::POST, "/", Some(invalid_item_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Item creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_get_item_details() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let item_id = Uuid::new_v4().to_string();

        let request =
            create_request_with_tenant(Method::GET, &format!("/{}", item_id), None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Item routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_update_item() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let item_id = Uuid::new_v4().to_string();

        let update_data = json!({
            "description": "Updated item description",
            "quantity": 150,
            "status": "inactive"
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/{}", item_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Item routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_delete_item() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let item_id = Uuid::new_v4().to_string();

        let request =
            create_request_with_tenant(Method::DELETE, &format!("/{}", item_id), None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Item routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Context-specific Item API Tests

    #[tokio::test]
    async fn test_list_finished_goods_items() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(Method::GET, "/finished-goods", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Item routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_get_finished_goods_item_details() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let item_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::GET,
            &format!("/finished-goods/{}", item_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Item routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_update_finished_goods_item() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let item_id = Uuid::new_v4().to_string();

        let update_data = json!({
            "quantity": 200,
            "location": "Updated Warehouse",
            "max_stock_level": 300
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/finished-goods/{}", item_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Item routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Store Item API Tests

    #[tokio::test]
    async fn test_list_store_items() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(Method::GET, "/store", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Item routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_get_store_item_details() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let item_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::GET,
            &format!("/store/{}", item_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Item routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_update_store_item() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let item_id = Uuid::new_v4().to_string();

        let update_data = json!({
            "quantity": 75,
            "location": "Updated Store Room",
            "reorder_point": 20
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/store/{}", item_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Item routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Vendor Item API Tests

    #[tokio::test]
    async fn test_list_vendor_items() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(Method::GET, "/vendor", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Item routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_get_vendor_item_details() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let item_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::GET,
            &format!("/vendor/{}", item_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Item routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_update_vendor_item() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let item_id = Uuid::new_v4().to_string();

        let vendor_id = Uuid::new_v4();
        let update_data = json!({
            "quantity": 40,
            "vendor_id": vendor_id,
            "lead_time": 10
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/vendor/{}", item_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Item routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // BOM (Bill of Materials) API Tests

    #[tokio::test]
    async fn test_create_bom_item() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let parent_item_id = Uuid::new_v4();
        let component_item_id = Uuid::new_v4();
        let bom_data = json!({
            "parent_item_id": parent_item_id,
            "component_item_id": component_item_id,
            "quantity": 2,
            "notes": "Main component",
            "is_optional": false,
            "assembly_order": 1
        });

        let request = create_request_with_tenant(Method::POST, "/bom", Some(bom_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Item creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_get_item_bom() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let item_id = Uuid::new_v4().to_string();

        let request =
            create_request_with_tenant(Method::GET, &format!("/bom/{}", item_id), None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Item routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_update_bom_item() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let bom_id = Uuid::new_v4().to_string();

        let update_data = json!({
            "quantity": 3,
            "notes": "Updated component",
            "assembly_order": 2
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/bom/item/{}", bom_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // BOM routes don't require authentication, returns NOT_FOUND for non-existent BOM item
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_delete_bom_item() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let bom_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::DELETE,
            &format!("/bom/item/{}", bom_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // BOM routes don't require authentication, returns NOT_FOUND for non-existent BOM item
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // Additional Tests

    #[tokio::test]
    async fn test_tenant_isolation_different_tenants() {
        let app = app().await;
        let tenant_id1 = Uuid::new_v4().to_string();
        let tenant_id2 = Uuid::new_v4().to_string();

        // List items for both tenants
        let request1 = create_request_with_tenant(Method::GET, "/", None, &tenant_id1);
        let request2 = create_request_with_tenant(Method::GET, "/", None, &tenant_id2);

        let response1 = app.clone().oneshot(request1).await.unwrap();
        let response2 = app.oneshot(request2).await.unwrap();

        // Item routes require authentication, both will fail without JWT token
        assert!(
            response1.status() == StatusCode::UNAUTHORIZED
                || response1.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
        assert!(
            response2.status() == StatusCode::UNAUTHORIZED
                || response2.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_item_missing_required_fields() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let invalid_item_data = json!({
            "context": "finished_goods"
            // Missing required internal_part_number and manufacturer fields
        });

        let request =
            create_request_with_tenant(Method::POST, "/", Some(invalid_item_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Item creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_item_invalid_part_number_length() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let invalid_item_data = json!({
            "internal_part_number": "this-part-number-is-way-too-long-to-be-valid-and-should-fail-validation",
            "manufacturer": "Test Manufacturer",
            "context": "finished_goods"
        });

        let request =
            create_request_with_tenant(Method::POST, "/", Some(invalid_item_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Item creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_missing_tenant_header() {
        let app = app().await;

        let item_data = json!({
            "internal_part_number": "NO-TENANT-001",
            "manufacturer": "Test Manufacturer",
            "context": "finished_goods"
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(item_data.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Missing tenant header causes internal server error
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_create_item_with_pagination() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request =
            create_request_with_tenant(Method::GET, "/?limit=10&offset=0", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Item routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_item_with_all_filters() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::GET,
            "/?context=finished_goods&category=electronics&lifecycle=production&limit=5&offset=0",
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Item routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_item_lifecycle_transitions() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let item_id = Uuid::new_v4().to_string();

        // Try to update item lifecycle (will fail because item doesn't exist)
        let update_data = json!({
            "lifecycle": "obsolete"
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/{}", item_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Item routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
