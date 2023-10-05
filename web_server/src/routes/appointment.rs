use super::*;
use shared::domain::{Appointment, DBAppointment, NewAppointment, NewInvitation, InvitationParams, SendAppointmentEmails};
use shared::qr_client::QRClient;
use futures::future::join_all;
use shared::email_client::EmailClient;

pub fn appointment_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/appointment")
            .route(web::post().to(add_appointment))
    )
        .service(
            web::resource("/appointment/{id}")
                .route(web::get().to(get_appointment_by_id))
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
            let res = qr_client_clone.get_short_url(id.to_string()).await.unwrap();
            (id, res.short_url)
        }
    }).collect();

    let results = join_all(futures).await;

    for (id, short_url) in results {
        payload.push(NewInvitation {
            id,
            appointment_id: appt_id.clone(),
            short_url
        });
    }

    if let Some(emails) = send_appointment_emails.email {
        for (email, invitation) in emails.iter().zip(payload.iter()) {
            let image_string = qr_client.generate_qr_code_base64(invitation.short_url.clone()).await?;
            println!("{}", &image_string);
            let email_template  = include_str!("./../templates/email.html");
            let html_body = email_template.replace("{IMAGE_STRING}", &image_string);
            println!("{}", &html_body);
            let plain_body = "Hello dear customer.";
            let _ = email_client.send_email(
                email,
                "You invitation",
                &html_body,
                &plain_body
            ).await;
        }
    }

    let mut transaction = open_transaction(pool).await?;

    let _ = preserve_new_invitations(&mut transaction, &payload).await?;

    commit_transaction(transaction, "Failed to commit SQL transaction to store a new course.")
        .await?;

    Ok(
        HttpResponse::Ok()
            .json(payload)
    )
}

#[tracing::instrument(
    name = "Preserve new invitations in DB",
    skip(transaction),
)]
async fn preserve_new_invitations(
    transaction: &mut Transaction<'_, Postgres>,
    invitations: &Vec<NewInvitation>,
) -> Result<(), CustomError> {

    let mut query_builder = QueryBuilder::new("INSERT INTO invitation (id, appointment_id, short_url) ");

    query_builder.push_values(invitations, |mut b, new_invitation| {
        b
            .push_bind(new_invitation.id)
            .push_bind(new_invitation.appointment_id)
            .push_bind(new_invitation.short_url.clone().into_inner());
    });

    let query = query_builder.build();

    let _ = query.execute(&mut **transaction).await?;

    Ok(())
}

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
    name = "Preserve new appointment in DB",
    skip(transaction),
)]
async fn preserve_new_appointment(
    transaction: &mut Transaction<'_, Postgres>,
    new_appointment: NewAppointment
) -> Result<Uuid, CustomError> {

    let link = match new_appointment.link {
        None => None,
        Some(link) => Some(link.into_inner())
    };
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
    name = "Get stored appointments from DB",
    skip(pool),
)]
async fn get_stored_appointments(
    pool: &PgPool,
    appointment_id: Option<Uuid>
) -> Result<Vec<Appointment>, CustomError> {
    let mut query = "
        SELECT id, title, description, format, address, link, date, duration
        FROM appointment
    ".to_string();

    if let Some(_) = appointment_id {
        query.push_str(" WHERE id = $1 ");
    }

    let records: Vec<DBAppointment> = if let Some(id) = appointment_id {
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