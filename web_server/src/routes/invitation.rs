use super::*;

use shared::domain::{DBInvitation, Invitation};
use uuid::Uuid;
use shared::qr_client::QRClient;


pub fn invitation_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/invitation/{id}")
            .route(web::get().to(get_invitation_by_id))
    )
        .service(
            web::resource("/invitation/{id}/qr")
                .route(web::get().to(get_invitation_qr))
        );
}

#[tracing::instrument(
    name = "Get invitation QR",
    skip(pool, qr_client)
)]
pub async fn get_invitation_qr(
    invitation_id: web::Path<Uuid>,
    pool: Data<PgPool>,
    qr_client: Data<QRClient>
) -> Result<HttpResponse, CustomError> {
    let invitation_id = invitation_id.into_inner();
    let response = get_stored_invitations(pool.as_ref(), Some(invitation_id.clone())).await?;
    let qr_client = qr_client.into_inner();

    if response.is_empty() {
        return Err(CustomError::NotFound(format!("Not found for {}", invitation_id)));
    }

    let invitation = response.first().unwrap();

    let image_string = qr_client.generate_qr_code_base64(invitation.short_url.clone()).await?;
    let qr_template  = include_str!("./../templates/qr.html");
    let formatted_html = qr_template.replace("{IMAGE_STRING}", &image_string);
    Ok(
        HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(formatted_html)
    )
}

#[tracing::instrument(
    name = "Validate invitation by id",
    skip(pool)
)]
pub async fn get_invitation_by_id(
    invitation_id: web::Path<Uuid>,
    pool: Data<PgPool>
) -> Result<HttpResponse, CustomError> {
    let response = get_stored_invitations(pool.as_ref(), Some(invitation_id.into_inner())).await?;
    Ok(
        HttpResponse::Ok()
            .json(response.first())
    )
}

pub(crate) async fn get_stored_invitations(
    pool: &PgPool,
    appointment_id: Option<Uuid>
) -> Result<Vec<Invitation>, CustomError> {
    let mut query = "
        SELECT id, appointment_id, used, short_url
        FROM invitation
    ".to_string();

    if let Some(_) = appointment_id {
        query.push_str(" WHERE id = $1 ");
    }

    let records: Vec<DBInvitation> = if let Some(id) = appointment_id {
        sqlx::query_as::<_, DBInvitation>(&query)
            .bind(id)
            .fetch_all(pool).await?
    } else {
        sqlx::query_as::<_, DBInvitation>(&query)
            .fetch_all(pool).await?
    };

    let mut result = vec![];
    for record in records {
        result.push(Invitation::from(record))
    }
    Ok(result)
}