use super::*;

use uuid::Uuid;
use shared::qr_client::QRClient;
use crate::repository::get_stored_invitations;


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
async fn get_invitation_qr(
    invitation_id: web::Path<Uuid>,
    pool: Data<PgPool>,
    qr_client: Data<QRClient>
) -> Result<HttpResponse, CustomError> {
    let invitation_id = invitation_id.into_inner();
    let response = get_stored_invitations(
        pool.as_ref(),
        Some(invitation_id)
    ).await?;
    let qr_client = qr_client.into_inner();

    if response.is_empty() {
        return Err(CustomError::NotFound(format!("Not found for {}", invitation_id)));
    }

    let invitation = response.first().unwrap();

    let image_string = qr_client.generate_qr_code_base64(invitation.short_url.clone()).await?;
    let qr_template  = render_template("qr").await?;
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
async fn get_invitation_by_id(
    invitation_id: web::Path<Uuid>,
    pool: Data<PgPool>
) -> Result<HttpResponse, CustomError> {
    let response = get_stored_invitations(pool.as_ref(), Some(invitation_id.into_inner())).await?;
    Ok(
        HttpResponse::Ok()
            .json(response.first())
    )
}