use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::{Validate, ValidationError};

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
