# simpsonm09-fixture-rust working agreements

A layered Rust + axum item CRUD service. It is the Rust fixture for the fleet standard.

## Ground rules

- Business logic lives in `src/item/service.rs` and works in `domain::Item`. The controller never touches the store and the service never sees a DTO. Only `src/item/api.rs` bridges DTOs and the domain; only the store adapter bridges the port and its records.
- `docs/openapi.json` is generated. Annotate the handlers and the DTOs with `utoipa` and run `just spec`; never hand-edit the document. A test fails when the committed document drifts.
- The store is in memory. A restart discards items and restores the three dev seeds.
- No secret, credential, or machine path is committed.

## Commands

- `just install`, `just deps`, `just lint`, `just lint-fix`, `just aislop`, `just test`, `just coverage`, `just spec`, `just serve`, `just verify`, `just prune`.

## Repo facts

- Language and toolchain: Rust 1.99 with `clippy`, `rustfmt`, and `llvm-tools-preview`, `axum` 0.8 for HTTP, `utoipa` 5 for the OpenAPI document, `tokio` for the runtime, pinned in `mise.toml` and `Cargo.toml`.
- Tests: `cargo test`. Unit tests cover `ItemService` against a fake `ItemStore`; integration tests drive the router with `tower::ServiceExt::oneshot`. Coverage uses `cargo-llvm-cov` and writes `coverage/lcov.info`, the path the fleet patch-coverage gate reads.
- Data: no database. `src/item/store.rs` holds a `BTreeMap` behind a `Mutex` and seeds three items on construction.
- Domain: `GET`, `POST`, `PUT`, and `DELETE` over `/items`. Reads, updates, and deletes of an unknown id return a domain `ItemError::NotFound` that `src/item/api.rs` maps to a 404 `application/problem+json` body.
- Contracts: `docs/openapi.json` is generated from the handlers and the DTOs. Regenerate it with `just spec`.
- Docs: `docs/README.md` indexes the architecture, the items feature, and the OpenAPI contract.
- Windows build: the GNU target needs MinGW binutils (`dlltool`, `as`, `ld`) on `PATH` or `cargo test` fails with `dlltool.exe: program not found`. `cargo llvm-cov` cannot run on the Windows GNU host (E0463, no `libprofiler_builtins`), so run `just coverage` in CI only.

## Skills

No repo-local skills. General best practices and integration come from the plugins.
