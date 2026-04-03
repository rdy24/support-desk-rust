mod common;

use common::{setup_test_app, register_and_login};
use reqwest::Client;
use uuid::Uuid;

#[tokio::test]
async fn test_create_ticket_authenticated() {
    let (base_url, _pool) = setup_test_app().await;
    let client = Client::new();

    // Register dan login
    let email = format!("user_{}@example.com", Uuid::new_v4());
    let token = common::register_and_login(&base_url, &email, "password123").await;

    // Buat ticket
    let response = client
        .post(format!("{}/tickets", base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "subject": "Cannot login to system",
            "description": "I am unable to access my account after the recent update",
            "category": "technical",
            "priority": "high"
        }))
        .send()
        .await
        .expect("Failed to create ticket");

    assert_eq!(response.status(), 201); // Created

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["subject"], "Cannot login to system");
    assert_eq!(body["data"]["category"], "technical");
}

#[tokio::test]
async fn test_create_ticket_unauthenticated() {
    let (base_url, _pool) = setup_test_app().await;
    let client = Client::new();

    // Coba buat ticket tanpa token
    let response = client
        .post(format!("{}/tickets", base_url))
        .json(&serde_json::json!({
            "subject": "Some problem",
            "description": "This is a problem description",
            "category": "general",
            "priority": "low"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 401); // Unauthorized
}

#[tokio::test]
async fn test_get_tickets_list() {
    let (base_url, _pool) = setup_test_app().await;
    let client = Client::new();

    // Register dan login
    let email = format!("user_{}@example.com", Uuid::new_v4());
    let token = register_and_login(&base_url, &email, "password123").await;

    // Buat ticket pertama
    let _response1 = client
        .post(format!("{}/tickets", base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "subject": "First ticket",
            "description": "This is the first ticket",
            "category": "general",
            "priority": "low"
        }))
        .send()
        .await
        .expect("Failed to create first ticket");

    // Buat ticket kedua
    let _response2 = client
        .post(format!("{}/tickets", base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "subject": "Second ticket",
            "description": "This is the second ticket",
            "category": "billing",
            "priority": "medium"
        }))
        .send()
        .await
        .expect("Failed to create second ticket");

    // Ambil list tiket
    let response = client
        .get(format!("{}/tickets", base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to get tickets list");

    assert_eq!(response.status(), 200); // OK

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["success"], true);
    assert!(body["data"].is_array());
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_get_single_ticket() {
    let (base_url, _pool) = setup_test_app().await;
    let client = Client::new();

    // Register dan login
    let email = format!("user_{}@example.com", Uuid::new_v4());
    let token = register_and_login(&base_url, &email, "password123").await;

    // Buat ticket
    let create_response = client
        .post(format!("{}/tickets", base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "subject": "Specific ticket",
            "description": "Get this specific ticket",
            "category": "technical",
            "priority": "urgent"
        }))
        .send()
        .await
        .expect("Failed to create ticket");

    let create_body: serde_json::Value = create_response
        .json()
        .await
        .expect("Failed to parse create response");

    let ticket_id = create_body["data"]["id"]
        .as_str()
        .expect("No ID in response");

    // Ambil ticket spesifik
    let response = client
        .get(format!("{}/tickets/{}", base_url, ticket_id))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to get ticket");

    assert_eq!(response.status(), 200); // OK

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["subject"], "Specific ticket");
}
