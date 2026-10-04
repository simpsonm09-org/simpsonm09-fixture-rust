//! The HTTP controller. It speaks DTOs, drives the service, and maps service
//! errors to responses. It never touches the store.

use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::item::domain::{Item, ItemError};
use crate::item::service::ItemService;

/// Payload used to create or replace an item. `id` is assigned by the store.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ItemRequest {
    /// Item name.
    #[schema(max_length = 200, example = "Widget")]
    pub name: String,
    /// Item description.
    #[schema(max_length = 2000, example = "A small widget", nullable = true)]
    pub description: Option<String>,
}

/// An item returned by the API.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ItemResponse {
    /// Server-assigned identifier.
    #[schema(example = 1)]
    pub id: i64,
    /// Item name.
    #[schema(example = "Widget")]
    pub name: String,
    /// Item description.
    #[schema(example = "A small widget", nullable = true)]
    pub description: Option<String>,
}

/// RFC 7807 problem detail returned for item errors.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ProblemDetail {
    /// Problem type URI.
    pub r#type: String,
    /// Short human-readable summary.
    pub title: String,
    /// HTTP status code.
    pub status: u16,
    /// Explanation specific to this occurrence.
    pub detail: String,
}

/// Translates a request DTO into a domain item.
fn to_item(request: ItemRequest) -> Item {
    Item {
        id: None,
        name: request.name,
        description: request.description,
    }
}

/// Translates a domain item into a response DTO.
fn to_response(item: Item) -> ItemResponse {
    ItemResponse {
        id: item.id.expect("a persisted item must have an id"),
        name: item.name,
        description: item.description,
    }
}

/// Maps a JSON extraction rejection (missing field, wrong type, malformed body)
/// to the contract's 400 validation error, so the transport response is a
/// problem detail instead of axum's default 422.
fn rejection_to_error(rejection: JsonRejection) -> ItemError {
    ItemError::Validation(rejection.body_text())
}

/// Validates a request at the controller boundary. Blank or over-length names
/// and over-length descriptions are rejected.
fn validate(request: &ItemRequest) -> Result<(), ItemError> {
    if request.name.trim().is_empty() {
        return Err(ItemError::Validation("name must not be blank".into()));
    }
    if request.name.chars().count() > 200 {
        return Err(ItemError::Validation(
            "name must be at most 200 characters".into(),
        ));
    }
    if let Some(description) = &request.description {
        if description.chars().count() > 2000 {
            return Err(ItemError::Validation(
                "description must be at most 2000 characters".into(),
            ));
        }
    }
    Ok(())
}

impl IntoResponse for ItemError {
    fn into_response(self) -> Response {
        let (status, title, detail) = match self {
            ItemError::NotFound(id) => (
                StatusCode::NOT_FOUND,
                "Item not found",
                format!("Item {id} was not found"),
            ),
            ItemError::Validation(message) => (StatusCode::BAD_REQUEST, "Validation failed", message),
        };
        let body = ProblemDetail {
            r#type: "about:blank".to_string(),
            title: title.to_string(),
            status: status.as_u16(),
            detail,
        };
        let mut response = Json(body).into_response();
        *response.status_mut() = status;
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
        response
    }
}

/// Lists every item.
#[utoipa::path(
    get,
    path = "/items",
    tag = "Items",
    responses((status = 200, description = "List every item", body = [ItemResponse]))
)]
pub async fn list_items(State(service): State<Arc<ItemService>>) -> Json<Vec<ItemResponse>> {
    Json(service.list_items().into_iter().map(to_response).collect())
}

/// Gets one item by id.
#[utoipa::path(
    get,
    path = "/items/{id}",
    tag = "Items",
    params(("id" = i64, Path, description = "Item identifier", example = 1)),
    responses(
        (status = 200, description = "Get one item by id", body = ItemResponse),
        (status = 404, description = "Item not found", body = ProblemDetail, content_type = "application/problem+json")
    )
)]
pub async fn get_item(
    State(service): State<Arc<ItemService>>,
    Path(id): Path<i64>,
) -> Result<Json<ItemResponse>, ItemError> {
    service.get_item(id).map(|item| Json(to_response(item)))
}

/// Creates an item.
#[utoipa::path(
    post,
    path = "/items",
    tag = "Items",
    request_body = ItemRequest,
    responses(
        (status = 201, description = "Create an item", body = ItemResponse),
        (status = 400, description = "Validation failed", body = ProblemDetail, content_type = "application/problem+json")
    )
)]
pub async fn create_item(
    State(service): State<Arc<ItemService>>,
    request: Result<Json<ItemRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<ItemResponse>), ItemError> {
    let Json(request) = request.map_err(rejection_to_error)?;
    validate(&request)?;
    let domain = to_item(request);
    let created = service.create_item(domain.name, domain.description);
    Ok((StatusCode::CREATED, Json(to_response(created))))
}

/// Replaces an item.
#[utoipa::path(
    put,
    path = "/items/{id}",
    tag = "Items",
    params(("id" = i64, Path, description = "Item identifier", example = 1)),
    request_body = ItemRequest,
    responses(
        (status = 200, description = "Replace an item", body = ItemResponse),
        (status = 400, description = "Validation failed", body = ProblemDetail, content_type = "application/problem+json"),
        (status = 404, description = "Item not found", body = ProblemDetail, content_type = "application/problem+json")
    )
)]
pub async fn update_item(
    State(service): State<Arc<ItemService>>,
    Path(id): Path<i64>,
    request: Result<Json<ItemRequest>, JsonRejection>,
) -> Result<Json<ItemResponse>, ItemError> {
    let Json(request) = request.map_err(rejection_to_error)?;
    validate(&request)?;
    let domain = to_item(request);
    service
        .update_item(id, domain.name, domain.description)
        .map(|item| Json(to_response(item)))
}

/// Deletes an item.
#[utoipa::path(
    delete,
    path = "/items/{id}",
    tag = "Items",
    params(("id" = i64, Path, description = "Item identifier", example = 1)),
    responses(
        (status = 204, description = "Item deleted"),
        (status = 404, description = "Item not found", body = ProblemDetail, content_type = "application/problem+json")
    )
)]
pub async fn delete_item(
    State(service): State<Arc<ItemService>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ItemError> {
    service.delete_item(id)?;
    Ok(StatusCode::NO_CONTENT)
}
