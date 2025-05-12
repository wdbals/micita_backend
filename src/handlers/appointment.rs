use crate::errors::ApiError;
use crate::models::appointment::{
    Appointment, AppointmentFilter, AppointmentResponse, NewAppointment, UpdateAppointment,
};
use crate::models::enums::AppointmentStatus;
use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use validator::Validate;

/// Lista citas con filtros avanzados y paginación
///
/// # Parámetros (opcionales vía query string)
/// - `patient_id`: Filtrar por mascota
/// - `cliente_id`: Filtrar por dueño de la mascota
/// - `veterinarian_id`: Filtrar por veterinario
/// - `status`: Filtrar por estado (scheduled, completed, etc.)
/// - `start_date`: Citas después de esta fecha
/// - `end_date`: Citas antes de esta fecha
/// - reason_contains: Filtra por razón
/// - `limit`: Máximo de resultados (default: 50)
/// - `offset`: Desplazamiento (default: 0)
///
/// # Ejemplo
/// GET /appointments?patient_id=5&status=scheduled&limit=10
#[actix_web::get("")]
async fn list_appointments(
    filters: web::Query<AppointmentFilter>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Listando citas con filtros: {:?}", &filters);

    let appointments = sqlx::query_as!(
        Appointment,
        r#"
        SELECT
            id,
            patient_id,
            client_id,
            veterinarian_id,
            start_time as "start_time!: chrono::DateTime<chrono::Utc>",
            end_time as "end_time!: chrono::DateTime<chrono::Utc>",
            status as "status!: AppointmentStatus",
            reason
        FROM appointments
        WHERE
            ($1::int IS NULL OR patient_id = $1) AND
            ($2::int IS NULL OR client_id = $2) AND
            ($3::int IS NULL OR veterinarian_id = $3) AND
            ($4::appointment_status IS NULL OR status = $4) AND
            ($5::timestamptz IS NULL OR start_time >= $5) AND
            ($6::timestamptz IS NULL OR end_time <= $6) AND
            ($7::text IS NULL OR reason ILIKE '%' || $7 || '%')
        ORDER BY start_time DESC
        LIMIT $8 OFFSET $9
        "#,
        filters.patient_id,
        filters.client_id,
        filters.veterinarian_id,
        filters.status.clone() as Option<AppointmentStatus>,
        filters.start_date,
        filters.end_date,
        filters.reason_contains,
        filters.limit.unwrap_or(50).min(400),
        filters.offset.unwrap_or(0)
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error al listar citas: {}", e);
        ApiError::InternalServerError("Error al obtener citas".into())
    })?;

    // Convertir a respuestas enriquecidas
    let responses = futures::future::try_join_all(
        appointments
            .into_iter()
            .map(|app| async { AppointmentResponse::from_appointment(app, pool.get_ref()).await }),
    )
    .await?;

    Ok(HttpResponse::Ok().json(responses))
}

/// Obtiene una cita específica por su ID
///
/// # Respuestas
/// - 200 OK: Devuelve la cita en formato JSON
/// - 404 Not Found: Si la cita no existe
/// - 500 Internal Server Error: Error de base de datos
#[actix_web::get("/{id}")]
async fn get_appointment(
    id: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Obteniendo cita con ID: {}", id);

    // Obtener la cita básica
    let appointment = sqlx::query_as!(
        Appointment,
        r#"
        SELECT
            id,
            patient_id,
            client_id,
            veterinarian_id,
            start_time as "start_time!: chrono::DateTime<chrono::Utc>",
            end_time as "end_time!: chrono::DateTime<chrono::Utc>",
            status as "status!: AppointmentStatus",
            reason
        FROM appointments
        WHERE id = $1
        "#,
        id.clone()
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error al buscar cita: {}", e);
        ApiError::InternalServerError("Error al acceder a la base de datos".into())
    })?;

    match appointment {
        Some(appt) => {
            // Convertir a respuesta enriquecida
            let response = AppointmentResponse::from_appointment(appt, pool.get_ref())
                .await
                .map_err(|e| {
                    tracing::error!("Error al enriquecer datos de cita: {}", e);
                    ApiError::InternalServerError("Error al procesar la cita".into())
                })?;

            tracing::info!("Cita {} encontrada", id);
            Ok(HttpResponse::Ok().json(response))
        }
        None => {
            tracing::warn!("Cita {} no encontrada", id);
            Err(ApiError::NotFound("Cita no encontrada".into()))
        }
    }
}

