use crate::handlers;
use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api").configure(handlers::config), // Puedes agregar middleware global aquí
    );
}
