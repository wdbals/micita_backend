use crate::models::statistic::*;
use crate::{errors::ApiError, models::enums::UserRole};

use actix_web::{HttpResponse, web};
use sqlx::PgPool;

#[actix_web::get("")]
async fn get_statistics(
    query: web::Query<StatisticsQuery>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let query = query.into_inner();
    let mut response = StatisticsResponse {
        appointments_by_month: None,
        user_counts: None,
        procedures_by_type: None,
        patients_by_species: None,
        veterinarian_stats: None,
    };

    match query.role {
        UserRole::Admin => {
            if query.type_.is_none() || query.type_ == Some("appointments".to_string()) {
                response.appointments_by_month = Some(
                    get_appointments_by_month(pool.get_ref(), query.start_date, query.end_date)
                        .await?,
                );
            }
            if query.type_.is_none() || query.type_ == Some("users".to_string()) {
                response.user_counts = Some(get_user_counts(pool.get_ref()).await?);
            }
            if query.type_.is_none() || query.type_ == Some("procedures".to_string()) {
                response.procedures_by_type = Some(
                    get_procedures_by_type(pool.get_ref(), query.start_date, query.end_date)
                        .await?,
                );
            }
            if query.type_.is_none() || query.type_ == Some("patients".to_string()) {
                response.patients_by_species = Some(get_patients_by_species(pool.get_ref()).await?);
            }
        }
        UserRole::Veterinarian => {
            if let Some(user_id) = query.user_id {
                response.veterinarian_stats = Some(
                    get_veterinarian_stats(
                        pool.get_ref(),
                        user_id,
                        query.start_date,
                        query.end_date,
                    )
                    .await?,
                );
            } else {
                return Err(ApiError::ValidationError(
                    "El ID del veterinario es requerido".into(),
                ));
            }
        }
        UserRole::Assistant => {}
    }

    Ok(HttpResponse::Ok().json(response))
}

async fn get_appointments_by_month(
    pool: &PgPool,
    start_date: Option<chrono::NaiveDate>,
    end_date: Option<chrono::NaiveDate>,
) -> Result<Vec<AppointmentsByMonth>, ApiError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            TO_CHAR(start_time, 'YYYY-MM') AS month,
            COUNT(*) AS count
        FROM appointments
        WHERE ($1::date IS NULL OR start_time::date >= $1)
          AND ($2::date IS NULL OR start_time::date <= $2)
        GROUP BY month
        ORDER BY month ASC
        "#,
        start_date,
        end_date
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| AppointmentsByMonth {
            month: row.month.unwrap_or_default(),
            count: row.count.unwrap_or(0),
        })
        .collect())
}

async fn get_user_counts(pool: &PgPool) -> Result<UserCounts, ApiError> {
    let counts = sqlx::query!(
        r#"
        SELECT
            COUNT(*) AS total_users,
            SUM(CASE WHEN role = 'veterinarian' THEN 1 ELSE 0 END) AS veterinarians,
            SUM(CASE WHEN role = 'assistant' THEN 1 ELSE 0 END) AS assistants,
            SUM(CASE WHEN role = 'admin' THEN 1 ELSE 0 END) AS admins
        FROM users
        "#
    )
    .fetch_one(pool)
    .await?;

    Ok(UserCounts {
        total_users: counts.total_users.unwrap_or(0),
        veterinarians: counts.veterinarians.unwrap_or(0),
        assistants: counts.assistants.unwrap_or(0),
        admins: counts.admins.unwrap_or(0),
    })
}

async fn get_procedures_by_type(
    pool: &PgPool,
    start_date: Option<chrono::NaiveDate>,
    end_date: Option<chrono::NaiveDate>,
) -> Result<Vec<ProceduresByType>, ApiError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            p.type::text AS procedure_type,
            COUNT(*) AS count
        FROM patient_procedures pp
        JOIN procedures p ON pp.procedure_id = p.id
        WHERE ($1::date IS NULL OR pp.date >= $1)
          AND ($2::date IS NULL OR pp.date <= $2)
        GROUP BY p.type
        ORDER BY count DESC
        "#,
        start_date,
        end_date
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| ProceduresByType {
            procedure_type: row.procedure_type.unwrap_or_default(),
            count: row.count.unwrap_or(0),
        })
        .collect())
}

async fn get_patients_by_species(pool: &PgPool) -> Result<Vec<PatientsBySpecies>, ApiError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            species::text AS species,
            COUNT(*) AS count
        FROM patients
        GROUP BY species
        ORDER BY count DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| PatientsBySpecies {
            species: row.species.unwrap_or_default(),
            count: row.count.unwrap_or(0),
        })
        .collect())
}

async fn get_veterinarian_stats(
    pool: &PgPool,
    user_id: i32,
    start_date: Option<chrono::NaiveDate>,
    end_date: Option<chrono::NaiveDate>,
) -> Result<VeterinarianStats, ApiError> {
    // Citas por estado
    let appointments_by_status = sqlx::query!(
        r#"
        SELECT
            status::text AS status,
            COUNT(*) AS count
        FROM appointments
        WHERE veterinarian_id = $1
          AND ($2::date IS NULL OR start_time::date >= $2)
          AND ($3::date IS NULL OR start_time::date <= $3)
        GROUP BY status
        "#,
        user_id,
        start_date,
        end_date
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| AppointmentsByStatus {
        status: row.status.unwrap_or_default(),
        count: row.count.unwrap_or(0),
    })
    .collect();

    // Procedimientos realizados
    let procedures_performed = sqlx::query!(
        r#"
        SELECT
            p.type::text AS procedure_type,
            COUNT(*) AS count
        FROM patient_procedures pp
        JOIN procedures p ON pp.procedure_id = p.id
        WHERE pp.veterinarian_id = $1
          AND ($2::date IS NULL OR pp.date >= $2)
          AND ($3::date IS NULL OR pp.date <= $3)
        GROUP BY procedure_type
        "#,
        user_id,
        start_date,
        end_date
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| ProceduresByType {
        procedure_type: row.procedure_type.unwrap_or_default(),
        count: row.count.unwrap_or(0),
    })
    .collect();

    // Registros médicos creados
    let medical_records_created = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM medical_records
        WHERE veterinarian_id = $1
          AND ($2::date IS NULL OR date::date >= $2)
          AND ($3::date IS NULL OR date::date <= $3)
        "#,
        user_id,
        start_date,
        end_date
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    // Pacientes atendidos
    let patients_attended = sqlx::query!(
        r#"
        SELECT
            pa.species::text AS species,
            COUNT(DISTINCT pa.id) AS count
        FROM medical_records mr
        JOIN patients pa ON mr.patient_id = pa.id
        WHERE mr.veterinarian_id = $1
          AND ($2::date IS NULL OR mr.date::date >= $2)
          AND ($3::date IS NULL OR mr.date::date <= $3)
        GROUP BY pa.species
        "#,
        user_id,
        start_date,
        end_date
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| PatientsBySpecies {
        species: row.species.unwrap_or_default(),
        count: row.count.unwrap_or(0),
    })
    .collect();

    Ok(VeterinarianStats {
        appointments_by_status,
        procedures_performed,
        medical_records_created,
        patients_attended,
    })
}

// Exporta todas las funciones como un grupo
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/stats").service(get_statistics), // Agrega más servicios aquí...
    );
}
