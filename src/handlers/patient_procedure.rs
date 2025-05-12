use crate::errors::ApiError;
use crate::models::patient_procedure::{
    NewPatientProcedure, PatientProcedure, PatientProcedureFilter, PatientProcedureResponse,
    UpdatePatientProcedure,
};

use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use validator::Validate;

/// Crea un nuevo procedimiento
///
/// # Ejemplo de petición
/// ```json
/// {
///   "patient_id": 1,
///   "procedure_id": 2,
///   "veterinarian_id": 3,
///   "date": "2025-05-15",
///   "next_due_date": "2026-05-15",
///   "notes": "Procedimiento de rutina"
/// }
/// ```
#[actix_web::post("")]
async fn create_patient_procedure(
    new_procedure: web::Json<NewPatientProcedure>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Creando nuevo procedimiento");

    // Validar los datos de entrada
    let new_procedure = new_procedure.into_inner();
    new_procedure.validate()?;
    // validate_date_pair(&new_procedure)?;

    // Insertar el procedimiento en la base de datos
    let procedure = sqlx::query_as!(
        PatientProcedure,
        r#"
        INSERT INTO patient_procedures (
            patient_id,
            procedure_id,
            veterinarian_id,
            date,
            next_due_date,
            notes
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING
            id,
            patient_id as "patient_id!: i32",
            procedure_id as "procedure_id!: i32",
            veterinarian_id as "veterinarian_id!: Option<i32>",
            date as "date!: chrono::NaiveDate",
            next_due_date as "next_due_date!: Option<chrono::NaiveDate>",
            notes
        "#,
        new_procedure.patient_id,
        new_procedure.procedure_id,
        new_procedure.veterinarian_id,
        new_procedure.date,
        new_procedure.next_due_date,
        new_procedure.notes.map(|s| s.trim().to_string())
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error al crear procedimiento: {}", e);
        ApiError::InternalServerError("Error al guardar el procedimiento".into())
    })?;

    tracing::info!("Procedimiento creado exitosamente ID: {}", procedure.id);

    // Convertir a respuesta enriquecida
    let response = PatientProcedureResponse::from_procedure(procedure, pool.get_ref()).await?;

    Ok(HttpResponse::Created()
        .append_header(("Location", format!("/patient-procedures/{}", response.id)))
        .json(response))
}

/// Lista procedimientos con filtros avanzados y paginación
///
/// # Parámetros (opcionales vía query string)
/// - `patient_id`: Filtrar por ID del paciente
/// - `procedure_id`: Filtrar por ID del procedimiento
/// - `veterinarian_id`: Filtrar por ID del veterinario
/// - `start_date`: Filtrar por fecha mínima
/// - `end_date`: Filtrar por fecha máxima
/// - `limit`: Máximo de resultados (default: 50)
/// - `offset`: Desplazamiento (default: 0)
///
/// # Ejemplo
/// GET /patient-procedures?patient_id=1&start_date=2023-01-01&limit=10
#[actix_web::get("")]
async fn list_patient_procedures(
    filters: web::Query<PatientProcedureFilter>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Listando procedimientos con filtros: {:?}", &filters);

    let procedures = sqlx::query_as!(
        PatientProcedure,
        r#"
        SELECT
            id,
            patient_id as "patient_id!: i32",
            procedure_id as "procedure_id!: i32",
            veterinarian_id as "veterinarian_id!: Option<i32>",
            date as "date!: chrono::NaiveDate",
            next_due_date as "next_due_date!: Option<chrono::NaiveDate>",
            notes
        FROM patient_procedures
        WHERE
            ($1::int IS NULL OR patient_id = $1) AND
            ($2::int IS NULL OR procedure_id = $2) AND
            ($3::int IS NULL OR veterinarian_id = $3) AND
            ($4::date IS NULL OR date >= $4) AND
            ($5::date IS NULL OR date <= $5)
        ORDER BY date DESC
        LIMIT $6 OFFSET $7
        "#,
        filters.patient_id,
        filters.procedure_id,
        filters.veterinarian_id,
        filters.start_date,
        filters.end_date,
        filters.limit.unwrap_or(50).min(400),
        filters.offset.unwrap_or(0)
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error al listar procedimientos: {}", e);
        ApiError::InternalServerError("Error al obtener procedimientos".into())
    })?;

    // Convertir a respuestas enriquecidas
    let responses = futures::future::try_join_all(procedures.into_iter().map(|procedure| async {
        PatientProcedureResponse::from_procedure(procedure, pool.get_ref()).await
    }))
    .await?;

    Ok(HttpResponse::Ok().json(responses))
}

