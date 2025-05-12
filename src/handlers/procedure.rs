use crate::errors::ApiError;
use crate::models::enums::ProcedureType;
use crate::models::procedure::{
    NewProcedure, Procedure, ProcedureFilter, ProcedureResponse, UpdateProcedure,
};

use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use validator::Validate;

/// Crea un nuevo procedimiento
///
/// # Ejemplo de petición
/// ```json
/// {
///   "name": "Cirugía dental",
///   "procedure_type": "SURGICAL",
///   "description": "Procedimiento quirúrgico para extracción de muelas",
///   "duration_minutes": 90
/// }
/// ```
#[actix_web::post("")]
async fn create_procedure(
    new_procedure: web::Json<NewProcedure>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Creando nuevo procedimiento");

    // Validar los datos de entrada
    let new_procedure = new_procedure.into_inner();
    new_procedure.validate()?;

    // Insertar el procedimiento en la base de datos
    let procedure = sqlx::query_as!(
        Procedure,
        r#"
        INSERT INTO procedures (
            name,
            type,
            description,
            duration_minutes
        )
        VALUES ($1, $2, $3, $4)
        RETURNING
            id,
            name,
            type as "procedure_type!: ProcedureType",
            description,
            duration_minutes
        "#,
        new_procedure.name.trim(),
        new_procedure.procedure_type as ProcedureType,
        new_procedure.description.map(|s| s.trim().to_string()),
        new_procedure.duration_minutes
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error al crear procedimiento: {}", e);
        ApiError::InternalServerError("Error al guardar el procedimiento".into())
    })?;

    // Convertir a respuesta enriquecida
    let response = ProcedureResponse::from(procedure);

    tracing::info!("Procedimiento creado exitosamente ID: {}", response.id);

    Ok(HttpResponse::Created()
        .append_header(("Location", format!("/procedures/{}", response.id)))
        .json(response))
}

/// Lista procedimientos con filtros avanzados y paginación
///
/// # Parámetros (opcionales vía query string)
/// - `name_contains`: Filtrar por nombre que contenga cierto texto
/// - `procedure_type`: Filtrar por tipo de procedimiento
/// - `min_duration`: Duración mínima en minutos
/// - `max_duration`: Duración máxima en minutos
/// - `limit`: Máximo de resultados (default: 50)
/// - `offset`: Desplazamiento (default: 0)
///
/// # Ejemplo
/// GET /procedures?name_contains=dental&limit=10
#[actix_web::get("")]
async fn list_procedures(
    filters: web::Query<ProcedureFilter>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Listando procedimientos con filtros: {:?}", &filters);

    // Obtener los procedimientos base desde la base de datos
    let procedures = sqlx::query_as!(
        Procedure,
        r#"
        SELECT
            id,
            name,
            type as "procedure_type!: ProcedureType",
            description,
            duration_minutes
        FROM procedures
        WHERE
            ($1::TEXT IS NULL OR name ILIKE '%' || $1 || '%') AND
            ($2::procedure_type IS NULL OR type = $2) AND
            ($3::INT IS NULL OR duration_minutes >= $3) AND
            ($4::INT IS NULL OR duration_minutes <= $4)
        ORDER BY name ASC
        LIMIT $5 OFFSET $6
        "#,
        filters.name_contains,
        &filters.procedure_type as &Option<ProcedureType>,
        filters.min_duration,
        filters.max_duration,
        filters.limit.unwrap_or(50).min(400),
        filters.offset.unwrap_or(0)
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error al listar procedimientos: {}", e);
        ApiError::InternalServerError("Error al obtener procedimientos".into())
    })?;

    // Convertir cada procedimiento a una respuesta enriquecida
    let responses: Vec<ProcedureResponse> = procedures
        .into_iter()
        .map(ProcedureResponse::from)
        .collect();

    Ok(HttpResponse::Ok().json(responses))
}

