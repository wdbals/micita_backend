use crate::errors::ApiError;
use crate::models::medical_record::{
    MedicalRecord, MedicalRecordFilter, MedicalRecordRaw, MedicalRecordResponse, NewMedicalRecord,
    UpdateMedicalRecord,
};

use actix_web::{HttpResponse, web};
use bigdecimal::FromPrimitive;
use sqlx::{PgPool, types::BigDecimal};
use validator::Validate;

/// Crea un nuevo registro médico
///
/// # Ejemplo de petición
/// ```json
/// {
///   "patient_id": 1,
///   "veterinarian_id": 3,
///   "diagnosis": "Infección en la oreja",
///   "treatment": "Antibióticos",
///   "notes": "Seguimiento en una semana",
///   "weight_at_visit": 12.5
/// }
/// ```
#[actix_web::post("")]
async fn create_medical_record(
    new_record: web::Json<NewMedicalRecord>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Creando nuevo registro médico");

    // Validar los datos de entrada
    let new_record = new_record.into_inner();
    new_record.validate()?;

    // Insertar el registro médico en la base de datos
    let record: MedicalRecord = sqlx::query_as!(
        MedicalRecordRaw,
        r#"
        INSERT INTO medical_records (
            patient_id,
            veterinarian_id,
            date,
            diagnosis,
            treatment,
            notes,
            weight_at_visit
        )
        VALUES ($1, $2, NOW(), $3, $4, $5, $6)
        RETURNING
            id,
            patient_id as "patient_id!: i32",
            veterinarian_id as "veterinarian_id!: i32",
            date as "date!: chrono::DateTime<chrono::Utc>",
            diagnosis,
            treatment,
            notes,
            weight_at_visit as "weight_at_visit!: BigDecimal"
        "#,
        new_record.patient_id,
        new_record.veterinarian_id,
        new_record.diagnosis.trim(),
        new_record.treatment.map(|s| s.trim().to_string()),
        new_record.notes.map(|s| s.trim().to_string()),
        BigDecimal::from_f64(new_record.weight_at_visit.ok_or_else(|| {
            ApiError::ValidationError("El campo weight_at_visit es obligatorio".into())
        })?)
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error al crear registro médico: {}", e);
        ApiError::InternalServerError("Error al guardar el registro médico".into())
    })?
    .into();

    // Obtener el nombre del veterinario
    let vet_name: String = sqlx::query_scalar!(
        r#"
        SELECT name
        FROM users
        WHERE id = $1
        "#,
        record.veterinarian_id
    )
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or_else(|_| "Veterinario desconocido".to_string());

    tracing::info!("Registro médico creado exitosamente ID: {}", record.id);

    Ok(HttpResponse::Created()
        .append_header(("Location", format!("/medical_records/{}", record.id)))
        .json(MedicalRecordResponse::from_record_with_vet(
            record, vet_name,
        )))
}

