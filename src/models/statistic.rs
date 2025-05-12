use crate::models::enums::UserRole;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct StatisticsQuery {
    pub role: UserRole,       // "admin" o "veterinarian"
    pub user_id: Option<i32>, // Solo relevante si role = "veterinarian"
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub type_: Option<String>, // "appointments", "users", "procedures", etc.
}

#[derive(Debug, Serialize)]
pub struct StatisticsResponse {
    pub appointments_by_month: Option<Vec<AppointmentsByMonth>>,
    pub user_counts: Option<UserCounts>,
    pub procedures_by_type: Option<Vec<ProceduresByType>>,
    pub patients_by_species: Option<Vec<PatientsBySpecies>>,
    pub veterinarian_stats: Option<VeterinarianStats>,
}

#[derive(Debug, Serialize)]
pub struct AppointmentsByMonth {
    pub month: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct UserCounts {
    pub total_users: i64,
    pub veterinarians: i64,
    pub assistants: i64,
    pub admins: i64,
}

#[derive(Debug, Serialize)]
pub struct ProceduresByType {
    pub procedure_type: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct PatientsBySpecies {
    pub species: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct VeterinarianStats {
    pub appointments_by_status: Vec<AppointmentsByStatus>,
    pub procedures_performed: Vec<ProceduresByType>,
    pub medical_records_created: i64,
    pub patients_attended: Vec<PatientsBySpecies>,
}

#[derive(Debug, Serialize)]
pub struct AppointmentsByStatus {
    pub status: String,
    pub count: i64,
}