/// Obtiene un procedimiento por ID
///
/// # Ejemplo
/// GET /procedures/1
#[actix_web::get("/{id}")]
async fn get_procedure(
    id: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Obteniendo procedimiento ID: {}", id);

    // Obtener el procedimiento base
    let procedure = sqlx::query_as!(
        Procedure,
        r#"
        SELECT
            id,
            name,
            type as "procedure_type!: ProcedureType",
            description,
            duration_minutes
        FROM procedures
        WHERE id = $1
        "#,
        id.clone()
    )
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or(ApiError::NotFound("El procedimiento no existe".into()))?;

    // Convertir a respuesta enriquecida
    let response = ProcedureResponse::from(procedure);

    Ok(HttpResponse::Ok().json(response))
}

/// Actualiza un procedimiento existente
///
/// # Ejemplo de petición
/// ```json
/// {
///   "name": "Cirugía dental actualizada",
///   "description": null
/// }
/// ```
#[actix_web::put("/{id}")]
async fn update_procedure(
    id: web::Path<i32>,
    updated_procedure: web::Json<UpdateProcedure>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Actualizando procedimiento ID: {}", id);

    let updated_procedure = updated_procedure.into_inner();
    updated_procedure.validate()?;

    // Verificar si el procedimiento existe
    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM procedures
            WHERE id = $1
        )
        "#,
    )
    .bind(id.clone())
    .fetch_one(pool.get_ref())
    .await?;

    if !exists {
        return Err(ApiError::NotFound("El procedimiento no existe".into()));
    }

    let is_description: bool =
        updated_procedure.description.is_some() && updated_procedure.description == Some(None);
    let is_duration: bool = updated_procedure.duration_minutes.is_some()
        && updated_procedure.duration_minutes == Some(None);

    // Actualizar el procedimiento
    let procedure = sqlx::query_as!(
        Procedure,
        r#"
        UPDATE procedures
        SET
            name = CASE WHEN $1::TEXT IS NOT NULL THEN $1 ELSE name END,
            type = CASE WHEN $2::procedure_type IS NOT NULL THEN $2 ELSE type END,
            description = CASE
                WHEN $3::TEXT IS NOT NULL THEN $3 -- Nuevo valor
                WHEN $4::BOOLEAN THEN NULL -- Borrar el valor
                ELSE description -- Mantener el valor existente
            END,
            duration_minutes = CASE
                WHEN $5::INT IS NOT NULL THEN $5 -- Nuevo valor
                WHEN $6::BOOLEAN THEN NULL -- Borrar el valor
                ELSE duration_minutes -- Mantener el valor existente
            END
        WHERE id = $7
        RETURNING
            id,
            name,
            type as "procedure_type!: ProcedureType",
            description,
            duration_minutes
        "#,
        updated_procedure.name,
        updated_procedure.procedure_type as Option<ProcedureType>,
        updated_procedure.description.flatten(),
        is_description,
        updated_procedure.duration_minutes.flatten(),
        is_duration,
        id.clone()
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error al actualizar procedimiento: {}", e);
        ApiError::InternalServerError("Error al actualizar el procedimiento".into())
    })?;

    // Convertir a respuesta enriquecida
    let response = ProcedureResponse::from(procedure);

    Ok(HttpResponse::Ok().json(response))
}

/// Elimina un procedimiento existente
///
/// # Ejemplo
/// DELETE /procedures/1
#[actix_web::delete("/{id}")]
async fn delete_procedure(
    id: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Eliminando procedimiento ID: {}", id);

    // Verificar si el procedimiento existe
    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM procedures
            WHERE id = $1
        )
        "#,
    )
    .bind(id.clone())
    .fetch_one(pool.get_ref())
    .await?;

    if !exists {
        return Err(ApiError::NotFound("El procedimiento no existe".into()));
    }

    // Eliminar el procedimiento
    let rows_affected = sqlx::query(
        r#"
        DELETE FROM procedures
        WHERE id = $1
        "#,
    )
    .bind(id.clone())
    .execute(pool.get_ref())
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(ApiError::NotFound("El procedimiento no existe".into()));
    }

    tracing::info!("Procedimiento ID {} eliminado exitosamente", id);
    Ok(HttpResponse::NoContent().finish())
}

// Exporta todas las funciones como un grupo
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/procedures")
            .service(create_procedure)
            .service(list_procedures)
            .service(get_procedure)
            .service(update_procedure)
            .service(delete_procedure), // Agrega más servicios aquí...
    );
}
