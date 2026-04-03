mod common;

use common::setup_test_app;
use reqwest::Client;

#[tokio::test]
async fn test_register_success() {
    let (base_url, _pool) = setup_test_app().await;
    let client = Client::new();

    let response = client
        .post(format!("{}/auth/register", base_url))
        .json(&serde_json::json!({
            "name": "Test User",
            "email": "test@example.com",
            "password": "password123",
            "role": "customer"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 201); // Created

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["email"], "test@example.com");
}

#[tokio::test]
async fn test_register_duplicate_email() {
    let (base_url, _pool) = setup_test_app().await;
    let client = Client::new();

    // Register pertama kali
    client
        .post(format!("{}/auth/register", base_url))
        .json(&serde_json::json!({
            "name": "First User",
            "email": "duplicate@example.com",
            "password": "password123",
            "role": "customer"
        }))
        .send()
        .await
        .expect("Failed to register first time");

    // Register kedua dengan email yang sama
    let response = client
        .post(format!("{}/auth/register", base_url))
        .json(&serde_json::json!({
            "name": "Second User",
            "email": "duplicate@example.com",
            "password": "password456",
            "role": "customer"
        }))
        .send()
        .await
        .expect("Failed to register second time");

    assert_eq!(response.status(), 409); // Conflict
}

#[tokio::test]
async fn test_login_success() {
    let (base_url, _pool) = setup_test_app().await;
    let client = Client::new();

    // Register terlebih dahulu
    client
        .post(format!("{}/auth/register", base_url))
        .json(&serde_json::json!({
            "name": "Login Test User",
            "email": "login@example.com",
            "password": "password123",
            "role": "customer"
        }))
        .send()
        .await
        .expect("Failed to register");

    // Login
    let response = client
        .post(format!("{}/auth/login", base_url))
        .json(&serde_json::json!({
            "email": "login@example.com",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to login");

    assert_eq!(response.status(), 200); // OK

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["success"], true);
    assert!(body["data"]["token"].is_string());
}

#[tokio::test]
async fn test_login_wrong_password() {
    let (base_url, _pool) = setup_test_app().await;
    let client = Client::new();

    // Register
    client
        .post(format!("{}/auth/register", base_url))
        .json(&serde_json::json!({
            "name": "Wrong Pass User",
            "email": "wrongpass@example.com",
            "password": "correctpassword",
            "role": "customer"
        }))
        .send()
        .await
        .expect("Failed to register");

    // Login dengan password yang salah
    let response = client
        .post(format!("{}/auth/login", base_url))
        .json(&serde_json::json!({
            "email": "wrongpass@example.com",
            "password": "wrongpassword"
        }))
        .send()
        .await
        .expect("Failed to login attempt");

    assert_eq!(response.status(), 401); // Unauthorized
}

#[tokio::test]
async fn test_login_unregistered_email() {
    let (base_url, _pool) = setup_test_app().await;
    let client = Client::new();

    // Login dengan email yang tidak terdaftar
    let response = client
        .post(format!("{}/auth/login", base_url))
        .json(&serde_json::json!({
            "email": "unregistered@example.com",
            "password": "somepassword"
        }))
        .send()
        .await
        .expect("Failed to login attempt");

    assert_eq!(response.status(), 401); // Unauthorized
}
