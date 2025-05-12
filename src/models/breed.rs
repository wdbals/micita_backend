use crate::models::enums::AnimalSpecies;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

/// Estructura para razas de animales
#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Breed {
    pub id: i32,
    pub species: AnimalSpecies,
    pub name: String,
}

/// Estructura para crear nueva raza
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct NewBreed {
    pub species: AnimalSpecies,
    #[validate(length(min = 3, max = 50))]
    pub name: String,
}

/// Estructura para respuesta API
#[derive(Debug, Serialize)]
pub struct BreedResponse {
    pub id: i32,
    pub species: AnimalSpecies,
    pub name: String,
}

impl From<Breed> for BreedResponse {
    fn from(breed: Breed) -> Self {
        Self {
            id: breed.id,
            species: breed.species,
            name: breed.name,
        }
    }
}
