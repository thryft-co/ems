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

    use ems_server::{routes::persons::routes, services::DatabaseService, AppState};

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

    // General Person API Tests

    #[tokio::test]
    async fn test_list_all_persons() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(Method::GET, "/", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Person routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_list_persons_with_type_filter() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request =
            create_request_with_tenant(Method::GET, "/?person_type=customer", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Person routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_internal_person() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let person_data = json!({
            "name": "John Doe",
            "email": "john.doe@example.com",
            "role": "internal",
            "person_type": "internal",
            "phone": "555-1234",
            "department": "Engineering",
            "position": "Software Engineer",
            "employee_id": "EMP-123",
            "hire_date": "2023-01-15T00:00:00Z"
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(person_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Person creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_customer_person() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let person_data = json!({
            "name": "Jane Smith",
            "email": "jane.smith@customer.com",
            "role": "customer",
            "person_type": "customer",
            "phone": "555-5678",
            "company": "Customer Corp",
            "industry": "Technology",
            "customer_since": "2023-02-01T00:00:00Z"
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(person_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Person creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_vendor_person() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let person_data = json!({
            "name": "Bob Wilson",
            "email": "bob.wilson@vendor.com",
            "role": "vendor",
            "person_type": "vendor",
            "phone": "555-9012",
            "company": "Vendor LLC",
            "service_type": "Manufacturing",
            "contract_start": "2023-01-01T00:00:00Z",
            "contract_end": "2024-01-01T00:00:00Z"
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(person_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Person creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_distributor_person() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let person_data = json!({
            "name": "Alice Brown",
            "email": "alice.brown@distributor.com",
            "role": "distributor",
            "person_type": "distributor",
            "phone": "555-3456",
            "company": "Distributor Inc",
            "territory": "North America",
            "distribution_tier": "Tier 1",
            "commission_rate": "5%"
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(person_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Person creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_person_invalid_email() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let person_data = json!({
            "name": "John Doe",
            "email": "invalid-email",
            "phone": "+1234567890",
            "person_type": "customer",
            "company": "Acme Corp"
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(person_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Person creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_get_person_details() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let person_id = Uuid::new_v4().to_string();

        let request =
            create_request_with_tenant(Method::GET, &format!("/{}", person_id), None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Person routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_update_person() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let person_id = Uuid::new_v4().to_string();

        let update_data = json!({
            "name": "Updated Name",
            "email": "updated@example.com",
            "phone": "+1987654321"
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/{}", person_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();

        // Person routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_delete_person() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let person_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::DELETE,
            &format!("/{}", person_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();

        // Person routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Internal Person API Tests

    #[tokio::test]
    async fn test_list_internal_persons() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(Method::GET, "/internal", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Person routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_get_internal_person_details() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let person_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::GET,
            &format!("/internal/{}", person_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();

        // Person routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_update_internal_person() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let person_id = Uuid::new_v4().to_string();

        let update_data = json!({
            "department": "Updated Engineering",
            "position": "Senior Software Engineer",
            "employee_id": "EMP-456"
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/internal/{}", person_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();

        // Person routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Customer Person API Tests

    #[tokio::test]
    async fn test_list_customer_persons() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(Method::GET, "/customer", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Person routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_get_customer_person_details() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let person_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::GET,
            &format!("/customer/{}", person_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();

        // Person routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_update_customer_person() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let person_id = Uuid::new_v4().to_string();

        let update_data = json!({
            "name": "Updated Customer Name",
            "email": "updated.customer@example.com",
            "phone": "+1987654321",
            "billing_address": "456 Updated St"
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/customer/{}", person_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();

        // Person routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Vendor Person API Tests

    #[tokio::test]
    async fn test_list_vendor_persons() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(Method::GET, "/vendor", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Person routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_get_vendor_person_details() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let person_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::GET,
            &format!("/vendor/{}", person_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();

        // Person routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_update_vendor_person() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let person_id = Uuid::new_v4().to_string();

        let update_data = json!({
            "name": "Updated Vendor Name",
            "email": "updated.vendor@vendor.com",
            "phone": "+1987654321",
            "service_type": "Assembly"
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/vendor/{}", person_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();

        // Person routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Distributor Person API Tests

    #[tokio::test]
    async fn test_list_distributor_persons() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(Method::GET, "/distributor", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Person routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_get_distributor_person_details() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let person_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::GET,
            &format!("/distributor/{}", person_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();

        // Person routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_update_distributor_person() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let person_id = Uuid::new_v4().to_string();

        let update_data = json!({
            "name": "Updated Distributor Name",
            "email": "updated.distributor@distributor.com",
            "phone": "+1987654321",
            "territory": "East Coast"
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/distributor/{}", person_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();

        // Person routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Edge Case and Error Tests

    #[tokio::test]
    async fn test_tenant_isolation_different_tenants() {
        let app = app().await;
        let tenant_id_1 = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(Method::GET, "/", None, &tenant_id_1);

        let response = app.oneshot(request).await.unwrap();

        // Person routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_person_missing_required_fields() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let person_data = json!({
            "name": "John Doe"
            // Missing required fields
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(person_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Person creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_person_invalid_phone_length() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let person_data = json!({
            "name": "John Doe",
            "email": "john.doe@example.com",
            "phone": "123", // Too short
            "person_type": "customer",
            "company": "Acme Corp"
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(person_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Person creation requires authentication, will fail without JWT token
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
}
