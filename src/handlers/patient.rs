use crate::errors::ApiError;
use crate::models::enums::{AnimalGender, AnimalSpecies};
use crate::models::patient::{
    NewPatient, Patient, PatientFilter, PatientRaw, PatientResponse, UpdatePatient,
};

use actix_web::{HttpResponse, web};
use bigdecimal::{BigDecimal, FromPrimitive};
use sqlx::PgPool;
use validator::Validate;

/// Crea un nuevo paciente
///
/// # Ejemplo de petición
/// ```json
/// {
///   "name": "Max",
///   "species": "Dog",
///   "breed_id": 3,
///   "birth_date": "2020-05-15",
///   "gender": "Male",
///   "weight_kg": 12.5,
///   "client_id": 1,
///   "photo_url": "https://example.com/max.jpg "
/// }
/// ```
#[actix_web::post("")]
async fn create_patient(
    new_patient: web::Json<NewPatient>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Creando nuevo paciente");

    // Validar los datos de entrada
    let new_patient = new_patient.into_inner();
    new_patient.validate()?;

    // Insertar el paciente en la base de datos
    let patient: Patient = sqlx::query_as!(
        PatientRaw,
        r#"
        INSERT INTO patients (
            name,
            species,
            breed,
            birth_date,
            gender,
            weight_kg,
            client_id,
            photo_url
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING
            id,
            name,
            species as "species!: AnimalSpecies",
            breed as "breed_id!: Option<i32>",
            birth_date,
            gender as "gender!: Option<AnimalGender>",
            weight_kg as "weight_kg!: BigDecimal",
            client_id as "client_id!: i32",
            photo_url
        "#,
        new_patient.name.trim(),
        new_patient.species as AnimalSpecies,
        new_patient.breed_id,
        new_patient.birth_date,
        new_patient.gender as Option<AnimalGender>,
        BigDecimal::from_f64(new_patient.weight_kg.ok_or_else(|| {
            ApiError::ValidationError("El campo weight_at_visit es obligatorio".into())
        })?),
        new_patient.client_id,
        new_patient.photo_url.map(|s| s.trim().to_string())
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error al crear paciente: {}", e);
        ApiError::InternalServerError("Error al guardar el paciente".into())
    })?
    .into();

    // Obtener el nombre de la raza si existe
    let breed_name: Option<String> = if let Some(breed_id) = patient.breed_id {
        sqlx::query_scalar!(
            r#"
            SELECT name
            FROM breeds
            WHERE id = $1
            "#,
            breed_id
        )
        .fetch_optional(pool.get_ref())
        .await?
    } else {
        None
    };

    // Construir la respuesta
    let mut response: PatientResponse = patient.into();
    response.breed = breed_name;

    tracing::info!("Paciente creado exitosamente ID: {}", response.id);

    Ok(HttpResponse::Created()
        .append_header(("Location", format!("/patients/{}", response.id)))
        .json(response))
}

/// Lista pacientes con filtros avanzados y paginación
///
/// # Parámetros (opcionales vía query string)
/// - `name`: Filtrar por nombre (búsqueda parcial insensible a mayúsculas/minúsculas)
/// - `species`: Filtrar por especie (DOG, CAT, etc.)
/// - `breed_id`: Filtrar por ID de raza
/// - `client_id`: Filtrar por ID del cliente
/// - `gender`: Filtrar por género (MALE, FEMALE, etc.)
/// - `limit`: Máximo de resultados (default: 50)
/// - `offset`: Desplazamiento (default: 0)
#[actix_web::get("")]
async fn list_patients(
    filters: web::Query<PatientFilter>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Listando pacientes con filtros: {:?}", &filters);

    let patients = sqlx::query_as!(
        PatientRaw,
        r#"
        SELECT
            id,
            name,
            species as "species!: AnimalSpecies",
            breed as "breed_id!: Option<i32>",
            birth_date,
            gender as "gender!: Option<AnimalGender>",
            weight_kg as "weight_kg!: BigDecimal",
            client_id as "client_id!: i32",
            photo_url
        FROM patients
        WHERE
            ($1::text IS NULL OR name ILIKE '%' || $1 || '%') AND
            ($2::animal_species IS NULL OR species = $2) AND
            ($3::int IS NULL OR breed = $3) AND
            ($4::int IS NULL OR client_id = $4) AND
            ($5::animal_gender IS NULL OR gender = $5)
        ORDER BY name ASC
        LIMIT $6 OFFSET $7
        "#,
        filters.name.as_deref(),
        &filters.species as &Option<AnimalSpecies>,
        filters.breed_id,
        filters.client_id,
        &filters.gender as &Option<AnimalGender>,
        filters.limit.unwrap_or(50),
        filters.offset.unwrap_or(0)
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error al listar pacientes: {}", e);
        ApiError::InternalServerError("Error al obtener pacientes".into())
    })?;

    // Convertir a respuestas enriquecidas
    let mut responses = Vec::new();
    for patient in patients {
        let patient: Patient = patient.into();

        let breed_name: Option<String> = if let Some(breed_id) = patient.breed_id {
            sqlx::query_scalar!(
                r#"
                SELECT name
                FROM breeds
                WHERE id = $1
                "#,
                breed_id
            )
            .fetch_optional(pool.get_ref())
            .await?
        } else {
            None
        };

        let mut response: PatientResponse = patient.into();
        response.breed = breed_name;
        responses.push(response);
    }

    Ok(HttpResponse::Ok().json(responses))
}

