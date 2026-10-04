use std::sync::Arc;

use axum::routing::get;
use axum::Router;

use crate::item::api;
use crate::item::service::ItemService;

/// Builds the HTTP router. The service is shared by every handler and the
/// controller never reaches past it to the store.
pub fn build_router(service: Arc<ItemService>) -> Router {
    Router::new()
        .route("/items", get(api::list_items).post(api::create_item))
        .route(
            "/items/{id}",
            get(api::get_item)
                .put(api::update_item)
                .delete(api::delete_item),
        )
        .with_state(service)
}
