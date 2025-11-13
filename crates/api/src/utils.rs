use serde::{Deserialize, Serialize};

/// API SERVER VERSION
pub const API_SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Response Status
///
/// Used to represent the status of the response
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ResponseStatus {
    Success,
    Error,
}

/// Response API
///
/// Used to represent the response API
#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseAPI<D> {
    pub status: ResponseStatus,
    pub data: D,
}

impl<D> ResponseAPI<D> {
    /// Creates a new response API
    fn new(status: ResponseStatus, data: D) -> Self {
        Self { status, data }
    }

    /// Creates a new success response API
    pub fn success(data: D) -> Self {
        Self::new(ResponseStatus::Success, data)
    }

    /// Creates a new error response API
    pub fn error(data: D) -> Self {
        Self::new(ResponseStatus::Error, data)
    }
}

impl<D> actix_web::Responder for ResponseAPI<D>
where
    D: Serialize,
{
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        actix_web::HttpResponse::Ok().json(self)
    }
}
