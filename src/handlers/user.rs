use crate::auth::{create_jwt, verify_password};
use crate::errors::ApiError;
use crate::models::enums::UserRole;
use crate::models::user::{NewUser, UpdateUser, User, UserFilter, UserResponse};
use actix_web::{HttpResponse, web};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use validator::Validate;

/// Lista usuarios con filtros avanzados y paginación
///
/// # Parámetros (opcionales vía query string)
/// - `email`: Filtrar por correo electrónico
/// - `role`: Filtrar por rol
/// - `license_number`: Filtrar por número de licencia
/// - `is_active`: Filtrar por estado activo/inactivo
/// - `created_after`: Usuarios creados después de esta fecha
/// - `created_before`: Usuarios creados antes de esta fecha
/// - `limit`: Máximo de resultados (default: 50)
/// - `offset`: Desplazamiento (default: 0)
///
/// # Ejemplo
/// GET /users?role=admin&is_active=true&limit=10
#[actix_web::get("")]
async fn list_users(
    filters: web::Query<UserFilter>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Listando usuarios con filtros: {:?}", &filters);

    let users = sqlx::query_as!(
        User,
        r#"
        SELECT
            id,
            email,
            password_hash,
            name,
            role as "role: UserRole",
            license_number,
            is_active as "is_active!: bool",
            created_at as "created_at!: chrono::DateTime<chrono::Utc>",
            updated_at as "updated_at!: chrono::DateTime<chrono::Utc>"
        FROM users
        WHERE
            ($1::text IS NULL OR email ILIKE '%' || $1 || '%') AND
            ($2::user_role IS NULL OR role = $2) AND
            ($3::text IS NULL OR license_number = $3) AND
            ($4::bool IS NULL OR is_active = $4) AND
            ($5::timestamptz IS NULL OR created_at >= $5) AND
            ($6::timestamptz IS NULL OR created_at <= $6)
        ORDER BY created_at DESC
        LIMIT $7 OFFSET $8
        "#,
        filters.email,
        filters.role.clone() as Option<UserRole>,
        filters.license_number,
        filters.is_active,
        filters.created_after,
        filters.created_before,
        filters.limit.unwrap_or(50),
        filters.offset.unwrap_or(0)
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error al listar usuarios: {}", e);
        ApiError::InternalServerError("Error al obtener usuarios".into())
    })?;

    // Convertir a respuestas enriquecidas
    let responses: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();

    Ok(HttpResponse::Ok().json(responses))
}

/// Obtener un usuario por su ID
#[actix_web::get("/{id}")]
async fn get_user(id: web::Path<i32>, pool: web::Data<PgPool>) -> Result<HttpResponse, ApiError> {
    tracing::info!("Obteniendo usuario con ID: {}", &id);

    let user = sqlx::query_as!(
        User,
        r#"
        SELECT
            id,
            email,
            password_hash,
            name,
            role as "role: UserRole",
            license_number,
            is_active as "is_active!: bool",
            created_at as "created_at!: chrono::DateTime<chrono::Utc>",
            updated_at as "updated_at!: chrono::DateTime<chrono::Utc>"
        FROM users
        WHERE id = $1 AND is_active = true
        "#,
        id.clone()
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error al buscar usuario: {}", e);
        ApiError::InternalServerError("Error al obtener usuario".into())
    })?;

    match user {
        Some(rec) => {
            tracing::info!("Usuario {} encontrado", &id);
            Ok(HttpResponse::Ok().json(UserResponse::from(rec)))
        }
        None => {
            tracing::warn!("Usuario {} no encontrado", id);
            Err(ApiError::NotFound(format!(
                "Usuario con ID {} no encontrado",
                id
            )))
        }
    }
}

/// Crea un nuevo usuario
///
/// # Ejemplo de petición
/// ```json
/// {
///   "email": "nuevo@ejemplo.com",
///   "password": "contraseñaSegura123",
///   "name": "Nombre Usuario",
///   "role": "veterinarian",
///   "license_number": "LIC-12345"
/// }
/// ```
#[actix_web::post("")]
async fn create_user(
    new_user: web::Json<NewUser>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Creando nuevo usuario");

    // Validar los datos de entrada
    new_user.validate()?;

    let new_user = new_user.into_inner();

    // Verificar si el email ya existe
    let email_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
            .bind(&new_user.email)
            .fetch_one(pool.get_ref())
            .await?;

    if email_exists {
        tracing::warn!(
            "Intento de crear usuario con email existente: {:?}",
            new_user.email
        );
        return Err(ApiError::Conflict("El email ya está registrado".into()));
    }

    // Hashear la contraseña
    let password_hash = crate::auth::hash_password(&new_user.password).map_err(|e| {
        tracing::error!("Error al hashear contraseña: {}", e);
        ApiError::InternalServerError("Error al procesar contraseña".into())
    })?;

    // Insertar en la base de datos
    let user = sqlx::query_as!(
        User,
        r#"
            INSERT INTO users (
                email,
                password_hash,
                name,
                role,
                license_number,
                is_active
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING
                id,
                email,
                password_hash,
                name,
                role as "role!: UserRole",
                license_number,
                is_active as "is_active!: bool",
                created_at as "created_at!: chrono::DateTime<chrono::Utc>",
                updated_at as "updated_at!: chrono::DateTime<chrono::Utc>"
            "#,
        new_user.email.trim(),
        password_hash,
        new_user.name.trim(),
        new_user.role as UserRole, // Conversión explícita del enum
        new_user.license_number,
        true
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error al crear usuario: {}", e);
        ApiError::InternalServerError("Error al guardar usuario".into())
    })?;

    tracing::info!("Usuario creado exitosamente ID: {}", user.id);

    Ok(HttpResponse::Created()
        .append_header(("Location", format!("/users/{}", user.id)))
        .json(UserResponse::from(user)))
}

