use super::*;
use shared::domain::{NewAppointment, NewInvitation, InvitationParams, SendAppointmentEmails};
use shared::qr_client::QRClient;
use futures::future::join_all;
use url::Url;
use shared::email_client::EmailClient;
use crate::repository::{delete_stored_appointment, get_stored_appointments, preserve_new_appointment, preserve_new_invitations};

pub fn appointment_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/appointment")
            .route(web::post().to(add_appointment))
    )
        .service(
            web::resource("/appointment/{id}")
                .route(web::get().to(get_appointment_by_id))
                .route(web::delete().to(delete_appointment))
        )
        .service(
            web::resource("/appointment/{id}/invitation")
                .route(web::post().to(add_invitation))
        );
}



#[tracing::instrument(
    name = "Add invitation service handler",
    skip(pool, qr_client, email_client),
)]
pub async fn add_invitation(
    appointment_id: web::Path<Uuid>,
    query_params: web::Query<InvitationParams>,
    send_appointment_emails: web::Json<SendAppointmentEmails>,
    pool: Data<PgPool>,
    qr_client: Data<QRClient>,
    email_client: Data<EmailClient>
) -> Result<HttpResponse, CustomError> {
    let send_appointment_emails = send_appointment_emails.into_inner();
    let mut payload: Vec<NewInvitation>  = vec![];
    let appt_id = appointment_id.into_inner();
    let qr_client = qr_client.into_inner();
    let email_client = email_client.into_inner();

    let count = match &send_appointment_emails.email {
        None => { query_params.count.unwrap_or(1) }
        Some(emails) => {
            emails.len() as i32
        }
    };

    let futures: Vec<_> = (0..count).map(|_| {
        let id = Uuid::new_v4();
        let qr_client_clone = qr_client.clone();
        async move {
            qr_client_clone.get_short_url(id.to_string()).await.map(|res| (id, res.short_url))
        }
    }).collect();

    let results: Vec<Result<(Uuid, Url), anyhow::Error>> = join_all(futures).await;
    for result in results {
        let (id, short_url) = result?;
        payload.push(NewInvitation {
            id,
            appointment_id: appt_id,
            short_url
        });
    }

    let mut futures = vec![];

    if send_appointment_emails.email.is_some() {
        let email_invitation_composition = send_appointment_emails.email.as_ref()
            .expect("Can't extract email").iter().zip(payload.iter());
        for (email, invitation) in email_invitation_composition {
            let future = async {
                let image_string = qr_client.generate_qr_code_base64(invitation.short_url.clone()).await?;
                let email_template = render_template("email").await?;
                let html_body = email_template.replace("{IMAGE_STRING}", image_string.as_ref());
                let plain_body = "Hello dear customer.";
                email_client.send_email(
                    email,
                    "Your invitation",
                    &html_body,
                    plain_body
                ).await.map_err(|e| anyhow::anyhow!(e))
            };
            futures.push(future);
        }
    }

    for result in join_all(futures).await {
        result?;
    }

    let mut transaction = open_transaction(pool).await?;

    preserve_new_invitations(&mut transaction, &payload).await?;

    commit_transaction(transaction, "Failed to commit SQL transaction to store a new course.")
        .await?;

    Ok(
        HttpResponse::Ok()
            .json(payload)
    )
}

#[tracing::instrument(
    name = "Add new appointment",
    skip(pool),
)]
pub async fn add_appointment(
    new_appointment: web::Json<NewAppointment>,
    pool: Data<PgPool>
) -> Result<HttpResponse, CustomError> {
    let mut transaction = open_transaction(pool).await?;

    let response = preserve_new_appointment(&mut transaction, new_appointment.0).await?;

    commit_transaction(transaction, "Failed to commit SQL transaction to store a new course.")
        .await?;

    Ok(
        HttpResponse::Ok()
            .json(response)
    )
}

#[tracing::instrument(
    name = "Get appointment by id",
    skip(pool)
)]
pub async fn get_appointment_by_id(
    pool: Data<PgPool>,
    appointment_id: web::Path<Uuid>
) -> Result<HttpResponse, CustomError> {
    let response = get_stored_appointments(pool.as_ref(), Some(appointment_id.into_inner())).await?;
    Ok(
        HttpResponse::Ok()
            .json(response.first())
    )
}


#[tracing::instrument(
    name = "Delete appointment",
    skip(pool)
)]
pub async fn delete_appointment(
    pool: Data<PgPool>,
    appointment_id: web::Path<Uuid>
) -> Result<HttpResponse, CustomError> {

    let mut transaction = open_transaction(pool).await?;

    let response = delete_stored_appointment(&mut transaction, appointment_id.into_inner()).await?;

    commit_transaction(transaction, "Failed to commit SQL transaction to store a new course.")
        .await?;

    Ok(
        HttpResponse::Ok()
            .json(response)
    )
}