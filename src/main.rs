use std::sync::Arc;

use simpsonm09_fixture_rust::app::build_router;
use simpsonm09_fixture_rust::item::service::ItemService;
use simpsonm09_fixture_rust::item::store::InMemoryItemStore;

#[tokio::main]
async fn main() {
    let store = Arc::new(InMemoryItemStore::new());
    let service = Arc::new(ItemService::new(store));
    let app = build_router(service);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3000);
    let addr = format!("127.0.0.1:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind the HTTP listener");
    println!("listening on http://{addr}");
    axum::serve(listener, app).await.expect("serve the API");
}
