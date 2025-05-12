use actix_web::{Error, dev::ServiceRequest};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use std::env;

/// Verifica que la petición lleve la API_KEY del sistema
pub async fn api_key_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let valid_api_key = env::var("API_KEY").expect("API_KEY must be set");

    if credentials.token().eq(&valid_api_key) {
        tracing::info!("API Key is valid");
        Ok(req)
    } else {
        Err((actix_web::error::ErrorUnauthorized("Invalid API Key"), req))
    }
}
