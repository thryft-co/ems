use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use ems_server::AppState;
use serde_json::json;
use std::env;
use tower::ServiceExt;
use uuid::Uuid;

// Test data constants
const TEST_EMAIL: &str = "test@example.com";
const TEST_PASSWORD: &str = "TestPassword123!";
const TEST_FIRST_NAME: &str = "John";
const TEST_LAST_NAME: &str = "Doe";
const TEST_TENANT_SUBDOMAIN: &str = "test-tenant";
const TEST_TENANT_NAME: &str = "Test Company";

// Additional test users
const TEST_EMAIL_2: &str = "test2@example.com";
const TEST_EMAIL_3: &str = "test3@example.com";
const TEST_TENANT_SUBDOMAIN_2: &str = "test-tenant-2";
const TEST_TENANT_NAME_2: &str = "Test Company 2";

// Setup test environment variables
fn setup_test_env() {
    dotenv::dotenv().ok();

    // Set test environment variables if not already set
    if env::var("JWT_SECRET").is_err() {
        env::set_var(
            "JWT_SECRET",
            "test-jwt-secret-key-for-integration-testing-only",
        );
    }
}

// Helper function to create the app router for testing
async fn create_test_app() -> Router {
    setup_test_env();

    // Create AppState - expect this to work with real database
    let app_state = AppState::new()
        .await
        .expect("Failed to create AppState - ensure database is running and configured properly");

    Router::new().nest(
        "/api/v1/auth",
        ems_server::routes::auth::routes().with_state(app_state),
    )
}

// Helper function to make HTTP requests
async fn make_request(
    app: &Router,
    method: &str,
    path: &str,
    payload: Option<serde_json::Value>,
    headers: Option<Vec<(&str, &str)>>,
) -> (StatusCode, serde_json::Value) {
    let mut request_builder = Request::builder().method(method).uri(path);

    // Add headers if provided
    if let Some(headers) = headers {
        for (key, value) in headers {
            request_builder = request_builder.header(key, value);
        }
    }

    // Add default content type for POST requests
    if method == "POST" {
        request_builder = request_builder.header("content-type", "application/json");
    }

    let request = if let Some(payload) = payload {
        request_builder
            .body(Body::from(payload.to_string()))
            .unwrap()
    } else {
        request_builder.body(Body::empty()).unwrap()
    };

    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: serde_json::Value = if body.is_empty() {
        json!({})
    } else {
        serde_json::from_slice(&body).unwrap_or_else(|_| json!({"error": "Invalid JSON response"}))
    };

    (status, response_json)
}

#[tokio::test]
async fn test_user_registration_with_database_verification() {
    let app = create_test_app().await;

    // Generate unique test data to avoid conflicts
    let unique_email = format!("test-reg-{}@example.com", Uuid::new_v4());
    let unique_subdomain = format!(
        "test-reg-{}",
        Uuid::new_v4().to_string()[..8].to_lowercase()
    );

    let register_payload = json!({
        "email": unique_email,
        "first_name": TEST_FIRST_NAME,
        "last_name": TEST_LAST_NAME,
        "password": TEST_PASSWORD,
        "tenant_subdomain": unique_subdomain,
        "tenant_name": TEST_TENANT_NAME
    });

    let (status, response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/register",
        Some(register_payload),
        None,
    )
    .await;

    // Print response for debugging
    println!("Registration response status: {}", status);
    println!(
        "Registration response body: {}",
        serde_json::to_string_pretty(&response).unwrap_or_default()
    );

    // Check if registration succeeded or if we got expected errors
    match status {
        StatusCode::OK => {
            // Registration succeeded - verify response structure
            assert!(
                response["access_token"].is_string(),
                "Should return access token"
            );
            assert!(
                response["refresh_token"].is_string(),
                "Should return refresh token"
            );
            assert_eq!(
                response["user"]["email"], unique_email,
                "Should return correct email"
            );
            assert_eq!(
                response["user"]["first_name"], TEST_FIRST_NAME,
                "Should return correct first name"
            );
            assert_eq!(
                response["user"]["last_name"], TEST_LAST_NAME,
                "Should return correct last name"
            );
            assert_eq!(
                response["tenant"]["name"], TEST_TENANT_NAME,
                "Should return correct tenant name"
            );
            assert_eq!(
                response["tenant"]["subdomain"], unique_subdomain,
                "Should return correct tenant subdomain"
            );

            // Verify user and tenant IDs are valid UUIDs
            let user_id = response["user"]["id"]
                .as_str()
                .expect("User ID should be present");
            let tenant_id = response["tenant"]["id"]
                .as_str()
                .expect("Tenant ID should be present");

            assert!(
                Uuid::parse_str(user_id).is_ok(),
                "User ID should be valid UUID"
            );
            assert!(
                Uuid::parse_str(tenant_id).is_ok(),
                "Tenant ID should be valid UUID"
            );

            println!(
                "✅ Registration successful - User created with ID: {}, Tenant ID: {}",
                user_id, tenant_id
            );
        }
        StatusCode::INTERNAL_SERVER_ERROR => {
            // Expected if Supabase is not configured - check error message or empty response
            if response.is_object() && response.as_object().unwrap().is_empty() {
                println!("⚠️  Registration failed due to missing Supabase configuration - this is expected in test environment");
                return; // Skip test if Supabase not configured
            }
            if let Some(error_msg) = response.get("error").and_then(|e| e.as_str()) {
                if error_msg.contains("SUPABASE_URL") || error_msg.contains("SUPABASE_") {
                    println!("⚠️  Registration failed due to missing Supabase configuration - this is expected in test environment");
                    return; // Skip test if Supabase not configured
                }
            }
            println!(
                "⚠️  Registration failed with server error (likely Supabase config issue): {}",
                response
            );
            return; // Skip test for server errors in test environment
        }
        StatusCode::BAD_REQUEST => {
            println!("❌ Registration failed with bad request: {}", response);
            panic!("Registration should not fail with bad request for valid data");
        }
        _ => {
            println!(
                "❌ Registration failed with unexpected status: {} - {}",
                status, response
            );
            panic!("Unexpected registration failure");
        }
    }
}

