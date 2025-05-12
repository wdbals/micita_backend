use crate::errors::ApiError;
use crate::models::client::{Client, ClientFilter, ClientResponse, NewClient, UpdateClient};
use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use validator::Validate;

// /// Parámetros de paginación
// #[derive(Debug, Deserialize)]
// pub struct PaginationParams {
//     limit: Option<i64>,
//     offset: Option<i64>,
// }

// /// Lista todos los clientes con paginación básica.
// ///
// /// # Parámetros (opcionales vía query string)
// /// - `limit`: Límite de resultados (default: 100)
// /// - `offset`: Desplazamiento (default: 0)
// ///
// /// # Ejemplo
// /// GET /clients?limit=10&offset=20
// #[actix_web::get("")]
// async fn list_clients(
//     pool: web::Data<PgPool>,
//     query: web::Query<PaginationParams>,
// ) -> Result<HttpResponse, ApiError> {
//     tracing::info!("Listando clientes con parámetros: {:?}", query);

//     let users = sqlx::query_as!(
//         Client,
//         r#"
//         SELECT
//             id,
//             name,
//             email,
//             phone,
//             address,
//             notes,
//             assigned_to
//         FROM clients
//         LIMIT $1 OFFSET $2
//         "#,
//         query.limit.unwrap_or(100),
//         query.offset.unwrap_or(0)
//     )
//     .fetch_all(pool.get_ref())
//     .await
//     .map_err(|e| {
//         tracing::error!("Error al listar clientes: {}", e);
//         ApiError::InternalServerError("No se pudieron obtener los clientes".into())
//     })?
//     .into_iter()
//     .collect::<Vec<Client>>();

//     let users_response: Vec<ClientResponse> = users.into_iter().map(ClientResponse::from).collect();
//     Ok(HttpResponse::Ok().json(users_response))
// }

/// Lista todos los clientes con filtros avanzados y paginación.
///
/// # Parámetros (opcionales vía query string)
/// - `name`: Filtrar por nombre (búsqueda parcial insensible a mayúsculas/minúsculas)
/// - `phone`: Filtrar por número de teléfono exacto
/// - `assigned_to`: Filtrar por ID del usuario asignado
/// - `limit`: Máximo de resultados (default: 50)
/// - `offset`: Desplazamiento (default: 0)
///
/// # Ejemplo
/// GET /clients?name=Juan&phone=1234567890&assigned_to=3&limit=10&offset=0
#[actix_web::get("")]
async fn list_clients(
    filters: web::Query<ClientFilter>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Listando clientes con filtros: {:?}", &filters);

    let clients = sqlx::query_as!(
        Client,
        r#"
        SELECT
            id,
            name,
            email,
            phone,
            address,
            notes,
            assigned_to
        FROM clients
        WHERE
            ($1::text IS NULL OR name ILIKE '%' || $1 || '%') AND
            ($2::text IS NULL OR phone = $2) AND
            ($3::int IS NULL OR assigned_to = $3)
        ORDER BY name ASC
        LIMIT $4 OFFSET $5
        "#,
        filters.name.as_deref(),
        filters.phone.as_deref(),
        filters.assigned_to,
        filters.limit.unwrap_or(50),
        filters.offset.unwrap_or(0)
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error al listar clientes: {}", e);
        ApiError::InternalServerError("Error al obtener clientes".into())
    })?
    .into_iter()
    .collect::<Vec<Client>>();

    // Convertir a respuestas simplificadas
    let clients_response: Vec<ClientResponse> =
        clients.into_iter().map(ClientResponse::from).collect();

    Ok(HttpResponse::Ok().json(clients_response))
}

/// Obtener un cliente por su ID
#[actix_web::get("/{id}")]
async fn get_client(id: web::Path<i32>, pool: web::Data<PgPool>) -> Result<HttpResponse, ApiError> {
    tracing::info!("Obteniendo cliente con ID: {}", &id);

    let user = sqlx::query_as!(
        Client,
        r#"
        SELECT
            id,
            name,
            email,
            phone,
            address,
            notes,
            assigned_to
        FROM clients
        WHERE id = $1
        "#,
        id.clone()
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error al buscar Cliente: {}", e);
        ApiError::InternalServerError("Error al obtener cliente".into())
    })?;

    match user {
        Some(rec) => {
            tracing::info!("Cliente {} encontrado", &id);
            Ok(HttpResponse::Ok().json(ClientResponse::from(rec)))
        }
        None => {
            tracing::warn!("Cliente {} no encontrado", id);
            Err(ApiError::NotFound(format!(
                "Cliente con ID {} no encontrado",
                id
            )))
        }
    }
}

