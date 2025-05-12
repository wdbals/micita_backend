mod auth;
mod db;
mod errors;
mod handlers;
mod middleware;
mod models;
mod routes;

use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use actix_web_httpauth::middleware::HttpAuthentication;
use db::connect_to_db;
use tracing::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt().init();
    dotenv::dotenv().ok();

    info!("Iniciando el servidor");
    let allowed_origin =
        std::env::var("ALLOWED_ORIGIN").expect("ALLOWED_ORIGIN debe estar declarado");
    let db_pool = connect_to_db()
        .await
        .expect("Fallo la conexión a la base de datos");
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&allowed_origin)
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![actix_web::http::header::CONTENT_TYPE])
            .max_age(3600);

        let auth = HttpAuthentication::bearer(middleware::api_key_validator);

        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .wrap(cors)
            .wrap(auth)
            .configure(routes::config)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