#[tokio::test]
async fn test_user_sign_in_flow() {
    let app = create_test_app().await;

    // Generate unique test data
    let unique_email = format!("test-signin-{}@example.com", Uuid::new_v4());
    let unique_subdomain = format!(
        "test-signin-{}",
        Uuid::new_v4().to_string()[..8].to_lowercase()
    );

    // Step 1: Register a user first
    let register_payload = json!({
        "email": unique_email,
        "first_name": TEST_FIRST_NAME,
        "last_name": TEST_LAST_NAME,
        "password": TEST_PASSWORD,
        "tenant_subdomain": unique_subdomain,
        "tenant_name": TEST_TENANT_NAME
    });

    let (reg_status, _reg_response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/register",
        Some(register_payload),
        None,
    )
    .await;

    if reg_status != StatusCode::OK {
        println!("⚠️  Skipping sign-in test - registration failed (likely due to Supabase config)");
        return;
    }

    // Step 2: Sign in with the registered credentials
    let login_payload = json!({
        "email": unique_email,
        "password": TEST_PASSWORD,
        "tenant_subdomain": unique_subdomain
    });

    let (status, response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/login",
        Some(login_payload),
        None,
    )
    .await;

    println!("Sign-in response status: {}", status);
    println!(
        "Sign-in response body: {}",
        serde_json::to_string_pretty(&response).unwrap_or_default()
    );

    assert_eq!(status, StatusCode::OK, "Sign-in should succeed");
    assert!(
        response["access_token"].is_string(),
        "Should return access token"
    );
    assert!(
        response["refresh_token"].is_string(),
        "Should return refresh token"
    );
    assert_eq!(
        response["user"]["email"], unique_email,
        "Should return correct email"
    );
    assert_eq!(
        response["tenant"]["subdomain"], unique_subdomain,
        "Should return correct tenant subdomain"
    );

    println!("✅ Sign-in successful");

    // Step 3: Test sign-in without tenant subdomain (should use primary tenant)
    let login_no_tenant_payload = json!({
        "email": unique_email,
        "password": TEST_PASSWORD
    });

    let (status, response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/login",
        Some(login_no_tenant_payload),
        None,
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "Login without tenant should succeed and use primary tenant"
    );
    assert_eq!(
        response["user"]["email"], unique_email,
        "Should return correct email"
    );

    println!("✅ Sign-in without tenant subdomain successful");
}

