//! A layered Rust + axum item CRUD service for the simpsonm09 fleet standard.
//!
//! The request flow is controller (DTOs) -> service (domain `Item`) -> store
//! port, with an in-memory adapter behind the port. See `docs/architecture.md`.

pub mod app;
pub mod item;
pub mod openapi;
