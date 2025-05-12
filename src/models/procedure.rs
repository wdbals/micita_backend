use crate::models::enums::ProcedureType;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

/// Estructura completa para procedimientos médicos
#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Procedure {
    pub id: i32,
    pub name: String,
    #[serde(rename = "type")] // Para coincidir con 'type' en la DB (opcional)
    pub procedure_type: ProcedureType,
    pub description: Option<String>,
    pub duration_minutes: Option<i32>,
}

/// Estructura para crear nuevo procedimiento
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct NewProcedure {
    #[validate(length(min = 2, max = 100))]
    pub name: String,
    pub procedure_type: ProcedureType,
    #[validate(length(max = 500))]
    pub description: Option<String>,
    #[validate(range(min = 1, max = 1440))] // 1 minuto a 24 horas
    pub duration_minutes: Option<i32>,
}

/// Estructura para actualizar procedimiento
#[derive(Debug, Serialize, Deserialize, Validate, Default)]
pub struct UpdateProcedure {
    #[validate(length(min = 2, max = 100))]
    pub name: Option<String>,
    pub procedure_type: Option<ProcedureType>,
    #[validate(length(max = 500))]
    pub description: Option<Option<String>>, // Some(None) para borrar
    #[validate(range(min = 1, max = 1440))]
    pub duration_minutes: Option<Option<i32>>, // Some(None) para borrar
}

/// Estructura de respuesta para API
#[derive(Debug, Serialize)]
pub struct ProcedureResponse {
    pub id: i32,
    pub name: String,
    pub procedure_type: ProcedureType,
    pub description: Option<String>,
    pub duration_minutes: Option<i32>,
    pub duration_formatted: Option<String>, // Ej: "2 horas 30 minutos"
}

impl ProcedureResponse {
    /// Formatea la duración en minutos a texto legible
    pub fn format_duration(minutes: Option<i32>) -> Option<String> {
        minutes.map(|mins| {
            let hours = mins / 60;
            let remaining_minutes = mins % 60;

            match (hours, remaining_minutes) {
                (0, m) => format!("{m} minutos"),
                (h, 0) => format!("{h} horas"),
                (h, m) => format!("{h} horas {m} minutos"),
            }
        })
    }
}

impl From<Procedure> for ProcedureResponse {
    fn from(procedure: Procedure) -> Self {
        Self {
            id: procedure.id,
            name: procedure.name,
            procedure_type: procedure.procedure_type,
            description: procedure.description,
            duration_minutes: procedure.duration_minutes,
            duration_formatted: Self::format_duration(procedure.duration_minutes),
        }
    }
}

/// Filtros para búsqueda de procedimientos
#[derive(Debug, Deserialize, Default)]
pub struct ProcedureFilter {
    pub name_contains: Option<String>,
    pub procedure_type: Option<ProcedureType>,
    pub min_duration: Option<i32>,
    pub max_duration: Option<i32>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
