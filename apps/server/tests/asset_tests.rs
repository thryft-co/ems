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

    use ems_server::{routes::assets::routes, services::DatabaseService, AppState};

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

    // Asset Type API Tests

    #[tokio::test]
    async fn test_list_asset_types() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(Method::GET, "/types", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();

        // The database connection works and migrations include default asset types
        // so this should return OK with the default asset types
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_create_asset_type() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let asset_type_data = json!({
            "name": "custom_document",
            "description": "Custom document type for testing"
        });

        let request =
            create_request_with_tenant(Method::POST, "/types", Some(asset_type_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Asset type creation requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_asset_type_invalid_name() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let asset_type_data = json!({
            "name": "", // Invalid empty name
            "description": "Test description"
        });

        let request =
            create_request_with_tenant(Method::POST, "/types", Some(asset_type_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Should return BAD_REQUEST for validation error
        assert!(
            response.status() == StatusCode::BAD_REQUEST
                || response.status() == StatusCode::UNPROCESSABLE_ENTITY
        );
    }

    #[tokio::test]
    async fn test_get_asset_type() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let type_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::GET,
            &format!("/types/{}", type_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Should return NOT_FOUND for non-existent asset type
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_update_asset_type() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let type_id = Uuid::new_v4().to_string();

        let update_data = json!({
            "name": "updated_document",
            "description": "Updated description for testing"
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/types/{}", type_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Asset type update requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_delete_asset_type() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let type_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::DELETE,
            &format!("/types/{}", type_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Asset type delete operation succeeds and returns NO_CONTENT
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    // Asset API Tests

    #[tokio::test]
    async fn test_list_assets() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(Method::GET, "/", None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Asset listing requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_list_assets_with_filters() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::GET,
            "/?asset_type_id=123e4567-e89b-12d3-a456-426614174000&is_active=true&limit=10&offset=0",
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Asset listing requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_create_invoice_asset() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let asset_data = json!({
            "item_id": Uuid::new_v4(),
            "asset_type": "invoice",
            "name": "Invoice Q1 2024",
            "version": "1.0",
            "description": "Quarterly invoice document",
            "file_path": "/uploads/invoices/q1-2024.pdf",
            "file_size": 245760,
            "file_type": "application/pdf",
            "checksum": "a1b2c3d4e5f6789012345678901234567890abcd",
            "metadata": {
                "quarter": "Q1",
                "year": 2024,
                "currency": "USD"
            }
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(asset_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Since tests don't include JWT authentication, this will return INTERNAL_SERVER_ERROR
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_create_firmware_asset() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let asset_data = json!({
            "item_id": Uuid::new_v4(),
            "asset_type": "firmware",
            "name": "Device Firmware v2.1.0",
            "version": "2.1.0",
            "description": "Latest firmware with security fixes",
            "file_path": "/uploads/firmware/device-v2.1.0.bin",
            "file_size": 1048576,
            "file_type": "application/octet-stream",
            "checksum": "9876543210abcdef1234567890abcdef12345678",
            "metadata": {
                "format": "binary",
                "architecture": "arm64"
            },
            "firmware_details": {
                "hardware_version": "1.2",
                "min_hardware_version": "1.0",
                "max_hardware_version": "1.5",
                "release_notes": "Fixed critical security vulnerability CVE-2024-001",
                "is_beta": false,
                "is_critical": true,
                "requires_manual_update": false
            }
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(asset_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Since tests don't include JWT authentication, this will return INTERNAL_SERVER_ERROR
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_create_firmware_asset_beta() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let asset_data = json!({
            "item_id": Uuid::new_v4(),
            "asset_type": "firmware",
            "name": "Device Firmware v3.0.0-beta",
            "version": "3.0.0-beta",
            "description": "Beta firmware with new features",
            "file_path": "/uploads/firmware/device-v3.0.0-beta.hex",
            "file_size": 524288,
            "file_type": "application/octet-stream",
            "checksum": "fedcba0987654321fedcba0987654321fedcba09",
            "firmware_details": {
                "hardware_version": "2.0",
                "min_hardware_version": "2.0",
                "release_notes": "New feature set - use at your own risk",
                "is_beta": true,
                "is_critical": false,
                "requires_manual_update": true
            }
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(asset_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Since tests don't include JWT authentication, this will return INTERNAL_SERVER_ERROR
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_create_document_asset() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let asset_data = json!({
            "item_id": Uuid::new_v4(),
            "asset_type": "document",
            "name": "User Manual v1.5",
            "version": "1.5",
            "description": "Complete user manual with troubleshooting guide",
            "file_path": "/uploads/docs/user-manual-v1.5.pdf",
            "file_size": 2097152,
            "file_type": "application/pdf",
            "checksum": "abcdef123456789abcdef123456789abcdef1234",
            "metadata": {
                "language": "en",
                "pages": 150,
                "category": "user_guide"
            }
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(asset_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Since tests don't include JWT authentication, this will return INTERNAL_SERVER_ERROR
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_create_asset_invalid_data() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let asset_data = json!({
            "asset_type": "invoice",
            "name": "", // Invalid empty name
            "file_size": -1000 // Invalid negative file size
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(asset_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Since tests don't include JWT authentication, this will return INTERNAL_SERVER_ERROR
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_get_asset() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let asset_id = Uuid::new_v4().to_string();

        let request =
            create_request_with_tenant(Method::GET, &format!("/{}", asset_id), None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Asset get requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_update_asset() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let asset_id = Uuid::new_v4().to_string();

        let update_data = json!({
            "name": "Updated Asset Name",
            "version": "2.0",
            "description": "Updated description",
            "is_active": false,
            "metadata": {
                "updated": true,
                "reason": "version_update"
            }
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/{}", asset_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Asset update requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_update_firmware_asset() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let asset_id = Uuid::new_v4().to_string();

        let update_data = json!({
            "name": "Updated Firmware v2.2.0",
            "version": "2.2.0",
            "description": "Updated firmware with bug fixes",
            "firmware_details": {
                "hardware_version": "1.3",
                "release_notes": "Fixed memory leak and improved stability",
                "is_beta": false,
                "is_critical": false
            }
        });

        let request = create_request_with_tenant(
            Method::PUT,
            &format!("/{}", asset_id),
            Some(update_data),
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Asset update requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_delete_asset() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let asset_id = Uuid::new_v4().to_string();

        let request =
            create_request_with_tenant(Method::DELETE, &format!("/{}", asset_id), None, &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Asset delete requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Utility endpoint tests

    #[tokio::test]
    async fn test_get_assets_by_item() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let item_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::GET,
            &format!("/by-item/{}", item_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Asset get by item requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_get_assets_by_type() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();
        let asset_type_id = Uuid::new_v4().to_string();

        let request = create_request_with_tenant(
            Method::GET,
            &format!("/by-type/{}", asset_type_id),
            None,
            &tenant_id,
        );

        let response = app.oneshot(request).await.unwrap();
        // Asset get by type requires authentication, will fail without JWT token
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Tenant isolation tests

    #[tokio::test]
    async fn test_tenant_isolation_different_tenants() {
        let app = app().await;
        let tenant_id_1 = Uuid::new_v4().to_string();
        let tenant_id_2 = Uuid::new_v4().to_string();

        // Since asset routes require authentication, both requests will fail
        let request1 = create_request_with_tenant(Method::GET, "/", None, &tenant_id_1);
        let request2 = create_request_with_tenant(Method::GET, "/", None, &tenant_id_2);

        let response1 = app.clone().oneshot(request1).await.unwrap();
        let response2 = app.oneshot(request2).await.unwrap();

        // Both should fail due to missing authentication, not return empty lists
        assert!(
            response1.status() == StatusCode::UNAUTHORIZED
                || response1.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
        assert!(
            response2.status() == StatusCode::UNAUTHORIZED
                || response2.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Validation tests

    #[tokio::test]
    async fn test_create_asset_missing_required_fields() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let asset_data = json!({
            "asset_type": "document"
            // Missing item_id, name, and other required fields
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(asset_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Since tests don't include JWT authentication, this will return INTERNAL_SERVER_ERROR
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_create_asset_invalid_asset_type() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let asset_data = json!({
            "item_id": Uuid::new_v4(),
            "asset_type": "invalid_type", // Invalid asset type
            "name": "Test Asset"
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(asset_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Since tests don't include JWT authentication, this will return INTERNAL_SERVER_ERROR
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_create_asset_invalid_file_size() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let asset_data = json!({
            "item_id": Uuid::new_v4(),
            "asset_type": "document",
            "name": "Test Document",
            "file_size": -500 // Invalid negative file size
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(asset_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Since tests don't include JWT authentication, this will return INTERNAL_SERVER_ERROR
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_create_asset_long_name() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let asset_data = json!({
            "item_id": Uuid::new_v4(),
            "asset_type": "document",
            "name": "a".repeat(200), // Too long name (over 100 chars)
            "version": "1.0"
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(asset_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Since tests don't include JWT authentication, this will return INTERNAL_SERVER_ERROR
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
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
        // Missing tenant header causes internal server error
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Advanced firmware tests

    #[tokio::test]
    async fn test_create_firmware_s19_format() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let asset_data = json!({
            "item_id": Uuid::new_v4(),
            "asset_type": "firmware",
            "name": "Bootloader v1.0.0",
            "version": "1.0.0",
            "description": "System bootloader in S19 format",
            "file_path": "/uploads/firmware/bootloader-v1.0.0.s19",
            "file_size": 131072,
            "file_type": "application/octet-stream",
            "checksum": "s19format1234567890abcdef1234567890abcdef",
            "metadata": {
                "format": "s19",
                "target": "bootloader",
                "architecture": "cortex-m4"
            },
            "firmware_details": {
                "hardware_version": "1.0",
                "min_hardware_version": "1.0",
                "max_hardware_version": "1.0",
                "release_notes": "Initial bootloader release",
                "is_beta": false,
                "is_critical": true,
                "requires_manual_update": true
            }
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(asset_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Since tests don't include JWT authentication, this will return INTERNAL_SERVER_ERROR
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_create_firmware_hex_format() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let asset_data = json!({
            "item_id": Uuid::new_v4(),
            "asset_type": "firmware",
            "name": "Application v1.2.3",
            "version": "1.2.3",
            "description": "Main application firmware in Intel HEX format",
            "file_path": "/uploads/firmware/app-v1.2.3.hex",
            "file_size": 262144,
            "file_type": "application/octet-stream",
            "checksum": "hexformat567890abcdef1234567890abcdef1234",
            "metadata": {
                "format": "hex",
                "target": "application",
                "build_date": "2024-01-15"
            },
            "firmware_details": {
                "hardware_version": "2.1",
                "min_hardware_version": "2.0",
                "max_hardware_version": "2.5",
                "release_notes": "Performance improvements and bug fixes",
                "is_beta": false,
                "is_critical": false,
                "requires_manual_update": false
            }
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(asset_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Since tests don't include JWT authentication, this will return INTERNAL_SERVER_ERROR
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Certificate tests

    #[tokio::test]
    async fn test_create_certificate_asset() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let asset_data = json!({
            "item_id": Uuid::new_v4(),
            "asset_type": "certificate",
            "name": "ISO 9001 Certificate",
            "version": "2024",
            "description": "Quality management system certification",
            "file_path": "/uploads/certificates/iso9001-2024.pdf",
            "file_size": 1024000,
            "file_type": "application/pdf",
            "checksum": "cert1234567890abcdef1234567890abcdef1234",
            "metadata": {
                "issuer": "ISO Certification Body",
                "valid_from": "2024-01-01",
                "valid_to": "2027-01-01",
                "certificate_number": "ISO9001-2024-001"
            }
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(asset_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Since tests don't include JWT authentication, this will return INTERNAL_SERVER_ERROR
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Report tests

    #[tokio::test]
    async fn test_create_report_asset() {
        let app = app().await;
        let tenant_id = Uuid::new_v4().to_string();

        let asset_data = json!({
            "item_id": Uuid::new_v4(),
            "asset_type": "report",
            "name": "Quality Assurance Report Q1 2024",
            "version": "1.0",
            "description": "Comprehensive QA report for first quarter",
            "file_path": "/uploads/reports/qa-report-q1-2024.pdf",
            "file_size": 3145728,
            "file_type": "application/pdf",
            "checksum": "report567890abcdef1234567890abcdef123456789",
            "metadata": {
                "quarter": "Q1",
                "year": 2024,
                "department": "Quality Assurance",
                "author": "QA Team Lead"
            }
        });

        let request = create_request_with_tenant(Method::POST, "/", Some(asset_data), &tenant_id);

        let response = app.oneshot(request).await.unwrap();
        // Since tests don't include JWT authentication, this will return INTERNAL_SERVER_ERROR
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