/// Actualiza un usuario existente (actualización parcial)
#[actix_web::put("/{id}")]
async fn update_user(
    id: web::Path<i32>,
    updated_user: web::Json<UpdateUser>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Actualizando usuario ID: {}", id);

    let updated_user = updated_user.into_inner();
    updated_user.validate()?;

    // Hashear contraseña solo si se proporcionó
    let password_hash = match updated_user.password {
        Some(password) => Some(crate::auth::hash_password(&password).map_err(|e| {
            tracing::error!("Error al hashear contraseña: {}", e);
            ApiError::InternalServerError("Error al procesar contraseña".into())
        })?),
        None => None,
    };

    let user = sqlx::query_as!(
        User,
        r#"
        UPDATE users SET
            email = COALESCE($1, email),
            password_hash = COALESCE($2, password_hash),
            name = COALESCE($3, name),
            role = COALESCE($4, role),
            license_number = COALESCE($5, license_number),
            is_active = COALESCE($6, is_active),
            updated_at = NOW()
        WHERE id = $7
        RETURNING
            id,
            email,
            password_hash,
            name,
            role as "role!: UserRole",
            license_number,
            is_active as "is_active!: bool",
            created_at as "created_at!: chrono::DateTime<Utc>",
            updated_at as "updated_at!: chrono::DateTime<Utc>"
        "#,
        updated_user.email,
        password_hash,
        updated_user.name,
        updated_user.role as Option<UserRole>,
        updated_user.license_number,
        updated_user.is_active,
        id.clone()
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error de base de datos: {}", e);
        match e {
            sqlx::Error::Database(err) if err.constraint() == Some("users_email_key") => {
                ApiError::Conflict("El email ya está en uso".into())
            }
            _ => ApiError::InternalServerError("Error al actualizar usuario".into()),
        }
    })?;

    match user {
        Some(user) => {
            tracing::info!("Usuario {} actualizado exitosamente", &user.id);
            Ok(HttpResponse::Ok().json(UserResponse::from(user)))
        }
        None => {
            tracing::warn!("Usuario {} no encontrado", &id);
            Err(ApiError::NotFound("Usuario no encontrado".into()))
        }
    }
}

/// Elimina un usuario (borrado lógico)
#[actix_web::delete("/{id}")]
async fn delete_user(
    id: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let result = sqlx::query!(
        r#"
        UPDATE users
        SET
            is_active = false,
            updated_at = NOW()
        WHERE id = $1 AND is_active = true
        RETURNING id, updated_at
        "#,
        id.clone()
    )
    .fetch_optional(pool.get_ref())
    .await?;

    match result {
        Some(user) => {
            tracing::info!("Usuario {} desactivado el {}", user.id, user.updated_at);
            Ok(HttpResponse::NoContent().finish())
        }
        None => {
            tracing::warn!("Usuario {} no encontrado o ya inactivo", id);
            Err(ApiError::NotFound("Usuario no encontrado".into()))
        }
    }
}

// Estructuras para login
#[derive(Debug, Deserialize, Validate)]
struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}

#[actix_web::post("/login")]
async fn login(
    pool: web::Data<PgPool>,
    login_request: web::Json<LoginRequest>,
) -> Result<impl actix_web::Responder, ApiError> {
    // Buscar usuario por email
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT
            id,
            email,
            password_hash,
            name,
            role as "role: UserRole",
            license_number,
            is_active as "is_active!: bool",
            created_at as "created_at!: chrono::DateTime<chrono::Utc>",
            updated_at as "updated_at!: chrono::DateTime<chrono::Utc>"
        FROM users
        WHERE email = $1 AND is_active = true
        "#,
        &login_request.email.trim()
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error al obtener usuario: {}", e);
        ApiError::InternalServerError("Error al obtener el usuario".into())
    })?;

    match user {
        Some(user) => {
            let is_valid_password = verify_password(&login_request.password, &user.password_hash)?;

            if !is_valid_password {
                return Err(ApiError::Unauthorized(format!("Contraseña invalida!")));
            }

            let token = create_jwt(user.id, &user.role)?;

            let response = LoginResponse {
                token,
                user: UserResponse::from(user),
            };

            Ok(HttpResponse::Ok().json(response))
        }
        None => Err(ApiError::Unauthorized(format!(
            "Correo o contraseña invalida"
        ))),
    }
}

// Exporta todas las funciones como un grupo
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .service(list_users)
            .service(get_user)
            .service(create_user)
            .service(update_user)
            .service(delete_user)
            .service(login), // Agrega más servicios aquí...
    );
}
