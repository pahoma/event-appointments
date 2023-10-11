use sqlx::{PgPool, Postgres, QueryBuilder, Transaction};
use uuid::Uuid;
use shared::domain::{DBInvitation, Invitation, NewInvitation};
use crate::error::CustomError;

pub(crate) async fn get_stored_invitations(
    pool: &PgPool,
    appointment_id: Option<Uuid>
) -> Result<Vec<Invitation>, CustomError> {
    let mut query = "
        SELECT id, appointment_id, used, short_url
        FROM invitation
    ".to_string();

    let records: Vec<DBInvitation> = if let Some(id) = appointment_id {
        query.push_str(" WHERE id = $1 ");
        sqlx::query_as::<_, DBInvitation>(&query)
            .bind(id)
            .fetch_all(pool).await?
    } else {
        sqlx::query_as::<_, DBInvitation>(&query)
            .fetch_all(pool).await?
    };

    let result = records.into_iter().map(Invitation::from).collect();
    Ok(result)
}

#[tracing::instrument(
name = "Preserve new invitations in DB",
skip(transaction),
)]
pub(crate) async fn preserve_new_invitations(
    transaction: &mut Transaction<'_, Postgres>,
    invitations: &Vec<NewInvitation>,
) -> Result<(), CustomError> {

    let mut query_builder = QueryBuilder::new("INSERT INTO invitation (id, appointment_id, short_url) ");

    query_builder.push_values(invitations, |mut b, new_invitation| {
        b
            .push_bind(new_invitation.id)
            .push_bind(new_invitation.appointment_id)
            .push_bind(new_invitation.short_url.as_str().to_string());
    });

    let query = query_builder.build();

    let _ = query.execute(&mut **transaction).await?;

    Ok(())
}