/// Crea una nueva cita
///
/// # Ejemplo de petición
/// ```json
/// {
///   "patient_id": 1,
///   "client_id": 2,
///   "veterinarian_id": 3,
///   "start_time": "2023-11-01T10:00:00Z",
///   "end_time": "2023-11-01T11:00:00Z",
///   "reason": "Consulta de rutina"
/// }
/// ```
#[actix_web::post("")]
async fn create_appointment(
    new_appointment: web::Json<NewAppointment>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Creando nueva cita");

    // Validar los datos de entrada
    let new_appointment = new_appointment.into_inner();
    new_appointment.validate()?;

    // Verificar que el veterinario esté disponible en el rango de tiempo
    let veterinarian_is_available: bool = sqlx::query_scalar!(
        r#"
        SELECT NOT EXISTS (
            SELECT 1
            FROM appointments
            WHERE veterinarian_id = $1
              AND (
                  ($2, $3) OVERLAPS (start_time, end_time)
              )
        )
        "#,
        new_appointment.veterinarian_id,
        new_appointment.start_time,
        new_appointment.end_time,
    )
    .fetch_one(pool.get_ref())
    .await?
    .unwrap_or(true);

    if !veterinarian_is_available {
        tracing::warn!(
            "El veterinario con ID {} no está disponible en el rango de tiempo solicitado",
            new_appointment.veterinarian_id
        );
        return Err(ApiError::Conflict(
            "El veterinario no está disponible en este horario".into(),
        ));
    }

    // Insertar la cita en la base de datos
    let appointment = sqlx::query_as!(
        Appointment,
        r#"
        INSERT INTO appointments (
            patient_id,
            client_id,
            veterinarian_id,
            start_time,
            end_time,
            status,
            reason
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING
            id,
            patient_id,
            client_id,
            veterinarian_id,
            start_time as "start_time!: chrono::DateTime<chrono::Utc>",
            end_time as "end_time!: chrono::DateTime<chrono::Utc>",
            status as "status!: AppointmentStatus",
            reason
        "#,
        new_appointment.patient_id,
        new_appointment.client_id,
        new_appointment.veterinarian_id,
        new_appointment.start_time,
        new_appointment.end_time,
        AppointmentStatus::Scheduled as AppointmentStatus, // Estado inicial
        new_appointment.reason
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error al crear cita: {}", e);
        ApiError::InternalServerError("Error al guardar la cita".into())
    })?;

    tracing::info!("Cita creada exitosamente ID: {}", appointment.id);

    // Convertir a respuesta enriquecida
    let response = AppointmentResponse::from_appointment(appointment, pool.get_ref()).await?;

    Ok(HttpResponse::Created()
        .append_header(("Location", format!("/appointments/{}", response.id)))
        .json(response))
}

