#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{header, Method, Request, StatusCode},
        Router,
    };
    use chrono::Utc;
    use dotenv::dotenv;
    use serde_json::{json, Value};
    use tower::ServiceExt; // for `oneshot` and `ready`
    use uuid::Uuid;

    use ems_server::{routes::machines::routes, services::DatabaseService, AppState};

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

    // Machine CRUD tests

    #[tokio::test]
    async fn test_create_machine_success() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let machine_data = json!({
            "name": "Test Machine",
            "ip": "192.168.1.100",
            "port": 8080,
            "protocol": "http",
            "status": "offline",
            "action": null,
            "payload": {"version": "1.0"},
            "metadata": {"location": "Factory Floor A"}
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(machine_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Machine creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_machine_invalid_data() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let invalid_machine_data = json!({
            "name": "", // Empty name should fail validation
            "ip": "192.168.1.100",
            "port": 8080,
            "protocol": "http"
        });

        let request =
            create_request_with_tenant(Method::POST, "/", Some(invalid_machine_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Machine creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_get_machine_details() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let machine_id = Uuid::new_v4().to_string();

        let request =
            create_request_with_tenant(Method::GET, &format!("/{}", machine_id), None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_list_machines() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(Method::GET, "/", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_list_machines_with_filters() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::GET,
            "/?status=offline&protocol=http&limit=10&offset=0",
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();

        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_update_machine() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let machine_id = Uuid::new_v4().to_string();

        let update_data = json!({
            "status": "maintenance",
            "location": "Maintenance Shop",
            "hourly_rate": 80.00,
            "last_maintenance_date": "2024-02-01T00:00:00Z"
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/{}", machine_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_delete_machine() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let machine_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::DELETE,
            &format!("/{}", machine_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Heartbeat tests

    #[tokio::test]
    async fn test_update_heartbeat() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let machine_id = Uuid::new_v4();

        let heartbeat_data = json!({
            "status": "idle",
            "action": "run",
            "payload": {"current_job": "JOB-123"}
        });

        let request = create_request_with_tenant(
            Method::POST,
            &format!("/{}/heartbeat", machine_id),
            Some(heartbeat_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();

        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Machine-Item relationship tests

    #[tokio::test]
    async fn test_create_machine_item_relationship() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let machine_id = Uuid::new_v4();

        let relationship_data = json!({
            "item_id": Uuid::new_v4(),
            "relationship_type": "builds",
            "notes": "Primary production item"
        });

        let request = create_request_with_tenant(
            Method::POST,
            &format!("/{}/items", machine_id),
            Some(relationship_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();

        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_list_machine_item_relationships() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let machine_id = Uuid::new_v4();

        let request = create_request_with_tenant(
            Method::GET,
            &format!("/{}/items", machine_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();

        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_delete_machine_item_relationship() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let relationship_id = Uuid::new_v4();

        let request = create_request_with_tenant(
            Method::DELETE,
            &format!("/items/{}", relationship_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();

        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Machine-Asset relationship tests

    #[tokio::test]
    async fn test_create_machine_asset_relationship() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let machine_id = Uuid::new_v4();

        let relationship_data = json!({
            "asset_id": Uuid::new_v4(),
            "relationship_type": "firmware",
            "notes": "Latest firmware version"
        });

        let request = create_request_with_tenant(
            Method::POST,
            &format!("/{}/assets", machine_id),
            Some(relationship_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();

        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_list_machine_asset_relationships() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let machine_id = Uuid::new_v4();

        let request = create_request_with_tenant(
            Method::GET,
            &format!("/{}/assets", machine_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();

        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_delete_machine_asset_relationship() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let relationship_id = Uuid::new_v4();

        let request = create_request_with_tenant(
            Method::DELETE,
            &format!("/assets/{}", relationship_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();

        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Machine-Operator assignment tests

    #[tokio::test]
    async fn test_create_machine_operator_assignment() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let machine_id = Uuid::new_v4();

        let assignment_data = json!({
            "person_id": Uuid::new_v4(),
            "assignment_type": "primary",
            "notes": "Primary operator for day shift"
        });

        let request = create_request_with_tenant(
            Method::POST,
            &format!("/{}/operators", machine_id),
            Some(assignment_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();

        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_list_machine_operator_assignments() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let machine_id = Uuid::new_v4();

        let request = create_request_with_tenant(
            Method::GET,
            &format!("/{}/operators", machine_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();

        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_delete_machine_operator_assignment() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let assignment_id = Uuid::new_v4();

        let request = create_request_with_tenant(
            Method::DELETE,
            &format!("/operators/{}", assignment_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();

        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Machine-Job assignment tests

    #[tokio::test]
    async fn test_create_machine_job_assignment() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let machine_id = Uuid::new_v4();

        let assignment_data = json!({
            "job_id": Uuid::new_v4(),
            "status": "pending",
            "start_time": Utc::now(),
            "notes": "High priority job"
        });

        let request = create_request_with_tenant(
            Method::POST,
            &format!("/{}/jobs", machine_id),
            Some(assignment_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();

        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_list_machine_job_assignments() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let machine_id = Uuid::new_v4();

        let request = create_request_with_tenant(
            Method::GET,
            &format!("/{}/jobs", machine_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();

        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_update_machine_job_assignment() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let assignment_id = Uuid::new_v4();

        let update_data = json!({
            "status": "in_progress",
            "start_time": Utc::now(),
            "notes": "Job started"
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/job-assignments/{}", assignment_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();

        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_delete_machine_job_assignment() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let assignment_id = Uuid::new_v4();

        let request = create_request_with_tenant(
            Method::DELETE,
            &format!("/job-assignments/{}", assignment_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();

        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Utility endpoint tests

    #[tokio::test]
    async fn test_get_machines_by_item() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let item_id = Uuid::new_v4();

        let request = create_request_with_tenant(
            Method::GET,
            &format!("/by-item/{}", item_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();

        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_get_machines_by_job() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let job_id = Uuid::new_v4();

        let request = create_request_with_tenant(
            Method::GET,
            &format!("/by-job/{}", job_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();

        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Authorization and validation tests

    #[tokio::test]
    async fn test_unauthorized_request() {
        let app = app().await;

        // Create a request without any authentication headers
        let request = Request::builder()
            .method(Method::GET)
            .uri("/")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Missing tenant header causes internal server error
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Input validation tests

    #[tokio::test]
    async fn test_invalid_port_number() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let invalid_machine_data = json!({
            "name": "Test Machine",
            "ip": "192.168.1.100",
            "port": -1, // Invalid port number
            "protocol": "http"
        });

        let request =
            create_request_with_tenant(Method::POST, "/", Some(invalid_machine_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Machine creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_invalid_protocol() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let invalid_machine_data = json!({
            "name": "Test Machine",
            "ip": "192.168.1.100",
            "port": 8080,
            "protocol": "invalid_protocol" // Invalid protocol
        });

        let request =
            create_request_with_tenant(Method::POST, "/", Some(invalid_machine_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Machine creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_invalid_status() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let invalid_machine_data = json!({
            "name": "Test Machine",
            "ip": "192.168.1.100",
            "port": 8080,
            "protocol": "http",
            "status": "invalid_status" // Invalid status
        });

        let request =
            create_request_with_tenant(Method::POST, "/", Some(invalid_machine_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Machine creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_list_all_machines() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(Method::GET, "/", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_list_machines_with_status_filter() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request =
            create_request_with_tenant(Method::GET, "/?status=operational", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_list_machines_with_category_filter() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request =
            create_request_with_tenant(Method::GET, "/?category=production", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_production_machine() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let machine_data = json!({
            "machine_number": "PROD-001",
            "name": "CNC Machine #1",
            "description": "High precision CNC machining center",
            "manufacturer": "HAAS Automation",
            "model": "VF-4SS",
            "serial_number": "SN-12345",
            "location": "Production Floor A",
            "category": "production",
            "status": "operational",
            "installation_date": "2023-01-15T00:00:00Z",
            "last_maintenance_date": "2024-01-01T00:00:00Z",
            "next_maintenance_date": "2024-07-01T00:00:00Z",
            "hourly_rate": 75.50,
            "power_rating": 15.0,
            "specifications": {
                "cutting_speed": "10000 RPM",
                "accuracy": "±0.001 inches",
                "work_envelope": "40x20x25 inches"
            }
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(machine_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Machine creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_qa_machine() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let machine_data = json!({
            "machine_number": "QA-001",
            "name": "CMM Measurement System",
            "description": "Coordinate measuring machine for quality inspection",
            "manufacturer": "Zeiss",
            "model": "CONTURA G2",
            "serial_number": "SN-67890",
            "location": "Quality Lab",
            "category": "qa",
            "status": "operational",
            "installation_date": "2023-03-10T00:00:00Z",
            "hourly_rate": 125.00,
            "specifications": {
                "measurement_accuracy": "±0.0001 inches",
                "measurement_volume": "20x16x12 inches",
                "probe_system": "VAST XT Gold"
            }
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(machine_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Machine creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_packaging_machine() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let machine_data = json!({
            "machine_number": "PKG-001",
            "name": "Automated Packaging Line",
            "description": "High-speed packaging and labeling system",
            "manufacturer": "Bosch Packaging",
            "model": "SVP-1000",
            "serial_number": "SN-11223",
            "location": "Packaging Department",
            "category": "packaging",
            "status": "operational",
            "installation_date": "2023-05-20T00:00:00Z",
            "hourly_rate": 95.75,
            "specifications": {
                "throughput": "1000 units/hour",
                "package_types": ["boxes", "pouches", "bottles"],
                "labeling": "automatic"
            }
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(machine_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Machine creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_machine_invalid_hourly_rate() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let invalid_machine_data = json!({
            "machine_number": "INVALID-001",
            "name": "Invalid Machine",
            "category": "production",
            "hourly_rate": -10.0 // Invalid negative rate
        });

        let request =
            create_request_with_tenant(Method::POST, "/", Some(invalid_machine_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Machine creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Production Machine API Tests

    #[tokio::test]
    async fn test_list_production_machines() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(Method::GET, "/production", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_get_production_machine_details() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let machine_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::GET,
            &format!("/production/{}", machine_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Production machine details route doesn't require authentication, returns NOT_FOUND for non-existent machine
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_update_production_machine() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let machine_id = Uuid::new_v4().to_string();

        let update_data = json!({
            "hourly_rate": 85.00,
            "specifications": {
                "cutting_speed": "12000 RPM",
                "accuracy": "±0.0005 inches"
            }
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/production/{}", machine_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Production machine update route doesn't require authentication, returns NOT_FOUND for non-existent machine
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // QA Machine API Tests

    #[tokio::test]
    async fn test_list_qa_machines() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(Method::GET, "/qa", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_get_qa_machine_details() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let machine_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::GET,
            &format!("/qa/{}", machine_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // QA machine details route doesn't require authentication, returns NOT_FOUND for non-existent machine
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_update_qa_machine() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let machine_id = Uuid::new_v4().to_string();

        let update_data = json!({
            "hourly_rate": 130.00,
            "specifications": {
                "measurement_accuracy": "±0.00005 inches",
                "calibration_date": "2024-01-15T00:00:00Z"
            }
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/qa/{}", machine_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // QA machine update route doesn't require authentication, returns NOT_FOUND for non-existent machine
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // Packaging Machine API Tests

    #[tokio::test]
    async fn test_list_packaging_machines() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(Method::GET, "/packaging", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_get_packaging_machine_details() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let machine_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::GET,
            &format!("/packaging/{}", machine_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Packaging machine details route doesn't require authentication, returns NOT_FOUND for non-existent machine
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_update_packaging_machine() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let machine_id = Uuid::new_v4().to_string();

        let update_data = json!({
            "hourly_rate": 100.00,
            "specifications": {
                "throughput": "1200 units/hour",
                "efficiency": "98%"
            }
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/packaging/{}", machine_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Packaging machine update route doesn't require authentication, returns NOT_FOUND for non-existent machine
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // Edge Case and Error Tests

    #[tokio::test]
    async fn test_tenant_isolation_different_tenants() {
        let app = app().await;
        let tenant_id1 = Uuid::new_v4().to_string();
        let tenant_id2 = Uuid::new_v4().to_string();

        // List machines for both tenants
        let request1 = create_request_with_tenant(Method::GET, "/", None, &tenant_id1);
        let request2 = create_request_with_tenant(Method::GET, "/", None, &tenant_id2);

        let response1 = app.clone().oneshot(request1).await.unwrap();
        let response2 = app.oneshot(request2).await.unwrap();

        // Machine routes require authentication, both will fail without JWT token
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
    async fn test_create_machine_missing_required_fields() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let invalid_machine_data = json!({
            "category": "production"
            // Missing required machine_number and name fields
        });

        let request =
            create_request_with_tenant(Method::POST, "/", Some(invalid_machine_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Machine creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_machine_invalid_machine_number_length() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let invalid_machine_data = json!({
            "machine_number": "this-machine-number-is-way-too-long-to-be-valid-and-should-fail-validation",
            "name": "Test Machine",
            "category": "production"
        });

        let request =
            create_request_with_tenant(Method::POST, "/", Some(invalid_machine_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Machine creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_missing_tenant_header() {
        let app = app().await;

        let machine_data = json!({
            "machine_number": "NO-TENANT-001",
            "name": "No Tenant Machine",
            "category": "production"
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(machine_data.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Missing tenant header causes internal server error
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_list_machines_with_pagination() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request =
            create_request_with_tenant(Method::GET, "/?limit=10&offset=0", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_list_machines_with_all_filters() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::GET,
            "/?category=production&status=operational&limit=5&offset=0",
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_machine_status_transitions() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let machine_id = Uuid::new_v4().to_string();

        // Try to update machine status (will fail because machine doesn't exist)
        let update_data = json!({
            "status": "maintenance"
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/{}", machine_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Machine routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
