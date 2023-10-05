use actix_web::http;
use chrono::{NaiveDateTime, Utc};
use shared::domain::{AppointmentFormat, AppointmentWithInvitation, DBAppointmentWithInvitation};
use super::*;
use crate::error::CustomError;

pub fn validation_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/validations/{id}")
            .route(web::get().to(validate_invitation_by_id))
    );
}

#[tracing::instrument(
    name = "Validate invitation by id",
    skip(pool),
)]
pub(crate) async fn validate_invitation_by_id(
    invitation_id: web::Path<Uuid>,
    pool: Data<PgPool>
) -> Result<HttpResponse, CustomError> {
    let response = get_stored_invitation(pool.as_ref(), invitation_id.into_inner()).await?;

    let invitation = AppointmentWithInvitation::from(response);

    if invitation.used {
        return Err(CustomError::Forbidden("invitation has already been used".to_string()));
    }

    let now = Utc::now().naive_utc();
    let previous_date = now.date().pred_opt().expect("Failed to get the prev day");
    let midnight_time = chrono::NaiveTime::from_hms_opt(00, 00, 00).expect("Failed to create midnight time");
    let yesterday = NaiveDateTime::new(previous_date, midnight_time);

    println!("{} {} {}",  invitation.date.date(), yesterday.date(), invitation.date < yesterday);

    if invitation.date < yesterday {
        return Err(CustomError::Forbidden("The invitation date is outdated.".to_string()));
    }

    let http_response = match invitation.format {
        AppointmentFormat::ONLINE => {
            HttpResponse::PermanentRedirect()
                .append_header((http::header::LOCATION, invitation.link.unwrap().into_inner()))
                .finish()
        }
        AppointmentFormat::OFFLINE => {
            HttpResponse::Ok()
                .json(invitation)
        }
    };

    Ok(http_response)
}

pub(crate) async fn get_stored_invitation(
    pool: &PgPool,
    invitation_id: Uuid
) -> Result<DBAppointmentWithInvitation, CustomError> {
    let query = "
        SELECT invitation.id, invitation.appointment_id, invitation.used, invitation.short_url,
        appointment.link, appointment.format, appointment.address, appointment.date
        FROM invitation
        LEFT JOIN appointment
        ON invitation.appointment_id = appointment.id
        WHERE invitation.id = $1
    ".to_string();

    let result: DBAppointmentWithInvitation = sqlx::query_as::<_, DBAppointmentWithInvitation>(&query)
        .bind(invitation_id)
        .fetch_one(pool)
        .await?;

    Ok(result)
}