//! Integration tests driving the router with `tower::ServiceExt::oneshot`.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

use simpsonm09_fixture_rust::app::build_router;
use simpsonm09_fixture_rust::item::service::ItemService;
use simpsonm09_fixture_rust::item::store::InMemoryItemStore;

/// A router over an empty in-memory store, so each test starts clean.
fn empty_app() -> Router {
    let store = InMemoryItemStore::new();
    store.clear();
    build_router(Arc::new(ItemService::new(Arc::new(store))))
}

/// A router over the three dev seeds.
fn seeded_app() -> Router {
    build_router(Arc::new(ItemService::new(Arc::new(
        InMemoryItemStore::new(),
    ))))
}

struct TestResponse {
    status: StatusCode,
    content_type: String,
    body: Vec<u8>,
}

impl TestResponse {
    fn json(&self) -> Value {
        serde_json::from_slice(&self.body).expect("response body is JSON")
    }
}

async fn send(app: &Router, request: Request<Body>) -> TestResponse {
    let response = app.clone().oneshot(request).await.expect("router response");
    let status = response.status();
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes()
        .to_vec();
    TestResponse {
        status,
        content_type,
        body,
    }
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

fn json_body(method: &str, uri: &str, body: &Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn post(uri: &str, body: &Value) -> Request<Body> {
    json_body("POST", uri, body)
}

fn put(uri: &str, body: &Value) -> Request<Body> {
    json_body("PUT", uri, body)
}

fn delete(uri: &str) -> Request<Body> {
    Request::builder()
        .method("DELETE")
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn lists_no_items_when_the_store_is_empty() {
    let app = empty_app();
    let response = send(&app, get("/items")).await;
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.json(), json!([]));
}

#[tokio::test]
async fn drives_the_full_create_read_update_delete_lifecycle() {
    let app = empty_app();

    let created = send(
        &app,
        post(
            "/items",
            &json!({"name": "Widget", "description": "A small widget"}),
        ),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED);
    let created = created.json();
    let id = created["id"].as_i64().expect("id is an integer");
    assert_eq!(created["name"], "Widget");
    assert_eq!(created["description"], "A small widget");

    let list = send(&app, get("/items")).await;
    assert_eq!(list.status, StatusCode::OK);
    assert_eq!(list.json().as_array().unwrap().len(), 1);

    let one = send(&app, get(&format!("/items/{id}"))).await;
    assert_eq!(one.status, StatusCode::OK);
    assert_eq!(
        one.json(),
        json!({"id": id, "name": "Widget", "description": "A small widget"})
    );

    let updated = send(
        &app,
        put(
            &format!("/items/{id}"),
            &json!({"name": "Renamed", "description": "Still here"}),
        ),
    )
    .await;
    assert_eq!(updated.status, StatusCode::OK);
    assert_eq!(
        updated.json(),
        json!({"id": id, "name": "Renamed", "description": "Still here"})
    );

    let deleted = send(&app, delete(&format!("/items/{id}"))).await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);

    let missing = send(&app, get(&format!("/items/{id}"))).await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn stores_a_missing_description_as_null() {
    let app = empty_app();
    let created = send(&app, post("/items", &json!({"name": "No description"}))).await;
    assert_eq!(created.status, StatusCode::CREATED);
    assert_eq!(created.json()["description"], Value::Null);
}

#[tokio::test]
async fn seeds_three_items_on_startup() {
    let app = seeded_app();
    let response = send(&app, get("/items")).await;
    assert_eq!(response.status, StatusCode::OK);
    let body = response.json();
    let names: Vec<&str> = body
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["Widget", "Gadget", "Gizmo"]);
}

#[tokio::test]
async fn returns_a_404_problem_detail_for_an_unknown_id_on_get() {
    let app = empty_app();
    let response = send(&app, get("/items/999")).await;
    assert_eq!(response.status, StatusCode::NOT_FOUND);
    assert!(response.content_type.contains("application/problem+json"));
    assert_eq!(
        response.json(),
        json!({
            "type": "about:blank",
            "title": "Item not found",
            "status": 404,
            "detail": "Item 999 was not found"
        })
    );
}

#[tokio::test]
async fn returns_a_404_for_an_unknown_id_on_put() {
    let app = empty_app();
    let response = send(&app, put("/items/999", &json!({"name": "X"}))).await;
    assert_eq!(response.status, StatusCode::NOT_FOUND);
    assert!(response.content_type.contains("application/problem+json"));
    assert_eq!(response.json()["title"], "Item not found");
}

#[tokio::test]
async fn returns_a_404_for_an_unknown_id_on_delete() {
    let app = empty_app();
    let response = send(&app, delete("/items/999")).await;
    assert_eq!(response.status, StatusCode::NOT_FOUND);
    assert!(response.content_type.contains("application/problem+json"));
    assert_eq!(response.json()["title"], "Item not found");
}

#[tokio::test]
async fn rejects_a_blank_name_with_a_400_problem_detail() {
    let app = empty_app();
    let response = send(&app, post("/items", &json!({"name": "   "}))).await;
    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert!(response.content_type.contains("application/problem+json"));
    assert_eq!(
        response.json(),
        json!({
            "type": "about:blank",
            "title": "Validation failed",
            "status": 400,
            "detail": "name must not be blank"
        })
    );
    assert_eq!(
        send(&app, post("/items", &json!({"name": ""})))
            .await
            .status,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn rejects_a_missing_name_field_with_400() {
    let app = empty_app();
    let response = send(&app, post("/items", &json!({}))).await;
    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert!(response.content_type.contains("application/problem+json"));
    let body = response.json();
    assert_eq!(body["type"], "about:blank");
    assert_eq!(body["title"], "Validation failed");
    assert_eq!(body["status"], 400);
    assert!(
        body["detail"].as_str().unwrap().contains("name"),
        "the detail should name the missing field: {}",
        body["detail"]
    );
}

#[tokio::test]
async fn rejects_a_wrong_typed_field_with_400() {
    let app = empty_app();
    let response = send(&app, post("/items", &json!({"name": 123}))).await;
    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert!(response.content_type.contains("application/problem+json"));
    assert_eq!(response.json()["title"], "Validation failed");
    assert_eq!(response.json()["status"], 400);
}

#[tokio::test]
async fn rejects_an_over_length_name_with_400() {
    let app = empty_app();
    let response = send(&app, post("/items", &json!({"name": "a".repeat(201)}))).await;
    assert_eq!(response.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_an_over_length_description_with_400() {
    let app = empty_app();
    let response = send(
        &app,
        post(
            "/items",
            &json!({"name": "Ok", "description": "a".repeat(2001)}),
        ),
    )
    .await;
    assert_eq!(response.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_a_malformed_json_body_with_400() {
    let app = empty_app();
    let request = Request::builder()
        .method("POST")
        .uri("/items")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"name": "Broken""#))
        .unwrap();
    let response = send(&app, request).await;
    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert!(response.content_type.contains("application/problem+json"));
    assert_eq!(response.json()["title"], "Validation failed");
}

#[tokio::test]
async fn rejects_an_over_length_name_on_put_with_400() {
    let app = empty_app();
    let created = send(&app, post("/items", &json!({"name": "Widget"}))).await;
    let id = created.json()["id"].as_i64().unwrap();
    let response = send(
        &app,
        put(&format!("/items/{id}"), &json!({"name": "a".repeat(201)})),
    )
    .await;
    assert_eq!(response.status, StatusCode::BAD_REQUEST);
}