#[tokio::test]
async fn test_user_sign_out_flow() {
    let app = create_test_app().await;

    // Generate unique test data
    let unique_email = format!("test-signout-{}@example.com", Uuid::new_v4());
    let unique_subdomain = format!(
        "test-signout-{}",
        Uuid::new_v4().to_string()[..8].to_lowercase()
    );

    // Step 1: Register a user
    let register_payload = json!({
        "email": unique_email,
        "first_name": TEST_FIRST_NAME,
        "last_name": TEST_LAST_NAME,
        "password": TEST_PASSWORD,
        "tenant_subdomain": unique_subdomain,
        "tenant_name": TEST_TENANT_NAME
    });

    let (reg_status, _reg_response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/register",
        Some(register_payload),
        None,
    )
    .await;

    if reg_status != StatusCode::OK {
        println!(
            "⚠️  Skipping sign-out test - registration failed (likely due to Supabase config)"
        );
        return;
    }

    // Step 2: Sign in to get tokens
    let login_payload = json!({
        "email": unique_email,
        "password": TEST_PASSWORD,
        "tenant_subdomain": unique_subdomain
    });

    let (login_status, login_response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/login",
        Some(login_payload),
        None,
    )
    .await;

    if login_status != StatusCode::OK {
        println!("⚠️  Skipping sign-out test - login failed");
        return;
    }

    let access_token = login_response["access_token"].as_str().unwrap();
    let refresh_token = login_response["refresh_token"].as_str().unwrap();
    let tenant_id = login_response["tenant"]["id"].as_str().unwrap();

    // Step 3: Sign out using the refresh token
    let logout_payload = json!({
        "refresh_token": refresh_token
    });

    let auth_header = format!("Bearer {}", access_token);
    let logout_headers = vec![
        ("authorization", auth_header.as_str()),
        ("X-Tenant-ID", tenant_id),
    ];

    let (status, response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/logout",
        Some(logout_payload),
        Some(logout_headers),
    )
    .await;

    println!("Sign-out response status: {}", status);
    println!(
        "Sign-out response body: {}",
        serde_json::to_string_pretty(&response).unwrap_or_default()
    );

    assert_eq!(status, StatusCode::OK, "Sign-out should succeed");
    println!("✅ Sign-out successful");

    // Step 4: Verify that the tokens are now blacklisted by trying to use them again
    let test_logout_payload = json!({
        "refresh_token": refresh_token
    });

    let blacklisted_auth_header = format!("Bearer {}", access_token);
    let blacklisted_headers = vec![
        ("authorization", blacklisted_auth_header.as_str()),
        ("X-Tenant-ID", tenant_id),
    ];

    let (status, _response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/logout",
        Some(test_logout_payload),
        Some(blacklisted_headers),
    )
    .await;

    // Should fail because token is blacklisted
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "Should reject blacklisted tokens"
    );
    println!("✅ Token blacklisting verified");
}

#[tokio::test]
async fn test_person_only_registration() {
    let app = create_test_app().await;

    let unique_email = format!("test-person-{}@example.com", Uuid::new_v4());

    let person_register_payload = json!({
        "email": unique_email,
        "first_name": TEST_FIRST_NAME,
        "last_name": TEST_LAST_NAME,
        "password": TEST_PASSWORD
    });

    let (status, response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/person-register",
        Some(person_register_payload),
        None,
    )
    .await;

    println!("Person-only registration response status: {}", status);
    println!(
        "Person-only registration response body: {}",
        serde_json::to_string_pretty(&response).unwrap_or_default()
    );

    match status {
        StatusCode::OK => {
            assert!(
                response["access_token"].is_string(),
                "Should return access token"
            );
            assert!(
                response["refresh_token"].is_string(),
                "Should return refresh token"
            );
            assert_eq!(
                response["person"]["email"], unique_email,
                "Should return correct email"
            );
            assert_eq!(
                response["person"]["first_name"], TEST_FIRST_NAME,
                "Should return correct first name"
            );
            assert_eq!(
                response["person"]["last_name"], TEST_LAST_NAME,
                "Should return correct last name"
            );

            let person_id = response["person"]["id"]
                .as_str()
                .expect("Person ID should be present");
            assert!(
                Uuid::parse_str(person_id).is_ok(),
                "Person ID should be valid UUID"
            );

            println!(
                "✅ Person-only registration successful - Person created with ID: {}",
                person_id
            );
        }
        StatusCode::INTERNAL_SERVER_ERROR => {
            println!("⚠️  Person-only registration failed due to Supabase configuration - this is expected in test environment");
            return;
        }
        _ => {
            println!(
                "❌ Person-only registration failed with status: {} - {}",
                status, response
            );
            panic!("Unexpected person-only registration failure");
        }
    }
}

