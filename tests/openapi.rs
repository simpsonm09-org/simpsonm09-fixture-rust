//! Keeps `docs/openapi.json` in step with the derived document and checks the
//! two contract details the OpenAPI shape must hold.

use serde_json::Value;
use simpsonm09_fixture_rust::openapi;

/// The document exactly as `just spec` writes it.
fn generated() -> String {
    openapi::serialized()
}

#[test]
fn committed_document_matches_the_generated_one() {
    let committed = std::fs::read_to_string("docs/openapi.json")
        .expect("docs/openapi.json exists; run `just spec`");
    assert_eq!(
        generated(),
        committed,
        "docs/openapi.json drifted; run `just spec` and commit the result"
    );
}

#[test]
fn description_is_a_nullable_string_not_an_object() {
    let document: Value = serde_json::from_str(&generated()).unwrap();
    for schema in ["ItemRequest", "ItemResponse"] {
        let description = &document["components"]["schemas"][schema]["properties"]["description"];
        let kind = &description["type"];
        let allows_string = kind == "string"
            || kind
                .as_array()
                .is_some_and(|types| types.iter().any(|value| value == "string"));
        let allows_null = kind == "string"
            || kind
                .as_array()
                .is_some_and(|types| types.iter().any(|value| value == "null"));
        assert!(
            allows_string,
            "{schema}.description must be a string, got {kind}"
        );
        assert!(
            allows_null,
            "{schema}.description must be nullable, got {kind}"
        );
        assert_ne!(kind, "object", "{schema}.description must not be an object");
    }
}

#[test]
fn not_found_responses_use_the_problem_json_content_type() {
    let document: Value = serde_json::from_str(&generated()).unwrap();
    for method in ["get", "put", "delete"] {
        let content = &document["paths"]["/items/{id}"][method]["responses"]["404"]["content"];
        assert!(
            content.get("application/problem+json").is_some(),
            "the 404 on {method} /items/{{id}} must be application/problem+json"
        );
    }
}
