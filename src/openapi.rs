//! The generated OpenAPI document. `just spec` writes it to `docs/openapi.json`
//! and a test fails when the committed document drifts.

use utoipa::OpenApi;

// The glob also brings the `__path_*` items the `paths(...)` list refers to.
use crate::item::api::*;

/// The OpenAPI document for the item API.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Simpsonm09 Fixture Rust API",
        version = "0.1.0",
        description = "Item CRUD service for the simpsonm09 repository fixture"
    ),
    paths(list_items, get_item, create_item, update_item, delete_item),
    components(schemas(ItemRequest, ItemResponse, ProblemDetail))
)]
pub struct ApiDoc;

/// Serializes the document exactly as `just spec` writes it: pretty JSON with a
/// trailing newline. The writer binary and the drift test share this so they
/// cannot disagree on the byte layout.
pub fn serialized() -> String {
    format!(
        "{}\n",
        ApiDoc::openapi()
            .to_pretty_json()
            .expect("serialize the OpenAPI document")
    )
}
