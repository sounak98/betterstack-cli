use serde::Deserialize;

/// Wrapper for paginated API responses (JSON:API format).
#[derive(Debug, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: Option<Pagination>,
}

// Fields mirror the JSON:API pagination shape. All are deserialized
// even if only `next` is currently used, for future features.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Pagination {
    pub first: Option<String>,
    pub last: Option<String>,
    pub prev: Option<String>,
    pub next: Option<String>,
}

/// Wrapper for single-resource API responses.
#[derive(Debug, Deserialize)]
pub struct SingleResponse<T> {
    pub data: T,
}
