//! An item as the service layer reasons about it, free of transport and store
//! detail. `id` is `None` until the store assigns it on save.

/// The domain item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
}

/// An error raised by the service. The API layer maps it to an HTTP response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemError {
    /// No item matched the id.
    NotFound(i64),
    /// The request failed validation.
    Validation(String),
}
