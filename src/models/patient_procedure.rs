use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use validator::{Validate, ValidationError};

use crate::errors::ApiError;

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct PatientProcedure {
    pub id: i32,
    pub patient_id: i32,
    pub procedure_id: i32,
    pub veterinarian_id: Option<i32>,
    pub date: NaiveDate,
    pub next_due_date: Option<NaiveDate>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[validate(schema(function = "validate_date_pair"))]
pub struct NewPatientProcedure {
    #[validate(range(min = 1))]
    pub patient_id: i32,
    #[validate(range(min = 1))]
    pub procedure_id: i32,
    #[validate(range(min = 1))]
    pub veterinarian_id: Option<i32>,
    #[validate(custom(function = "validate_not_past_date"))]
    pub date: NaiveDate,
    #[validate(custom(function = "validate_next_due_date"))]
    pub next_due_date: Option<NaiveDate>,
    #[validate(length(max = 1000))]
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdatePatientProcedure {
    #[validate(range(min = 1))]
    pub patient_id: Option<i32>,
    #[validate(range(min = 1))]
    pub procedure_id: Option<i32>,
    #[validate(range(min = 1))]
    pub veterinarian_id: Option<Option<i32>>, // Some(None) para borrar
    #[validate(custom(function = "validate_not_past_date"))]
    pub date: Option<NaiveDate>,
    #[validate(custom(function = "validate_next_due_date"))]
    pub next_due_date: Option<Option<NaiveDate>>, // Some(None) para borrar
    #[validate(length(max = 1000))]
    pub notes: Option<Option<String>>, // Some(None) para borrar
}

/// Valida que la fecha no sea en el pasado
pub fn validate_not_past_date(date: &NaiveDate) -> Result<(), ValidationError> {
    if date < &Utc::now().date_naive() {
        return Err(ValidationError::new("La fecha no puede ser en el pasado"));
    }
    Ok(())
}

/// Valida que next_due_date no sea en el pasado y sea posterior a date
pub fn validate_next_due_date(next_due_date: &&NaiveDate) -> Result<(), ValidationError> {
    let date = **next_due_date;

    if date < Utc::now().date_naive() {
        return Err(ValidationError::new(
            "La fecha de próximo vencimiento no puede ser en el pasado",
        ));
    }
    Ok(())
}

/// Valida el par date/next_due_date juntos
pub fn validate_date_pair(procedure: &NewPatientProcedure) -> Result<(), ValidationError> {
    if let Some(next_date) = procedure.next_due_date {
        if next_date < procedure.date {
            return Err(ValidationError::new(
                "La fecha de próximo vencimiento debe ser posterior a la fecha del procedimiento",
            ));
        }
    }
    Ok(())
}

/// Filtros para búsqueda de procedimientos
#[derive(Debug, Deserialize, Default)]
pub struct PatientProcedureFilter {
    pub patient_id: Option<i32>,       // Filtrar por ID del paciente
    pub procedure_id: Option<i32>,     // Filtrar por ID del procedimiento
    pub veterinarian_id: Option<i32>,  // Filtrar por ID del veterinario
    pub start_date: Option<NaiveDate>, // Filtrar por fecha mínima
    pub end_date: Option<NaiveDate>,   // Filtrar por fecha máxima
    pub limit: Option<i64>,            // Máximo de resultados (default: 50)
    pub offset: Option<i64>,           // Desplazamiento (default: 0)
}

/// Estructura de respuesta para API
#[derive(Debug, Serialize)]
pub struct PatientProcedureResponse {
    pub id: i32,
    pub patient_id: i32,                   // ID del paciente
    pub patient_name: String,              // Nombre del paciente
    pub procedure_id: i32,                 // ID del procedimiento
    pub procedure_name: String,            // Nombre del procedimiento
    pub veterinarian_id: Option<i32>,      // ID del veterinario (opcional)
    pub veterinarian_name: Option<String>, // Nombre del veterinario (opcional)
    pub date: NaiveDate,
    pub next_due_date: Option<NaiveDate>,
    pub notes: Option<String>,
}

impl PatientProcedureResponse {
    pub async fn from_procedure(
        procedure: PatientProcedure,
        pool: &PgPool,
    ) -> Result<Self, ApiError> {
        // Obtener el nombre del paciente
        let patient_name: String = sqlx::query_scalar!(
            r#"
            SELECT name
            FROM patients
            WHERE id = $1
            "#,
            procedure.patient_id
        )
        .fetch_optional(pool)
        .await?
        .unwrap_or_else(|| "Unknown Patient".to_string());

        // Obtener el nombre del procedimiento
        let procedure_name: String = sqlx::query_scalar!(
            r#"
            SELECT name
            FROM procedures
            WHERE id = $1
            "#,
            procedure.procedure_id
        )
        .fetch_optional(pool)
        .await?
        .unwrap_or_else(|| "Unknown Procedure".to_string());

        // Obtener el nombre del veterinario
        let veterinarian_name: Option<String> =
            if let Some(veterinarian_id) = procedure.veterinarian_id {
                sqlx::query_scalar!(
                    r#"
                SELECT name
                FROM users
                WHERE id = $1
                "#,
                    veterinarian_id
                )
                .fetch_optional(pool)
                .await?
            } else {
                None
            };

        Ok(Self {
            id: procedure.id,
            patient_id: procedure.patient_id,
            patient_name,
            procedure_id: procedure.procedure_id,
            procedure_name,
            veterinarian_id: procedure.veterinarian_id,
            veterinarian_name,
            date: procedure.date,
            next_due_date: procedure.next_due_date,
            notes: procedure.notes,
        })
    }
}
