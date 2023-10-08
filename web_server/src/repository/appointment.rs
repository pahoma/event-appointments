use anyhow::Context;
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;
use shared::domain::{Appointment, DBAppointment, NewAppointment};
use crate::error::CustomError;

#[tracing::instrument(
name = "Preserve new appointment in DB",
skip(transaction),
)]
pub(crate) async fn preserve_new_appointment(
    transaction: &mut Transaction<'_, Postgres>,
    new_appointment: NewAppointment
) -> Result<Uuid, CustomError> {

    let link = new_appointment.link.map(|link| link.to_string());
    let resp = sqlx::query(
        r#"
            INSERT INTO appointment (title, description, format, address, link, date, duration)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id;
        "#
    )
        .bind(new_appointment.title)
        .bind(new_appointment.description)
        .bind(new_appointment.format)
        .bind(new_appointment.address)
        .bind(link)
        .bind(new_appointment.date)
        .bind(new_appointment.duration.as_secs() as i64)
        .fetch_one(&mut **transaction)
        .await
        .context("Failed to insert a new appointment into the database.")?;

    let id: Uuid = resp.get("id");

    Ok(id)
}

#[tracing::instrument(
name = "Get stored appointments from DB",
skip(pool),
)]
pub(crate) async fn get_stored_appointments(
    pool: &PgPool,
    appointment_id: Option<Uuid>
) -> Result<Vec<Appointment>, CustomError> {
    let mut query = "
        SELECT id, title, description, format, address, link, date, duration
        FROM appointment
    ".to_string();

    let records: Vec<DBAppointment> = if let Some(id) = appointment_id {
        query.push_str(" WHERE id = $1 ");
        sqlx::query_as::<_, DBAppointment>(&query)
            .bind(id)
            .fetch_all(pool).await?
    } else {
        sqlx::query_as::<_, DBAppointment>(&query)
            .fetch_all(pool).await?
    };

    let mut result = vec![];
    for record in records {
        result.push(Appointment::from(record))
    }
    Ok(result)
}


#[tracing::instrument(
name = "Remove appointment stored in DB",
skip(transaction)
)]
pub(crate) async fn delete_stored_appointment(
    transaction: &mut Transaction<'_, Postgres>,
    appointment_id: Uuid
) -> Result<bool, CustomError> {
    let rows_affected = sqlx::query("DELETE FROM appointment WHERE id = $1")
        .bind(appointment_id)
        .execute(&mut **transaction)
        .await?
        .rows_affected();

    Ok(rows_affected > 0)
}