/// Actualiza una cita existente (actualización parcial)
///
/// # Ejemplo de petición
/// ```json
/// {
///   "start_time": "2023-11-01T14:00:00Z",
///   "end_time": "2023-11-01T15:00:00Z",
///   "reason": "Consulta de seguimiento"
/// }
/// ```
#[actix_web::put("/{id}")]
async fn update_appointment(
    id: web::Path<i32>,
    update_data: web::Json<UpdateAppointment>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Actualizando cita ID: {}", id);

    let update_data = update_data.into_inner();
    update_data.validate()?;

    // Manejo especial para Option<Option> fields
    let patient_id = match update_data.patient_id {
        Some(inner) => inner, // Some(patient_id) o None (para desasociar)
        None => None,         // Mantener valor existente
    };

    let client_id = match update_data.client_id {
        Some(inner) => inner, // Some(client_id) o None (para desasociar)
        None => None,         // Mantener valor existente
    };

    let veterinarian_id = update_data.veterinarian_id;

    // Verificar disponibilidad si se cambia el veterinario o el rango de tiempo
    if veterinarian_id.is_some()
        || update_data.start_time.is_some()
        || update_data.end_time.is_some()
    {
        let existing_appointment = sqlx::query_as!(
            Appointment,
            r#"
            SELECT
                id,
                patient_id,
                client_id,
                veterinarian_id,
                start_time as "start_time!: chrono::DateTime<chrono::Utc>",
                end_time as "end_time!: chrono::DateTime<chrono::Utc>",
                status as "status!: AppointmentStatus",
                reason
            FROM appointments
            WHERE id = $1
            "#,
            id.clone()
        )
        .fetch_optional(pool.get_ref())
        .await?
        .ok_or(ApiError::NotFound("La cita no existe".into()))?;

        let new_veterinarian_id = veterinarian_id.unwrap_or(existing_appointment.veterinarian_id);
        let new_start_time = update_data
            .start_time
            .unwrap_or(existing_appointment.start_time);
        let new_end_time = update_data
            .end_time
            .unwrap_or(existing_appointment.end_time);

        let veterinarian_is_available: bool = sqlx::query_scalar!(
            r#"
            SELECT NOT EXISTS (
                SELECT 1
                FROM appointments
                WHERE veterinarian_id = $1
                  AND id != $2 -- Excluir la cita actual
                  AND (
                      ($3, $4) OVERLAPS (start_time, end_time)
                  )
            )
            "#,
            new_veterinarian_id,
            id.clone(),
            new_start_time,
            new_end_time
        )
        .fetch_one(pool.get_ref())
        .await?
        .unwrap_or(true);

        if !veterinarian_is_available {
            tracing::warn!(
                "El veterinario con ID {} no está disponible en el nuevo rango de tiempo",
                new_veterinarian_id
            );
            return Err(ApiError::Conflict(
                "El veterinario no está disponible en este horario".into(),
            ));
        }
    }

    // Actualizar la cita en la base de datos
    let appointment = sqlx::query_as!(
        Appointment,
        r#"
        UPDATE appointments SET
            patient_id = COALESCE($1, patient_id),
            client_id = COALESCE($2, client_id),
            veterinarian_id = CASE WHEN $3::INTEGER IS NOT NULL THEN $3 ELSE veterinarian_id END,
            start_time = CASE WHEN $4::TIMESTAMPTZ IS NOT NULL THEN $4 ELSE start_time END,
            end_time = CASE WHEN $5::TIMESTAMPTZ IS NOT NULL THEN $5 ELSE end_time END,
            status = CASE WHEN $6::appointment_status IS NOT NULL THEN $6 ELSE status END,
            reason = CASE WHEN $7::TEXT IS NOT NULL THEN $7 ELSE reason END
        WHERE id = $8
        RETURNING
            id,
            patient_id,
            client_id,
            veterinarian_id,
            start_time as "start_time!: chrono::DateTime<chrono::Utc>",
            end_time as "end_time!: chrono::DateTime<chrono::Utc>",
            status as "status!: AppointmentStatus",
            reason
        "#,
        patient_id,
        client_id,
        veterinarian_id,
        update_data.start_time,
        update_data.end_time,
        update_data.status as Option<AppointmentStatus>,
        update_data.reason,
        id.clone()
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error al actualizar cita: {}", e);
        ApiError::InternalServerError("Error al actualizar la cita".into())
    })?;

    match appointment {
        Some(appointment) => {
            tracing::info!("Cita {} actualizada exitosamente", appointment.id);
            Ok(HttpResponse::Ok()
                .json(AppointmentResponse::from_appointment(appointment, pool.get_ref()).await?))
        }
        None => {
            tracing::warn!("Cita {} no encontrada", &id);
            Err(ApiError::NotFound("La cita no existe".into()))
        }
    }
}

/// Elimina una cita existente
#[actix_web::delete("/{id}")]
async fn delete_appointment(
    id: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Eliminando cita ID: {}", id);

    // Verificar si la cita existe
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS( SELECT 1 FROM appointments WHERE id = $1 )")
            .bind(id.clone())
            .fetch_one(pool.get_ref())
            .await?;

    if !exists {
        tracing::warn!("Intento de eliminar cita inexistente ID: {}", id);
        return Err(ApiError::NotFound("La cita no existe".into()));
    }

    // Verificar restricciones de estado (opcional)
    let status: Option<String> = sqlx::query_scalar(
        r#"
        SELECT status
        FROM appointments
        WHERE id = $1
        "#,
    )
    .bind(id.clone())
    .fetch_optional(pool.get_ref())
    .await?
    .flatten();

    if let Some(status) = status {
        if status == "completed" || status == "canceled" {
            return Err(ApiError::Conflict(format!(
                "No se puede eliminar una cita con estado '{}'",
                status
            )));
        }
    }

    // Eliminar la cita
    let rows_affected = sqlx::query!(
        r#"
        DELETE FROM appointments
        WHERE id = $1
        "#,
        id.clone()
    )
    .execute(pool.get_ref())
    .await?
    .rows_affected();

    if rows_affected == 0 {
        tracing::warn!("Cita ID {} no encontrada después de intentar eliminar", id);
        return Err(ApiError::NotFound("La cita no existe".into()));
    }

    tracing::info!("Cita ID {} eliminada exitosamente", id);
    Ok(HttpResponse::NoContent().finish())
}

// Exporta todas las funciones como un grupo
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/appointments")
            .service(list_appointments)
            .service(get_appointment)
            .service(create_appointment)
            .service(update_appointment)
            .service(delete_appointment), // Agrega más servicios aquí...
    );
}
