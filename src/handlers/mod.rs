mod appointment;
mod breed;
mod client;
mod medical_record;
mod patient;
mod patient_procedure;
mod procedure;
mod statistic;
mod user;

/// Configura todas las rutas de los Handlers
pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    appointment::config(cfg);
    breed::config(cfg);
    client::config(cfg);
    medical_record::config(cfg);
    patient::config(cfg);
    patient_procedure::config(cfg);
    procedure::config(cfg);
    statistic::config(cfg);
    user::config(cfg);
    // ... otros configs
}
