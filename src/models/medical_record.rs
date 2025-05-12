use bigdecimal::ToPrimitive;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, types::BigDecimal};
use validator::Validate;

#[derive(Debug, FromRow)]
pub struct MedicalRecordRaw {
    pub id: i32,
    pub patient_id: i32,
    pub veterinarian_id: i32,
    pub date: chrono::DateTime<chrono::Utc>,
    pub diagnosis: String,
    pub treatment: Option<String>,
    pub notes: Option<String>,
    pub weight_at_visit: Option<BigDecimal>,
}

impl From<MedicalRecordRaw> for MedicalRecord {
    fn from(raw: MedicalRecordRaw) -> Self {
        Self {
            id: raw.id,
            patient_id: raw.patient_id,
            veterinarian_id: raw.veterinarian_id,
            date: raw.date,
            diagnosis: raw.diagnosis,
            treatment: raw.treatment,
            notes: raw.notes,
            weight_at_visit: raw.weight_at_visit.and_then(|f| f.to_f64()),
        }
    }
}

/// Estructura completa del historial médico
#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct MedicalRecord {
    pub id: i32,
    pub patient_id: i32,
    pub veterinarian_id: i32,
    pub date: DateTime<Utc>,
    pub diagnosis: String,
    pub treatment: Option<String>,
    pub notes: Option<String>,
    pub weight_at_visit: Option<f64>, // Decimal(5,2) en SQL
}

/// Estructura para crear nuevo registro médico
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct NewMedicalRecord {
    #[validate(range(min = 1))]
    pub patient_id: i32,
    #[validate(range(min = 1))]
    pub veterinarian_id: i32,
    #[validate(length(min = 5, max = 2000))]
    pub diagnosis: String,
    #[validate(length(max = 2000))]
    pub treatment: Option<String>,
    #[validate(length(max = 2000))]
    pub notes: Option<String>,
    #[validate(range(min = 0.01, max = 999.99))]
    pub weight_at_visit: Option<f64>,
}

/// Estructura para actualizar registro médico
#[derive(Debug, Serialize, Deserialize, Validate, Default)]
pub struct UpdateMedicalRecord {
    #[validate(range(min = 1))]
    pub patient_id: Option<i32>,
    #[validate(range(min = 1))]
    pub veterinarian_id: Option<i32>,
    #[validate(length(min = 5, max = 2000))]
    pub diagnosis: Option<String>,
    #[validate(length(max = 2000))]
    pub treatment: Option<Option<String>>, // Some(None) para borrar
    #[validate(length(max = 2000))]
    pub notes: Option<Option<String>>, // Some(None) para borrar
    #[validate(range(min = 0.01, max = 999.99))]
    pub weight_at_visit: Option<Option<f64>>, // Some(None) para borrar
}

/// Estructura de respuesta para API
#[derive(Debug, Serialize)]
pub struct MedicalRecordResponse {
    pub id: i32,
    pub patient_id: i32,
    pub veterinarian_id: i32,
    pub veterinarian_name: String,
    pub date: DateTime<Utc>,
    pub diagnosis: String,
    pub treatment: Option<String>,
    pub notes: Option<String>,
    pub weight_at_visit: Option<f64>,
}

impl MedicalRecordResponse {
    /// Crea una respuesta a partir del registro médico y el nombre del veterinario
    pub fn from_record_with_vet(record: MedicalRecord, vet_name: String) -> Self {
        Self {
            id: record.id,
            patient_id: record.patient_id,
            veterinarian_id: record.veterinarian_id,
            veterinarian_name: vet_name,
            date: record.date,
            diagnosis: record.diagnosis,
            treatment: record.treatment,
            notes: record.notes,
            weight_at_visit: record.weight_at_visit,
        }
    }
}

/// Filtros para búsqueda de registros médicos
#[derive(Debug, Deserialize, Default)]
pub struct MedicalRecordFilter {
    pub patient_id: Option<i32>,
    pub veterinarian_id: Option<i32>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub diagnosis_contains: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
