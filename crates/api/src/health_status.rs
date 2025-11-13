use crate::utils::{API_SERVER_VERSION, ResponseAPI};
use actix_web::Responder;

pub struct HealthStatus;

impl HealthStatus {
    /// Index route
    ///
    /// Responds with a simple message
    pub async fn index() -> impl Responder {
        ResponseAPI::success(API_SERVER_VERSION)
    }

    /// Health Check route
    ///
    /// Responds with a simple message
    pub async fn health_check() -> impl Responder {
        ResponseAPI::success("OK")
    }
}
