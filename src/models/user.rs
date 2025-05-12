use crate::models::enums::UserRole;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

/// Estructura para usuario
#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i32,
    #[serde(skip_serializing)] // No exponer el hash en respuestas
    pub password_hash: String,
    pub email: String,
    pub name: String,
    pub role: UserRole,
    pub license_number: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Estructura para crear un nuevo usuario
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct NewUser {
    #[validate(email, length(max = 255))]
    pub email: String,
    #[validate(length(min = 8, max = 72))] // Longitud típica para bcrypt
    #[serde(skip_serializing)] // Nunca debería mostrarse
    pub password: String,
    #[validate(length(min = 2, max = 100))]
    pub name: String,
    pub role: UserRole,
    #[validate(length(max = 50))]
    pub license_number: Option<String>,
}

/// Estructura para actualizar usuario
#[derive(Debug, Serialize, Deserialize, Validate, Default)]
pub struct UpdateUser {
    #[validate(email, length(max = 255))]
    pub email: Option<String>,
    #[validate(length(min = 8, max = 72))]
    #[serde(skip_serializing)]
    pub password: Option<String>,
    #[validate(length(min = 2, max = 100))]
    pub name: Option<String>,
    pub role: Option<UserRole>,
    #[validate(length(max = 50))]
    pub license_number: Option<String>,
    pub is_active: Option<bool>,
}

/// Estructura para respuesta pública de usuario
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub email: String,
    pub name: String,
    pub role: UserRole,
    pub license_number: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user.role,
            license_number: user.license_number,
            is_active: user.is_active,
            created_at: user.created_at,
        }
    }
}

/// Filtros para búsqueda de usuarios
#[derive(Debug, Deserialize, Default)]
pub struct UserFilter {
    pub email: Option<String>,
    pub role: Option<UserRole>,
    pub license_number: Option<String>,
    pub is_active: Option<bool>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Estructura para login
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LoginUser {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}
