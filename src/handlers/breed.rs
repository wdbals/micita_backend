use crate::errors::ApiError;
use crate::models::breed::{Breed, BreedResponse, NewBreed};
use crate::models::enums::AnimalSpecies;

use actix_web::{HttpResponse, web};
use serde::Deserialize;
use sqlx::PgPool;
use validator::Validate;

/// Crea una nueva raza
///
/// # Ejemplo de petición
/// ```json
/// {
///   "species": "Dog",
///   "name": "Labrador Retriever"
/// }
/// ```
#[actix_web::post("")]
async fn create_breed(
    new_breed: web::Json<NewBreed>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Creando nueva raza");

    // Validar los datos de entrada
    let new_breed = new_breed.into_inner();
    new_breed.validate()?;

    // Verificar si la combinación de especie y nombre ya existe
    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM breeds
            WHERE species = $1 AND name ILIKE $2
        )
        "#,
    )
    .bind(&new_breed.species as &AnimalSpecies)
    .bind(new_breed.name.trim())
    .fetch_one(pool.get_ref())
    .await?;

    if exists {
        tracing::warn!(
            "Intento de crear raza duplicada: {} ({:?})",
            new_breed.name,
            new_breed.species
        );
        return Err(ApiError::Conflict("La raza ya existe".into()));
    }

    // Insertar la nueva raza en la base de datos
    let breed = sqlx::query_as!(
        Breed,
        r#"
        INSERT INTO breeds (species, name)
        VALUES ($1, $2)
        RETURNING id, species as "species!: AnimalSpecies", name
        "#,
        new_breed.species as AnimalSpecies,
        new_breed.name.trim()
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error al crear raza: {}", e);
        ApiError::InternalServerError("Error al guardar la raza".into())
    })?;

    tracing::info!("Raza creada exitosamente ID: {}", breed.id);

    Ok(HttpResponse::Created()
        .append_header(("Location", format!("/breeds/{}", breed.id)))
        .json(BreedResponse::from(breed)))
}

/// Parámetros de paginación
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    limit: Option<i64>,
    offset: Option<i64>,
}

/// Lista todas las razas con paginación básica
///
/// # Parámetros (opcionales vía query string)
/// - `limit`: Límite de resultados (default: 50)
/// - `offset`: Desplazamiento (default: 0)
///
/// # Ejemplo
/// GET /breeds?limit=10&offset=20
#[actix_web::get("")]
async fn list_breeds(
    query: web::Query<PaginationParams>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Listando razas con parámetros: {:?}", query);

    let breeds = sqlx::query_as!(
        Breed,
        r#"
        SELECT id, species as "species!: AnimalSpecies", name
        FROM breeds
        ORDER BY species ASC, name ASC
        LIMIT $1 OFFSET $2
        "#,
        query.limit.unwrap_or(50).min(400),
        query.offset.unwrap_or(0)
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error al listar razas: {}", e);
        ApiError::InternalServerError("Error al obtener las razas".into())
    })?;

    let response: Vec<BreedResponse> = breeds.into_iter().map(BreedResponse::from).collect();
    Ok(HttpResponse::Ok().json(response))
}

/// Obtiene una raza por ID
///
/// # Ejemplo
/// GET /breeds/1
#[actix_web::get("/{id}")]
async fn get_breed(id: web::Path<i32>, pool: web::Data<PgPool>) -> Result<HttpResponse, ApiError> {
    tracing::info!("Obteniendo raza ID: {}", id);

    let breed = sqlx::query_as!(
        Breed,
        r#"
        SELECT id, species as "species!: AnimalSpecies", name
        FROM breeds
        WHERE id = $1
        "#,
        id.clone()
    )
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or(ApiError::NotFound("La raza no existe".into()))?;

    Ok(HttpResponse::Ok().json(BreedResponse::from(breed)))
}

/// Actualiza una raza existente
///
/// # Ejemplo de petición
/// ```json
/// {
///   "name": "Golden Retriever"
/// }
/// ```
#[actix_web::put("/{id}")]
async fn update_breed(
    id: web::Path<i32>,
    updated_breed: web::Json<NewBreed>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Actualizando raza ID: {}", id);

    let updated_breed = updated_breed.into_inner();
    updated_breed.validate()?;

    // Verificar si la raza existe
    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM breeds
            WHERE id = $1
        )
        "#,
    )
    .bind(id.clone())
    .fetch_one(pool.get_ref())
    .await?;

    if !exists {
        tracing::warn!("Intento de actualizar raza inexistente ID: {}", id);
        return Err(ApiError::NotFound("La raza no existe".into()));
    }

    // Verificar si la combinación de especie y nombre ya existe
    let duplicate_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM breeds
            WHERE species = $1 AND name ILIKE $2 AND id != $3
        )
        "#,
    )
    .bind(&updated_breed.species as &AnimalSpecies)
    .bind(updated_breed.name.trim())
    .bind(id.clone())
    .fetch_one(pool.get_ref())
    .await?;

    if duplicate_exists {
        tracing::warn!(
            "Intento de actualizar raza duplicada: {} ({:?})",
            updated_breed.name,
            updated_breed.species
        );
        return Err(ApiError::Conflict("La raza ya existe".into()));
    }

    // Actualizar la raza en la base de datos
    let breed = sqlx::query_as!(
        Breed,
        r#"
        UPDATE breeds
        SET species = $1, name = $2
        WHERE id = $3
        RETURNING id, species as "species!: AnimalSpecies", name
        "#,
        updated_breed.species as AnimalSpecies,
        updated_breed.name.trim(),
        id.clone()
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error al actualizar raza: {}", e);
        ApiError::InternalServerError("Error al actualizar la raza".into())
    })?;

    Ok(HttpResponse::Ok().json(BreedResponse::from(breed)))
}

/// Elimina una raza existente
///
/// # Ejemplo
/// DELETE /breeds/1
#[actix_web::delete("/{id}")]
async fn delete_breed(
    id: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Eliminando raza ID: {}", id);

    // Verificar si la raza existe
    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM breeds
            WHERE id = $1
        )
        "#,
    )
    .bind(id.clone())
    .fetch_one(pool.get_ref())
    .await?;

    if !exists {
        tracing::warn!("Intento de eliminar raza inexistente ID: {}", id);
        return Err(ApiError::NotFound("La raza no existe".into()));
    }

    // Verificar dependencias (por ejemplo, pacientes asociados)
    let has_deps: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM patients
            WHERE breed = $1
        )
        "#,
    )
    .bind(id.clone())
    .fetch_one(pool.get_ref())
    .await?;

    if has_deps {
        return Err(ApiError::Conflict(
            "No se puede eliminar, la raza tiene mascotas registradas".into(),
        ));
    }

    // Eliminar la raza
    let rows_affected = sqlx::query!(
        r#"
        DELETE FROM breeds
        WHERE id = $1
        "#,
        id.clone()
    )
    .execute(pool.get_ref())
    .await?
    .rows_affected();

    if rows_affected == 0 {
        tracing::warn!("Raza ID {} no encontrada después de intentar eliminar", id);
        return Err(ApiError::NotFound("La raza no existe".into()));
    }

    tracing::info!("Raza ID {} eliminada exitosamente", id);
    Ok(HttpResponse::NoContent().finish())
}

// Exporta todas las funciones como un grupo
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/breeds")
            .service(create_breed)
            .service(list_breeds)
            .service(get_breed)
            .service(update_breed)
            .service(delete_breed),
    );
}
