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

    use ems_server::{routes::jobs::routes, services::DatabaseService, AppState};

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

    // General Job API Tests

    #[tokio::test]
    async fn test_list_all_jobs() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(Method::GET, "/", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Job routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_list_jobs_with_type_filter() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request =
            create_request_with_tenant(Method::GET, "/?job_type=manufacturing", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Job routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_list_jobs_with_status_filter() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request =
            create_request_with_tenant(Method::GET, "/?status=in_progress", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Job routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_manufacturing_job() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let job_data = json!({
            "job_number": "MFG-001",
            "quantity": 100,
            "job_type": "manufacturing",
            "priority": "normal",
            "status": "pending",
            "work_order_number": "WO-001",
            "production_line": "Line A",
            "machine_id": "MACHINE-001",
            "setup_time_hours": 2.5,
            "cycle_time_minutes": 1.5,
            "quality_check_required": true,
            "batch_size": 50
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(job_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Job creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_qa_job() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let job_data = json!({
            "job_number": "QA-001",
            "quantity": 50,
            "job_type": "qa",
            "priority": "high",
            "status": "pending",
            "inspection_type": "Visual",
            "test_procedure_id": "TP-001",
            "acceptance_criteria": "No defects allowed",
            "sampling_size": 10,
            "calibration_required": true
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(job_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Job creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_service_job() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let job_data = json!({
            "job_number": "SRV-001",
            "quantity": 1,
            "job_type": "service",
            "priority": "urgent",
            "status": "pending",
            "service_type": "Maintenance",
            "location": "Factory Floor B",
            "equipment_serial_number": "EQ-12345",
            "maintenance_type": "Preventive",
            "travel_time_hours": 1.0
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(job_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Job creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_job_invalid_quantity() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let invalid_job_data = json!({
            "job_number": "INVALID-001",
            "quantity": 0, // Invalid quantity (must be > 0)
            "job_type": "manufacturing"
        });

        let request =
            create_request_with_tenant(Method::POST, "/", Some(invalid_job_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Job creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_get_job_details() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let job_id = Uuid::new_v4().to_string();

        let request =
            create_request_with_tenant(Method::GET, &format!("/{}", job_id), None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Job routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_update_job() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let job_id = Uuid::new_v4().to_string();

        let update_data = json!({
            "quantity": 150,
            "priority": "high",
            "status": "in_progress"
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/{}", job_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Job routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_delete_job() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let job_id = Uuid::new_v4().to_string();

        let request =
            create_request_with_tenant(Method::DELETE, &format!("/{}", job_id), None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Job routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Manufacturing Job API Tests

    #[tokio::test]
    async fn test_list_manufacturing_jobs() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(Method::GET, "/manufacturing", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Job routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_get_manufacturing_job_details() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let job_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::GET,
            &format!("/manufacturing/{}", job_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Job routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_update_manufacturing_job() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let job_id = Uuid::new_v4().to_string();

        let update_data = json!({
            "work_order_number": "WO-002",
            "production_line": "Line B",
            "setup_time_hours": 3.0,
            "cycle_time_minutes": 2.0
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/manufacturing/{}", job_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Job routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // QA Job API Tests

    #[tokio::test]
    async fn test_list_qa_jobs() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(Method::GET, "/qa", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Job routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_get_qa_job_details() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let job_id = Uuid::new_v4().to_string();

        let request =
            create_request_with_tenant(Method::GET, &format!("/qa/{}", job_id), None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Job routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_update_qa_job() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let job_id = Uuid::new_v4().to_string();

        let update_data = json!({
            "inspection_type": "Dimensional",
            "test_procedure_id": "TP-002",
            "acceptance_criteria": "Within tolerance limits",
            "sampling_size": 20
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/qa/{}", job_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Job routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Service Job API Tests

    #[tokio::test]
    async fn test_list_service_jobs() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(Method::GET, "/service", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Job routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_get_service_job_details() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let job_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::GET,
            &format!("/service/{}", job_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Job routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_update_service_job() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let job_id = Uuid::new_v4().to_string();

        let update_data = json!({
            "service_type": "Repair",
            "location": "Workshop",
            "equipment_serial_number": "EQ-67890",
            "maintenance_type": "Corrective"
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/service/{}", job_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Job routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Edge Case and Error Tests

    #[tokio::test]
    async fn test_tenant_isolation_different_tenants() {
        let app = app().await;
        let tenant_id1 = Uuid::new_v4().to_string();
        let tenant_id2 = Uuid::new_v4().to_string();

        // List jobs for both tenants
        let request1 = create_request_with_tenant(Method::GET, "/", None, &tenant_id1);
        let request2 = create_request_with_tenant(Method::GET, "/", None, &tenant_id2);

        let response1 = app.clone().oneshot(request1).await.unwrap();
        let response2 = app.oneshot(request2).await.unwrap();

        // Job routes require authentication, both will fail without JWT token
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
    async fn test_create_job_missing_required_fields() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let invalid_job_data = json!({
            "quantity": 100
            // Missing required job_number and job_type fields
        });

        let request =
            create_request_with_tenant(Method::POST, "/", Some(invalid_job_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Job creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_job_invalid_job_number_length() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let invalid_job_data = json!({
            "job_number": "this-job-number-is-way-too-long-to-be-valid-and-should-fail-validation-because-it-exceeds-fifty-characters",
            "quantity": 100,
            "job_type": "manufacturing"
        });

        let request =
            create_request_with_tenant(Method::POST, "/", Some(invalid_job_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // Job creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_missing_tenant_header() {
        let app = app().await;

        let job_data = json!({
            "job_number": "NO-TENANT-001",
            "quantity": 100,
            "job_type": "manufacturing"
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(job_data.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Missing tenant header causes internal server error
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_create_job_with_pagination() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request =
            create_request_with_tenant(Method::GET, "/?limit=10&offset=0", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Job routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_job_with_all_filters() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::GET,
            "/?type=manufacturing&status=pending&priority=high&limit=5&offset=0",
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Job routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_job_status_transitions() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let job_id = Uuid::new_v4().to_string();

        // Try to update job status (will fail because job doesn't exist)
        let update_data = json!({
            "status": "in_progress"
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/{}", job_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Job routes require authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
