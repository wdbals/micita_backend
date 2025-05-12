use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

/// Estructura completa del cliente (dueño de mascotas)
#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Client {
    pub id: i32,
    pub name: String,
    pub email: Option<String>,
    pub phone: String,
    pub address: Option<String>,
    pub notes: Option<String>,
    pub assigned_to: Option<i32>, // ID del usuario asignado (veterinario/asistente)
}

/// Estructura para crear un nuevo cliente
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct NewClient {
    #[validate(length(min = 3, max = 100))]
    pub name: String,
    #[validate(email, length(max = 255))]
    pub email: Option<String>,
    #[validate(length(min = 10, max = 20))] // Ajusta según requisitos de teléfono
    pub phone: String,
    #[validate(length(max = 500))]
    pub address: Option<String>,
    #[validate(length(max = 1000))]
    pub notes: Option<String>,
    pub assigned_to: Option<i32>, // Validado en la DB via trigger
}

/// Estructura para actualizar cliente
#[derive(Debug, Serialize, Deserialize, Validate, Default)]
pub struct UpdateClient {
    #[validate(length(min = 3, max = 100))]
    pub name: Option<String>,
    #[validate(email, length(max = 255))]
    pub email: Option<String>, // Option<Option> para permitir null
    #[validate(length(min = 10, max = 20))]
    pub phone: Option<String>,
    #[validate(length(max = 500))]
    pub address: Option<String>, // Puede ser Some(null) para borrar
    #[validate(length(max = 1000))]
    pub notes: Option<String>, // Puede ser Some(null) para borrar
    pub assigned_to: Option<Option<i32>>, // Some(None) para desasignar
}

/// Estructura de respuesta simplificada para el cliente
#[derive(Debug, Serialize)]
pub struct ClientResponse {
    pub id: i32,
    pub name: String,
    pub email: Option<String>,
    pub phone: String,
    pub assigned_to: Option<i32>,
}

impl From<Client> for ClientResponse {
    fn from(client: Client) -> Self {
        Self {
            id: client.id,
            name: client.name,
            email: client.email,
            phone: client.phone,
            assigned_to: client.assigned_to,
        }
    }
}

/// Estructura para búsqueda/filtrado de clientes
#[derive(Debug, Deserialize, Default)]
pub struct ClientFilter {
    pub name: Option<String>,
    pub phone: Option<String>,
    pub assigned_to: Option<i32>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
