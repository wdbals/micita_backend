use crate::models::enums::AppointmentStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::{Validate, ValidationError};

/// Estructura completa para citas
#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Appointment {
    pub id: i32,
    pub patient_id: Option<i32>,
    pub client_id: Option<i32>,
    pub veterinarian_id: i32,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub status: AppointmentStatus,
    pub reason: String,
}

/// Estructura para crear nueva cita
#[derive(Debug, Serialize, Deserialize, Validate)]
#[validate(schema(function = "validate_appointment_times"))]
pub struct NewAppointment {
    #[validate(range(min = 1))]
    pub patient_id: Option<i32>,
    #[validate(range(min = 1))]
    pub client_id: Option<i32>,
    #[validate(range(min = 1))]
    pub veterinarian_id: i32,
    #[validate(custom(function = "validate_future_datetime"))]
    pub start_time: DateTime<Utc>,
    #[validate(custom(function = "validate_future_datetime"))]
    pub end_time: DateTime<Utc>,
    #[validate(length(min = 5, max = 500))]
    pub reason: String,
}

/// Estructura para actualizar cita
#[derive(Debug, Serialize, Deserialize, Validate, Default)]
pub struct UpdateAppointment {
    #[validate(range(min = 1))]
    pub patient_id: Option<Option<i32>>, // Some(None) para desasociar
    #[validate(range(min = 1))]
    pub client_id: Option<Option<i32>>, // Some(None) para desasociar
    #[validate(range(min = 1))]
    pub veterinarian_id: Option<i32>,
    pub start_time: Option<DateTime<Utc>>,
    #[validate(custom(function = "validate_future_datetime"))]
    pub end_time: Option<DateTime<Utc>>,
    pub status: Option<AppointmentStatus>,
    #[validate(length(min = 5, max = 500))]
    pub reason: Option<String>,
}

/// Valida que la fecha/hora sea en el futuro
pub fn validate_future_datetime(dt: &DateTime<Utc>) -> Result<(), ValidationError> {
    if dt < &Utc::now() {
        return Err(ValidationError::new("La fecha/hora debe ser en el futuro"));
    }
    Ok(())
}

/// Valida la relación entre start_time y end_time
pub fn validate_appointment_times(appointment: &NewAppointment) -> Result<(), ValidationError> {
    // Validar que end_time > start_time
    if appointment.end_time <= appointment.start_time {
        return Err(ValidationError::new(
            "La hora de fin debe ser posterior a la de inicio",
        ));
    }

    // Validar duración mínima (5 minutos)
    let duration = appointment.end_time - appointment.start_time;
    if duration.num_minutes() < 5 {
        return Err(ValidationError::new(
            "La cita debe durar al menos 5 minutos",
        ));
    }

    // Validar duración máxima (4 horas)
    if duration.num_hours() > 4 {
        return Err(ValidationError::new(
            "La cita no puede durar más de 4 horas",
        ));
    }

    Ok(())
}

/// Estructura de respuesta enriquecida para API
#[derive(Debug, Serialize)]
pub struct AppointmentResponse {
    pub id: i32,
    pub patient_id: Option<i32>,
    pub patient_name: Option<String>,
    pub client_id: Option<i32>,
    pub client_name: Option<String>,
    pub veterinarian_id: i32,
    pub veterinarian_name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub status: AppointmentStatus,
    pub reason: String,
    pub duration_minutes: i64,
}

impl AppointmentResponse {
    /// Crea una respuesta enriquecida a partir de una cita
    pub async fn from_appointment(
        appointment: Appointment,
        pool: &sqlx::PgPool,
    ) -> Result<Self, sqlx::Error> {
        let record = sqlx::query!(
            r#"
            SELECT
                p.name as patient_name,
                c.name as client_name,
                u.name as vet_name
            FROM users u
            LEFT JOIN patients p ON p.id = $1
            LEFT JOIN clients c ON c.id = $2
            WHERE u.id = $3
            "#,
            appointment.patient_id,
            appointment.client_id,
            appointment.veterinarian_id,
        )
        .fetch_one(pool)
        .await?;

        let (patient_name, client_name, vet_name) =
            (record.patient_name, record.client_name, record.vet_name);

        let duration = appointment.end_time - appointment.start_time;

        Ok(Self {
            id: appointment.id,
            patient_id: appointment.patient_id,
            patient_name: Some(patient_name),
            client_id: appointment.client_id,
            client_name: Some(client_name),
            veterinarian_id: appointment.veterinarian_id,
            veterinarian_name: vet_name,
            start_time: appointment.start_time,
            end_time: appointment.end_time,
            status: appointment.status,
            reason: appointment.reason,
            duration_minutes: duration.num_minutes(),
        })
    }
}

/// Filtros para búsqueda de citas
#[derive(Debug, Deserialize, Default)]
pub struct AppointmentFilter {
    pub patient_id: Option<i32>,
    pub client_id: Option<i32>,
    pub veterinarian_id: Option<i32>,
    pub status: Option<AppointmentStatus>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub reason_contains: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
