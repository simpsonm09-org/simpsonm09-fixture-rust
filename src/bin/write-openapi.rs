use std::fs;
use std::path::Path;

use simpsonm09_fixture_rust::openapi;

/// Writes `docs/openapi.json` from the derived `ApiDoc`. Run by `just spec` from
/// the repository root so the generated document never drifts by hand.
fn main() {
    let path = Path::new("docs/openapi.json");
    fs::write(path, openapi::serialized()).expect("write docs/openapi.json");
    eprintln!("wrote {}", path.display());
}
