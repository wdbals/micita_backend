use crate::models::enums::{AnimalGender, AnimalSpecies};
use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

/// Estructura completa del paciente (mascota)
#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Patient {
    pub id: i32,
    pub name: String,
    pub species: AnimalSpecies,
    pub breed_id: Option<i32>, // Referencia a breeds.id
    pub birth_date: Option<NaiveDate>,
    pub gender: Option<AnimalGender>,
    pub weight_kg: Option<f64>, // Decimal(5,2) en SQL se mapea a f64
    pub client_id: i32,
    pub photo_url: Option<String>,
}

/// Estructura intermedia para manejar datos directamente desde la base de datos
#[derive(Debug, FromRow)]
pub struct PatientRaw {
    pub id: i32,
    pub name: String,
    pub species: AnimalSpecies,
    pub breed_id: Option<i32>, // Referencia a breeds.id
    pub birth_date: Option<NaiveDate>,
    pub gender: Option<AnimalGender>,
    pub weight_kg: Option<BigDecimal>, // Usamos BigDecimal aquí
    pub client_id: i32,
    pub photo_url: Option<String>,
}

impl From<PatientRaw> for Patient {
    fn from(raw: PatientRaw) -> Self {
        Self {
            id: raw.id,
            name: raw.name,
            species: raw.species,
            breed_id: raw.breed_id,
            birth_date: raw.birth_date,
            gender: raw.gender,
            weight_kg: raw.weight_kg.and_then(|f| f.to_f64()), // Conversión explícita
            client_id: raw.client_id,
            photo_url: raw.photo_url,
        }
    }
}

/// Estructura para crear nuevo paciente
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct NewPatient {
    #[validate(length(min = 2, max = 100))]
    pub name: String,
    pub species: AnimalSpecies,
    pub breed_id: Option<i32>, // Validado contra species via trigger
    pub birth_date: Option<NaiveDate>,
    pub gender: Option<AnimalGender>,
    #[validate(range(min = 0.01, max = 999.99))]
    pub weight_kg: Option<f64>,
    pub client_id: i32, // Validar existencia en DB
    #[validate(url, length(max = 512))]
    pub photo_url: Option<String>,
}

/// Estructura para actualizar paciente
#[derive(Debug, Serialize, Deserialize, Validate, Default)]
pub struct UpdatePatient {
    #[validate(length(min = 2, max = 100))]
    pub name: Option<String>,
    pub species: Option<AnimalSpecies>, // Si cambia, debe validarse con breed_id
    pub breed_id: Option<Option<i32>>,  // Some(None) para quitar raza
    pub birth_date: Option<NaiveDate>,  // Some(None) para borrar
    pub gender: Option<AnimalGender>,   // Some(None) para borrar
    #[validate(range(min = 0.01, max = 999.99))]
    pub weight_kg: Option<f64>, // Some(None) para borrar
    pub client_id: Option<i32>,
    #[validate(url, length(max = 512))]
    pub photo_url: Option<String>, // Some(None) para borrar
}

/// Estructura de respuesta para API
#[derive(Debug, Serialize)]
pub struct PatientResponse {
    pub id: i32,
    pub name: String,
    pub species: AnimalSpecies,
    pub breed: Option<String>, // Nombre de la raza
    pub breed_id: Option<i32>,
    pub birth_date: Option<NaiveDate>,
    pub gender: Option<AnimalGender>,
    pub weight_kg: Option<f64>,
    pub client_id: i32,
    pub photo_url: Option<String>,
}

impl From<Patient> for PatientResponse {
    fn from(patient: Patient) -> Self {
        Self {
            id: patient.id,
            name: patient.name,
            species: patient.species,
            breed: None, // Se llenará después si es necesario
            breed_id: patient.breed_id,
            birth_date: patient.birth_date,
            gender: patient.gender,
            weight_kg: patient.weight_kg,
            client_id: patient.client_id,
            photo_url: patient.photo_url,
        }
    }
}

/// Filtros para búsqueda de pacientes
#[derive(Debug, Deserialize, Default)]
pub struct PatientFilter {
    pub name: Option<String>,
    pub species: Option<AnimalSpecies>,
    pub breed_id: Option<i32>,
    pub client_id: Option<i32>,
    pub gender: Option<AnimalGender>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