/// Obtiene un procedimiento por ID
///
/// # Ejemplo
/// GET /patient-procedures/1
#[actix_web::get("/{id}")]
async fn get_patient_procedure(
    id: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Obteniendo procedimiento ID: {}", id);

    let procedure = sqlx::query_as!(
        PatientProcedure,
        r#"
        SELECT
            id,
            patient_id as "patient_id!: i32",
            procedure_id as "procedure_id!: i32",
            veterinarian_id as "veterinarian_id!: Option<i32>",
            date as "date!: chrono::NaiveDate",
            next_due_date as "next_due_date!: Option<chrono::NaiveDate>",
            notes
        FROM patient_procedures
        WHERE id = $1
        "#,
        id.clone()
    )
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or(ApiError::NotFound("El procedimiento no existe".into()))?;

    // Convertir a respuesta enriquecida
    let response = PatientProcedureResponse::from_procedure(procedure, pool.get_ref()).await?;

    Ok(HttpResponse::Ok().json(response))
}

/// Actualiza un procedimiento existente
///
/// # Ejemplo de petición
/// ```json
/// {
///   "date": "2025-06-01",
///   "notes": "Actualización de notas"
/// }
/// ```
#[actix_web::put("/{id}")]
async fn update_patient_procedure(
    id: web::Path<i32>,
    updated_procedure: web::Json<UpdatePatientProcedure>,
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
            FROM patient_procedures
            WHERE id = $1
        )
        "#,
    )
    .bind(id.clone())
    .fetch_one(pool.get_ref())
    .await?;

    if !exists {
        tracing::warn!("Intento de actualizar procedimiento inexistente ID: {}", id);
        return Err(ApiError::NotFound("El procedimiento no existe".into()));
    }

    // Actualizar el procedimiento
    let procedure = sqlx::query_as!(
        PatientProcedure,
        r#"
        UPDATE patient_procedures
        SET
            patient_id = CASE WHEN $1::INT IS NOT NULL THEN $1 ELSE patient_id END,
            procedure_id = CASE WHEN $2::INT IS NOT NULL THEN $2 ELSE procedure_id END,
            veterinarian_id = CASE WHEN $3::INT IS NOT NULL THEN $3 ELSE veterinarian_id END,
            date = CASE WHEN $4::DATE IS NOT NULL THEN $4 ELSE date END,
            next_due_date = CASE WHEN $5::DATE IS NOT NULL THEN $5 ELSE next_due_date END,
            notes = CASE WHEN $6::TEXT IS NOT NULL THEN $6 ELSE notes END
        WHERE id = $7
        RETURNING
            id,
            patient_id as "patient_id!: i32",
            procedure_id as "procedure_id!: i32",
            veterinarian_id as "veterinarian_id!: Option<i32>",
            date as "date!: chrono::NaiveDate",
            next_due_date as "next_due_date!: Option<chrono::NaiveDate>",
            notes
        "#,
        updated_procedure.patient_id,
        updated_procedure.procedure_id,
        updated_procedure.veterinarian_id.flatten(),
        updated_procedure.date,
        updated_procedure.next_due_date.flatten(),
        updated_procedure
            .notes
            .flatten()
            .map(|s| s.trim().to_string()),
        id.clone()
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error al actualizar procedimiento: {}", e);
        ApiError::InternalServerError("Error al actualizar el procedimiento".into())
    })?;

    // Convertir a respuesta enriquecida
    let response = PatientProcedureResponse::from_procedure(procedure, pool.get_ref()).await?;

    Ok(HttpResponse::Ok().json(response))
}

/// Elimina un procedimiento existente
///
/// # Ejemplo
/// DELETE /patient-procedures/1
#[actix_web::delete("/{id}")]
async fn delete_patient_procedure(
    id: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Eliminando procedimiento ID: {}", id);

    // Verificar si el procedimiento existe
    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM patient_procedures
            WHERE id = $1
        )
        "#,
    )
    .bind(id.clone())
    .fetch_one(pool.get_ref())
    .await?;

    if !exists {
        tracing::warn!("Intento de eliminar procedimiento inexistente ID: {}", id);
        return Err(ApiError::NotFound("El procedimiento no existe".into()));
    }

    // Eliminar el procedimiento
    let rows_affected = sqlx::query!(
        r#"
        DELETE FROM patient_procedures
        WHERE id = $1
        "#,
        id.clone()
    )
    .execute(pool.get_ref())
    .await?
    .rows_affected();

    if rows_affected == 0 {
        tracing::warn!(
            "Procedimiento ID {} no encontrado después de intentar eliminar",
            id
        );
        return Err(ApiError::NotFound("El procedimiento no existe".into()));
    }

    tracing::info!("Procedimiento ID {} eliminado exitosamente", id);
    Ok(HttpResponse::NoContent().finish())
}

// Exporta todas las funciones como un grupo
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/patient_procedures")
            .service(create_patient_procedure)
            .service(list_patient_procedures)
            .service(get_patient_procedure)
            .service(update_patient_procedure)
            .service(delete_patient_procedure), // Agrega más servicios aquí...
    );
}
