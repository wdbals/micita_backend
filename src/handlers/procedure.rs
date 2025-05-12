use actix_web::web;

// Exporta todas las funciones como un grupo
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/procedures"), // Agrega más servicios aquí...
    );
}