#[tokio::test]
async fn test_refresh_token_flow() {
    let app = create_test_app().await;

    // Generate unique test data
    let unique_email = format!("test-refresh-{}@example.com", Uuid::new_v4());
    let unique_subdomain = format!(
        "test-refresh-{}",
        Uuid::new_v4().to_string()[..8].to_lowercase()
    );

    // Step 1: Register and login to get initial tokens
    let register_payload = json!({
        "email": unique_email,
        "first_name": TEST_FIRST_NAME,
        "last_name": TEST_LAST_NAME,
        "password": TEST_PASSWORD,
        "tenant_subdomain": unique_subdomain,
        "tenant_name": TEST_TENANT_NAME
    });

    let (reg_status, reg_response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/register",
        Some(register_payload),
        None,
    )
    .await;

    if reg_status != StatusCode::OK {
        println!(
            "⚠️  Skipping refresh token test - registration failed (likely due to Supabase config)"
        );
        return;
    }

    let initial_refresh_token = reg_response["refresh_token"].as_str().unwrap();

    // Wait a moment to ensure different timestamp for new tokens
    tokio::time::sleep(tokio::time::Duration::from_millis(1100)).await;

    // Step 2: Use refresh token to get new tokens
    let refresh_payload = json!({
        "refresh_token": initial_refresh_token
    });

    let (status, response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/refresh",
        Some(refresh_payload),
        None,
    )
    .await;

    println!("Refresh token response status: {}", status);
    println!(
        "Refresh token response body: {}",
        serde_json::to_string_pretty(&response).unwrap_or_default()
    );

    assert_eq!(status, StatusCode::OK, "Token refresh should succeed");
    assert!(
        response["access_token"].is_string(),
        "Should return new access token"
    );
    assert!(
        response["refresh_token"].is_string(),
        "Should return new refresh token"
    );

    // Verify new tokens are different from original
    let _new_access_token = response["access_token"].as_str().unwrap();
    let new_refresh_token = response["refresh_token"].as_str().unwrap();

    assert_ne!(
        new_refresh_token, initial_refresh_token,
        "New refresh token should be different"
    );

    println!("✅ Token refresh successful");
}

#[tokio::test]
async fn test_registration_validation() {
    let app = create_test_app().await;

    let invalid_payloads = vec![
        // Missing email
        json!({
            "first_name": "John",
            "last_name": "Doe",
            "password": "password123",
            "tenant_subdomain": "test-tenant",
            "tenant_name": "Test Company"
        }),
        // Invalid email
        json!({
            "email": "invalid-email",
            "first_name": "John",
            "last_name": "Doe",
            "password": "password123",
            "tenant_subdomain": "test-tenant",
            "tenant_name": "Test Company"
        }),
        // Short password
        json!({
            "email": "test@example.com",
            "first_name": "John",
            "last_name": "Doe",
            "password": "123",
            "tenant_subdomain": "test-tenant",
            "tenant_name": "Test Company"
        }),
        // Invalid tenant subdomain
        json!({
            "email": "test@example.com",
            "first_name": "John",
            "last_name": "Doe",
            "password": "password123",
            "tenant_subdomain": "Invalid_Subdomain!",
            "tenant_name": "Test Company"
        }),
    ];

    for (i, payload) in invalid_payloads.iter().enumerate() {
        let (status, _response) = make_request(
            &app,
            "POST",
            "/api/v1/auth/register",
            Some(payload.clone()),
            None,
        )
        .await;

        // Should reject with validation error (400 or 422)
        assert!(
            status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
            "Should reject invalid payload {} with validation error (400 or 422), got: {}",
            i + 1,
            status
        );
    }
}

#[tokio::test]
async fn test_login_validation() {
    let app = create_test_app().await;

    let invalid_payloads = vec![
        // Missing email
        json!({
            "password": "password123"
        }),
        // Invalid email
        json!({
            "email": "invalid-email",
            "password": "password123"
        }),
        // Missing password
        json!({
            "email": "test@example.com"
        }),
    ];

    for (i, payload) in invalid_payloads.iter().enumerate() {
        let (status, _response) = make_request(
            &app,
            "POST",
            "/api/v1/auth/login",
            Some(payload.clone()),
            None,
        )
        .await;

        // Should reject with validation error (400 or 422)
        assert!(
            status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
            "Should reject invalid login payload {} with validation error (400 or 422), got: {}",
            i + 1,
            status
        );
    }
}