/// Lista registros médicos con filtros avanzados y paginación
///
/// # Parámetros (opcionales vía query string)
/// - `patient_id`: Filtrar por mascota
/// - `veterinarian_id`: Filtrar por veterinario
/// - `start_date`: Registros después de esta fecha
/// - `end_date`: Registros antes de esta fecha
/// - `diagnosis_contains`: Búsqueda parcial en diagnóstico
/// - `limit`: Máximo de resultados (default: 50)
/// - `offset`: Desplazamiento (default: 0)
///
/// # Ejemplo
/// GET /medical_records?patient_id=1&start_date=2023-01-01T00:00:00Z&limit=10
#[actix_web::get("")]
async fn list_medical_records(
    filters: web::Query<MedicalRecordFilter>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Listando registros médicos con filtros: {:?}", &filters);

    let records = sqlx::query_as!(
        MedicalRecordRaw,
        r#"
        SELECT
            id,
            patient_id as "patient_id!: i32",
            veterinarian_id as "veterinarian_id!: i32",
            date as "date!: chrono::DateTime<chrono::Utc>",
            diagnosis,
            treatment,
            notes,
            weight_at_visit as "weight_at_visit!: BigDecimal"
        FROM medical_records
        WHERE
            ($1::int IS NULL OR patient_id = $1) AND
            ($2::int IS NULL OR veterinarian_id = $2) AND
            ($3::timestamptz IS NULL OR date >= $3) AND
            ($4::timestamptz IS NULL OR date <= $4) AND
            ($5::text IS NULL OR diagnosis ILIKE '%' || $5 || '%')
        ORDER BY date DESC
        LIMIT $6 OFFSET $7
        "#,
        filters.patient_id,
        filters.veterinarian_id,
        filters.start_date,
        filters.end_date,
        filters.diagnosis_contains.as_deref(),
        filters.limit.unwrap_or(50).min(400),
        filters.offset.unwrap_or(0)
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error al listar registros médicos: {}", e);
        ApiError::InternalServerError("Error al obtener registros médicos".into())
    })?;

    // Convertir a respuestas enriquecidas
    let mut responses = Vec::new();
    for record_raw in records {
        let medical_record: MedicalRecord = record_raw.into(); // Usa From aquí

        // Obtener el nombre del veterinario
        let vet_name: String = sqlx::query_scalar!(
            r#"
            SELECT name
            FROM users
            WHERE id = $1
            "#,
            medical_record.veterinarian_id
        )
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or_else(|_| "Unknown Veterinarian".to_string());

        responses.push(MedicalRecordResponse::from_record_with_vet(
            medical_record,
            vet_name,
        ));
    }

    Ok(HttpResponse::Ok().json(responses))
}

/// Obtiene un registro médico por ID
///
/// # Ejemplo
/// GET /medical_records/1
#[actix_web::get("/{id}")]
async fn get_medical_record(
    id: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Obteniendo registro médico ID: {}", id);

    let record: MedicalRecord = sqlx::query_as!(
        MedicalRecordRaw,
        r#"
        SELECT
            id,
            patient_id as "patient_id!: i32",
            veterinarian_id as "veterinarian_id!: i32",
            date as "date!: chrono::DateTime<chrono::Utc>",
            diagnosis,
            treatment,
            notes,
            weight_at_visit as "weight_at_visit!: BigDecimal"
        FROM medical_records
        WHERE id = $1
        "#,
        id.clone()
    )
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or(ApiError::NotFound("El registro médico no existe".into()))?
    .into();

    // Obtener el nombre del veterinario
    let vet_name: String = sqlx::query_scalar!(
        r#"
        SELECT name
        FROM users
        WHERE id = $1
        "#,
        record.veterinarian_id
    )
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or_else(|_| "Unknown Veterinarian".to_string());

    Ok(
        HttpResponse::Ok().json(MedicalRecordResponse::from_record_with_vet(
            record, vet_name,
        )),
    )
}

