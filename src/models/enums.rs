use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Debug, Type, Serialize, Deserialize, Clone)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Veterinarian,
    Assistant,
    Admin,
}

#[derive(Debug, Type, Serialize, Deserialize, Clone)]
#[sqlx(type_name = "animal_species", rename_all = "lowercase")]
pub enum AnimalSpecies {
    Dog,
    Cat,
    Bird,
    Reptile,
    Rodent,
    Rabbit,
    Other,
}

#[derive(Debug, Type, Serialize, Deserialize, Clone)]
#[sqlx(type_name = "animal_gender", rename_all = "lowercase")]
pub enum AnimalGender {
    Male,
    Female,
    Unknown,
}

#[derive(Debug, Type, Serialize, Deserialize, Clone)]
#[sqlx(type_name = "appointment_status", rename_all = "snake_case")]
pub enum AppointmentStatus {
    Scheduled,
    Completed,
    Canceled,
    NoShow,
}

#[derive(Debug, Type, Serialize, Deserialize, Clone)]
#[sqlx(type_name = "procedure_type", rename_all = "lowercase")]
pub enum ProcedureType {
    Vaccine,
    Surgery,
    Deworming,
    Test,
    Grooming,
    Other,
}