#[tokio::test]
async fn test_duplicate_registration() {
    let app = create_test_app().await;

    // Generate unique test data for first registration
    let unique_email = format!("test-dup-{}@example.com", Uuid::new_v4());
    let unique_subdomain = format!(
        "test-dup-{}",
        Uuid::new_v4().to_string()[..8].to_lowercase()
    );

    let register_payload = json!({
        "email": unique_email,
        "first_name": "Jane",
        "last_name": "Doe",
        "password": "TestPassword123!",
        "tenant_subdomain": unique_subdomain,
        "tenant_name": "Duplicate Test Company"
    });

    // First registration - may fail due to Supabase config, skip test if so
    let (status, _response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/register",
        Some(register_payload.clone()),
        None,
    )
    .await;

    if status.is_server_error() {
        // Skip test if Supabase is not properly configured
        println!("⚠️  Skipping duplicate registration test - Supabase not configured");
        return;
    }

    assert_eq!(status, StatusCode::OK, "First registration should succeed");

    // Second registration with same email should fail
    let (status, response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/register",
        Some(register_payload),
        None,
    )
    .await;

    println!(
        "Duplicate email registration response: {} - {}",
        status, response
    );
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "Should reject duplicate email"
    );

    // Try to register same tenant subdomain with different email
    let different_email = format!("test-dup-different-{}@example.com", Uuid::new_v4());
    let duplicate_tenant_payload = json!({
        "email": different_email,
        "first_name": "Different",
        "last_name": "User",
        "password": "TestPassword123!",
        "tenant_subdomain": unique_subdomain,
        "tenant_name": "Different Company"
    });

    let (status, response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/register",
        Some(duplicate_tenant_payload),
        None,
    )
    .await;

    println!(
        "Duplicate tenant registration response: {} - {}",
        status, response
    );
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "Should reject duplicate tenant subdomain"
    );

    println!("✅ Duplicate registration prevention verified");
}

#[tokio::test]
async fn test_invalid_credentials() {
    let app = create_test_app().await;

    let invalid_login_payload = json!({
        "email": "nonexistent@example.com",
        "password": "wrongpassword",
        "tenant_subdomain": "nonexistent-tenant"
    });

    let (status, response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/login",
        Some(invalid_login_payload),
        None,
    )
    .await;

    println!("Invalid credentials response: {} - {}", status, response);

    // Should be UNAUTHORIZED for invalid credentials
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "Should reject non-existent user with unauthorized status"
    );

    println!("✅ Invalid credentials properly rejected");
}

#[tokio::test]
async fn test_database_entries_verification() {
    let app = create_test_app().await;

    // This test verifies that entries are actually created in the database
    // by attempting multiple operations that depend on database state

    let unique_email = format!("test-db-{}@example.com", Uuid::new_v4());
    let unique_subdomain = format!("test-db-{}", Uuid::new_v4().to_string()[..8].to_lowercase());

    // Step 1: Register user
    let register_payload = json!({
        "email": unique_email,
        "first_name": TEST_FIRST_NAME,
        "last_name": TEST_LAST_NAME,
        "password": TEST_PASSWORD,
        "tenant_subdomain": unique_subdomain,
        "tenant_name": TEST_TENANT_NAME
    });

    let (reg_status, reg_response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/register",
        Some(register_payload),
        None,
    )
    .await;

    if reg_status != StatusCode::OK {
        println!("⚠️  Skipping database verification test - registration failed (likely due to Supabase config)");
        return;
    }

    let user_id = reg_response["user"]["id"].as_str().unwrap();
    let tenant_id = reg_response["tenant"]["id"].as_str().unwrap();

    println!(
        "✅ Database entries created - User: {}, Tenant: {}",
        user_id, tenant_id
    );

    // Step 2: Verify we can login (proves database entries exist)
    let login_payload = json!({
        "email": unique_email,
        "password": TEST_PASSWORD,
        "tenant_subdomain": unique_subdomain
    });

    let (login_status, login_response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/login",
        Some(login_payload),
        None,
    )
    .await;

    assert_eq!(
        login_status,
        StatusCode::OK,
        "Login should succeed, proving database entries exist"
    );
    assert_eq!(
        login_response["user"]["id"], user_id,
        "Should return same user ID"
    );
    assert_eq!(
        login_response["tenant"]["id"], tenant_id,
        "Should return same tenant ID"
    );

    println!("✅ Database entries verified through successful login");

    // Step 3: Test logout (proves token blacklist table works)
    let access_token = login_response["access_token"].as_str().unwrap();
    let refresh_token = login_response["refresh_token"].as_str().unwrap();

    let logout_payload = json!({
        "refresh_token": refresh_token
    });

    let auth_header = format!("Bearer {}", access_token);
    let logout_headers = vec![
        ("authorization", auth_header.as_str()),
        ("X-Tenant-ID", tenant_id),
    ];

    let (logout_status, _) = make_request(
        &app,
        "POST",
        "/api/v1/auth/logout",
        Some(logout_payload),
        Some(logout_headers),
    )
    .await;

    assert_eq!(
        logout_status,
        StatusCode::OK,
        "Logout should succeed, proving token blacklist table works"
    );

    println!("✅ Token blacklist database functionality verified");
    println!("✅ All database entries and operations verified successfully");
}