/// Obtiene un paciente por ID
///
/// # Ejemplo
/// GET /patients/1
#[actix_web::get("/{id}")]
async fn get_patient(
    id: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Obteniendo paciente ID: {}", id);

    let patient: Patient = sqlx::query_as!(
        PatientRaw,
        r#"
        SELECT
            id,
            name,
            species as "species!: AnimalSpecies",
            breed as "breed_id!: Option<i32>",
            birth_date,
            gender as "gender!: Option<AnimalGender>",
            weight_kg as "weight_kg!: BigDecimal",
            client_id as "client_id!: i32",
            photo_url
        FROM patients
        WHERE id = $1
        "#,
        id.clone()
    )
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or(ApiError::NotFound("El paciente no existe".into()))?
    .into();

    // Obtener el nombre de la raza si existe
    let breed_name: Option<String> = if let Some(breed_id) = patient.breed_id {
        sqlx::query_scalar!(
            r#"
            SELECT name
            FROM breeds
            WHERE id = $1
            "#,
            breed_id
        )
        .fetch_optional(pool.get_ref())
        .await?
    } else {
        None
    };

    // Construir la respuesta
    let mut response: PatientResponse = patient.into();
    response.breed = breed_name;

    Ok(HttpResponse::Ok().json(response))
}

/// Actualiza un paciente existente
///
/// # Ejemplo de petición
/// ```json
/// {
///   "name": "Max Updated",
///   "weight_kg": 13.0
/// }
/// ```
#[actix_web::put("/{id}")]
async fn update_patient(
    id: web::Path<i32>,
    updated_patient: web::Json<UpdatePatient>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Actualizando paciente ID: {}", id);

    let updated_patient = updated_patient.into_inner();
    updated_patient.validate()?;

    // Verificar si el paciente existe
    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM patients
            WHERE id = $1
        )
        "#,
    )
    .bind(id.clone())
    .fetch_one(pool.get_ref())
    .await?;

    if !exists {
        tracing::warn!("Intento de actualizar paciente inexistente ID: {}", id);
        return Err(ApiError::NotFound("El paciente no existe".into()));
    }

    // Actualizar el paciente
    let patient: Patient = sqlx::query_as!(
        PatientRaw,
        r#"
        UPDATE patients
        SET
            name = CASE WHEN $1::TEXT IS NOT NULL THEN $1 ELSE name END,
            species = CASE WHEN $2::animal_species IS NOT NULL THEN $2 ELSE species END,
            breed = CASE WHEN $3::INT IS NOT NULL THEN $3 ELSE breed END,
            birth_date = CASE WHEN $4::DATE IS NOT NULL THEN $4 ELSE birth_date END,
            gender = CASE WHEN $5::animal_gender IS NOT NULL THEN $5 ELSE gender END,
            weight_kg = CASE WHEN $6::DECIMAL IS NOT NULL THEN $6 ELSE weight_kg END,
            client_id = CASE WHEN $7::INT IS NOT NULL THEN $7 ELSE client_id END,
            photo_url = CASE WHEN $8::TEXT IS NOT NULL THEN $8 ELSE photo_url END
        WHERE id = $9
        RETURNING
            id,
            name,
            species as "species!: AnimalSpecies",
            breed as "breed_id!: Option<i32>",
            birth_date,
            gender as "gender!: Option<AnimalGender>",
            weight_kg as "weight_kg!: BigDecimal",
            client_id as "client_id!: i32",
            photo_url
        "#,
        updated_patient.name.map(|s| s.trim().to_string()),
        updated_patient.species as Option<AnimalSpecies>,
        updated_patient.breed_id.flatten(),
        updated_patient.birth_date,
        updated_patient.gender as Option<AnimalGender>,
        updated_patient
            .weight_kg
            .and_then(|f| BigDecimal::from_f64(f)),
        updated_patient.client_id,
        updated_patient.photo_url.map(|s| s.trim().to_string()),
        id.clone()
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Error al actualizar paciente: {}", e);
        ApiError::InternalServerError("Error al actualizar el paciente".into())
    })?
    .into();

    // Obtener el nombre de la raza si existe
    let breed_name: Option<String> = if let Some(breed_id) = patient.breed_id {
        sqlx::query_scalar!(
            r#"
            SELECT name
            FROM breeds
            WHERE id = $1
            "#,
            breed_id
        )
        .fetch_optional(pool.get_ref())
        .await?
    } else {
        None
    };

    // Construir la respuesta
    let mut response: PatientResponse = patient.into();
    response.breed = breed_name;

    Ok(HttpResponse::Ok().json(response))
}

/// Elimina un paciente existente
///
/// # Ejemplo
/// DELETE /patients/1
#[actix_web::delete("/{id}")]
async fn delete_patient(
    id: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Eliminando paciente ID: {}", id);

    // Verificar si el paciente existe
    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM patients
            WHERE id = $1
        )
        "#,
    )
    .bind(id.clone())
    .fetch_one(pool.get_ref())
    .await?;

    if !exists {
        tracing::warn!("Intento de eliminar paciente inexistente ID: {}", id);
        return Err(ApiError::NotFound("El paciente no existe".into()));
    }

    // Eliminar el paciente
    let rows_affected = sqlx::query!(
        r#"
        DELETE FROM patients
        WHERE id = $1
        "#,
        id.clone()
    )
    .execute(pool.get_ref())
    .await?
    .rows_affected();

    if rows_affected == 0 {
        tracing::warn!(
            "Paciente ID {} no encontrado después de intentar eliminar",
            id
        );
        return Err(ApiError::NotFound("El paciente no existe".into()));
    }

    tracing::info!("Paciente ID {} eliminado exitosamente", id);
    Ok(HttpResponse::NoContent().finish())
}

// Exporta todas las funciones como un grupo
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/patients")
            .service(create_patient)
            .service(list_patients)
            .service(get_patient)
            .service(update_patient)
            .service(delete_patient), // Agrega más servicios aquí...
    );
}
