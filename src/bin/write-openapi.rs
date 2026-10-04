use std::fs;
use std::path::Path;

use simpsonm09_fixture_rust::openapi::ApiDoc;
use utoipa::OpenApi;

/// Writes `docs/openapi.json` from the derived `ApiDoc`. Run by `just spec` from
/// the repository root so the generated document never drifts by hand.
fn main() {
    let serialized = format!(
        "{}\n",
        ApiDoc::openapi()
            .to_pretty_json()
            .expect("serialize the OpenAPI document")
    );
    let path = Path::new("docs/openapi.json");
    fs::write(path, serialized).expect("write docs/openapi.json");
    eprintln!("wrote {}", path.display());
}