/// Actualiza un registro médico existente
///
/// # Ejemplo de petición
/// ```json
/// {
///   "diagnosis": "Infección en la oreja (actualizado)",
///   "treatment": null
/// }
/// ```
#[actix_web::put("/{id}")]
async fn update_medical_record(
    id: web::Path<i32>,
    updated_record: web::Json<UpdateMedicalRecord>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Actualizando registro médico ID: {}", id);

    let updated_record = updated_record.into_inner();
    updated_record.validate()?;

    // Verificar si el registro existe
    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM medical_records
            WHERE id = $1
        )
        "#,
    )
    .bind(id.clone())
    .fetch_one(pool.get_ref())
    .await?;

    if !exists {
        tracing::warn!(
            "Intento de actualizar registro médico inexistente ID: {}",
            id
        );
        return Err(ApiError::NotFound("El registro médico no existe".into()));
    }

    let weigth_at_visit = match updated_record.weight_at_visit {
        None => None,       // No se proporciona ningún cambio
        Some(None) => None, // Se desea eliminar el valor (NULL)
        Some(Some(value)) => BigDecimal::from_f64(value),
    };

    // Actualizar el registro médico
    let record: MedicalRecord = sqlx::query_as!(
        MedicalRecordRaw,
        r#"
        UPDATE medical_records
        SET
            patient_id = COALESCE($1, patient_id),
            veterinarian_id = COALESCE($2, veterinarian_id),
            diagnosis = CASE WHEN $3::TEXT IS NOT NULL THEN $3 ELSE diagnosis END,
            treatment = CASE WHEN $4::TEXT IS NOT NULL THEN $4 ELSE treatment END,
            notes = CASE WHEN $5::TEXT IS NOT NULL THEN $5 ELSE notes END,
            weight_at_visit = CASE
                    WHEN $6::NUMERIC IS NOT NULL THEN $6 -- Nuevo valor
                    WHEN $6 IS NULL AND $7::BOOLEAN THEN NULL -- Borrar el valor
                    ELSE weight_at_visit -- Mantener el valor existente
                    END
        WHERE id = $8
        RETURNING
            id,
            patient_id as "patient_id!: i32",
            veterinarian_id as "veterinarian_id!: i32",
            date as "date!: chrono::DateTime<chrono::Utc>",
            diagnosis,
            treatment,
            notes,
            weight_at_visit as "weight_at_visit!: BigDecimal"
        "#,
        updated_record.patient_id,
        updated_record.veterinarian_id,
        updated_record.diagnosis.map(|s| s.trim().to_string()),
        updated_record
            .treatment
            .flatten()
            .map(|s| s.trim().to_string()),
        updated_record.notes.flatten().map(|s| s.trim().to_string()),
        weigth_at_visit,
        updated_record.weight_at_visit.is_some() && updated_record.weight_at_visit == Some(None),
        id.clone()
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error al actualizar registro médico: {}", e);
        ApiError::InternalServerError("Error al actualizar el registro médico".into())
    })?
    .into();

    // Obtener el nombre del veterinario
    let vet_name: String = sqlx::query_scalar!(
        r#"
        SELECT name
        FROM users
        WHERE id = $1
        "#,
        record.veterinarian_id
    )
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or_else(|_| "Unknown Veterinarian".to_string());

    Ok(
        HttpResponse::Ok().json(MedicalRecordResponse::from_record_with_vet(
            record, vet_name,
        )),
    )
}

/// Elimina un registro médico existente
///
/// # Ejemplo
/// DELETE /medical_records/1
#[actix_web::delete("/{id}")]
async fn delete_medical_record(
    id: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Eliminando registro médico ID: {}", id);

    // Verificar si el registro existe
    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM medical_records
            WHERE id = $1
        )
        "#,
    )
    .bind(id.clone())
    .fetch_one(pool.get_ref())
    .await?;

    if !exists {
        tracing::warn!("Intento de eliminar registro médico inexistente ID: {}", id);
        return Err(ApiError::NotFound("El registro médico no existe".into()));
    }

    // Eliminar el registro
    let rows_affected = sqlx::query!(
        r#"
        DELETE FROM medical_records
        WHERE id = $1
        "#,
        id.clone()
    )
    .execute(pool.get_ref())
    .await?
    .rows_affected();

    if rows_affected == 0 {
        tracing::warn!(
            "Registro médico ID {} no encontrado después de intentar eliminar",
            id
        );
        return Err(ApiError::NotFound("El registro médico no existe".into()));
    }

    tracing::info!("Registro médico ID {} eliminado exitosamente", id);
    Ok(HttpResponse::NoContent().finish())
}

// Exporta todas las funciones como un grupo
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/medical_records")
            .service(create_medical_record)
            .service(list_medical_records)
            .service(get_medical_record)
            .service(update_medical_record)
            .service(delete_medical_record), // Agrega más servicios aquí...
    );
}