/// Crea un nuevo Cliente
///
/// # Ejemplo de petición
/// ```json
/// {
///   "name": "Nombre Cliente",
///   "email": "nuevo@ejemplo.com",
///   "phone": "1231231212",
///   "address": "C XX N XX",
///   "notes": "LIC-12345",
///   "assigned_to": 1,
/// }
/// ```
#[actix_web::post("")]
async fn create_client(
    new_client: web::Json<NewClient>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Creando nuevo cliente");

    // Validar los datos de entrada
    new_client.validate()?;

    let new_client = new_client.into_inner();

    // Verificar si el email ya existe
    let email_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM clients WHERE email = $1)")
            .bind(&new_client.email)
            .fetch_one(pool.get_ref())
            .await?;

    if email_exists {
        tracing::warn!(
            "Intento de crear cliente con email existente: {:?}",
            new_client.email
        );
        return Err(ApiError::Conflict("El email ya está registrado".into()));
    }

    // Insertar en la base de datos
    let user = sqlx::query_as!(
        Client,
        r#"
            INSERT INTO clients (
                name,
                email,
                phone,
                address,
                notes,
                assigned_to
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING
                id,
                name,
                email,
                phone,
                address,
                notes,
                assigned_to
            "#,
        new_client.name.trim(),
        new_client.email.map(|s| s.trim().to_string()),
        new_client.phone,
        new_client.address.map(|s| s.trim().to_string()),
        new_client.notes.map(|s| s.trim().to_string()),
        new_client.assigned_to
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error al crear cliente: {}", e);
        ApiError::InternalServerError("Error al guardar cliente".into())
    })?;

    tracing::info!("Cliente creado exitosamente ID: {}", user.id);

    Ok(HttpResponse::Created()
        .append_header(("Location", format!("/users/{}", user.id)))
        .json(ClientResponse::from(user)))
}

/// Actualiza un cliente existente (actualización parcial)
#[actix_web::put("/{id}")]
async fn update_client(
    id: web::Path<i32>,
    updated_client: web::Json<UpdateClient>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Actualizando cliente ID: {}", id);

    let updated_client = updated_client.into_inner();
    updated_client.validate()?;

    // Manejo especial para Option<Option> fields
    let email = match updated_client.email {
        Some(inner) => inner, // Some(email) o None (para setear NULL)
        None => {
            return Err(ApiError::ValidationError(
                "Email no puede ser removido, solo actualizado".into(),
            ));
        }
    };

    let assigned_to = match updated_client.assigned_to {
        Some(inner) => inner, // Some(user_id) para asignar o None para desasignar
        None => None,         // Mantener valor existente
    };

    let client = sqlx::query_as!(
        Client,
        r#"
        UPDATE clients SET
            name = COALESCE($1, name),
            email = CASE WHEN $2::TEXT IS NOT NULL THEN $2 ELSE email END,
            phone = COALESCE($3, phone),
            address = CASE WHEN $4::TEXT IS NOT NULL THEN $4 ELSE address END,
            notes = CASE WHEN $5::TEXT IS NOT NULL THEN $5 ELSE notes END,
            assigned_to = $6  -- Manejo directo del Option<Option>
        WHERE id = $7
        RETURNING
            id,
            name,
            email,
            phone,
            address,
            notes,
            assigned_to
        "#,
        updated_client.name,
        email, // Option<String>
        updated_client.phone,
        updated_client.address, // Option<String> (Some(null) será NULL)
        updated_client.notes,   // Option<String> (Some(null) será NULL)
        assigned_to,            // Option<i32>
        id.clone()
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error de base de datos: {}", e);
        match e {
            sqlx::Error::Database(err) if err.constraint() == Some("clients_email_key") => {
                ApiError::Conflict("El email ya está registrado".into())
            }
            sqlx::Error::Database(err) if err.constraint() == Some("clients_assigned_to_fkey") => {
                ApiError::ValidationError("El usuario asignado no existe".into())
            }
            _ => ApiError::InternalServerError("Error al actualizar cliente".into()),
        }
    })?;

    match client {
        Some(client) => {
            tracing::info!("Cliente {} actualizado exitosamente", client.id);
            Ok(HttpResponse::Ok().json(ClientResponse::from(client)))
        }
        None => {
            tracing::warn!("Cliente {} no encontrado", &id);
            Err(ApiError::NotFound("Cliente no encontrado".into()))
        }
    }
}

/// Elimina un cliente
#[actix_web::delete("/{id}")]
async fn delete_client_hard(
    id: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    // Verificar dependencias primero
    let has_deps: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM patients WHERE client_id = $1)")
            .bind(id.clone())
            .fetch_one(pool.get_ref())
            .await?;

    if has_deps {
        return Err(ApiError::Conflict(
            "No se puede eliminar, el cliente tiene mascotas registradas".into(),
        ));
    }

    sqlx::query!("DELETE FROM clients WHERE id = $1", id.clone())
        .execute(pool.get_ref())
        .await?;

    Ok(HttpResponse::NoContent().finish())
}

// Exporta todas las funciones como un grupo
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/clients")
            .service(list_clients)
            .service(get_client)
            .service(create_client)
            .service(update_client)
            .service(delete_client_hard), // Agrega más servicios aquí...
    );
}
