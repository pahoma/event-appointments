use std::str::FromStr;
use chrono::NaiveDateTime;
use crate::domain::{Email};
use url::Url;
use std::time::Duration;
use uuid::Uuid;
use serde::{Deserialize, Serialize, Deserializer, Serializer};
use sqlx::FromRow;


#[derive(Debug, Clone, Deserialize, Serialize, sqlx::Type)]
#[sqlx(type_name = "format")]
pub enum AppointmentFormat {
    ONLINE,
    OFFLINE
}

fn serialize_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
{
    serializer.serialize_u64(duration.as_secs())
}

// Implement deserialization for Duration
fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
{
    let secs = u64::deserialize(deserializer)?;
    Ok(Duration::from_secs(secs))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewAppointment {
    pub title: String,
    pub description: String,
    pub format: AppointmentFormat,
    pub address: Option<String>,
    pub link: Option<Url>,
    pub date: NaiveDateTime,
    #[serde(serialize_with = "serialize_duration", deserialize_with = "deserialize_duration")]
    pub duration: Duration,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct DBAppointment {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub format: AppointmentFormat,
    pub address: Option<String>,
    pub link: Option<String>,
    pub date: NaiveDateTime,
    pub duration: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Appointment {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub format: AppointmentFormat,
    pub address: Option<String>,
    pub link: Option<Url>,
    pub date: NaiveDateTime,
    #[serde(serialize_with = "serialize_duration", deserialize_with = "deserialize_duration")]
    pub duration: Duration,
}

impl From<DBAppointment> for Appointment {
    fn from(db_appointment: DBAppointment) -> Self {
        let link = match db_appointment.link {
            None => None,
            Some(s) => Some(Url::from_str(&s).expect("Can't convert String to Url"))
        };
        Appointment {
            id: db_appointment.id,
            title: db_appointment.title,
            description: db_appointment.description,
            format: db_appointment.format,
            address: db_appointment.address,
            link,
            date: db_appointment.date,
            duration: Duration::from_secs(db_appointment.duration as u64), // Convert i64 to Duration
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DBAppointmentWithInvitation {
    pub id: Uuid,
    pub appointment_id: Uuid,
    pub used: bool,
    pub short_url: String,
    pub format: AppointmentFormat,
    pub address: Option<String>,
    pub link: Option<String>,
    pub date: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppointmentWithInvitation {
    pub id: Uuid,
    pub appointment_id: Uuid,
    pub used: bool,
    pub short_url: Url,
    pub format: AppointmentFormat,
    pub address: Option<String>,
    pub link: Option<Url>,
    pub date: NaiveDateTime,
}

impl From<DBAppointmentWithInvitation> for AppointmentWithInvitation {
    fn from(db_appt_with_invitation: DBAppointmentWithInvitation) -> Self {
        let link = match db_appt_with_invitation.link {
            None => None,
            Some(s) => Some(Url::parse(&s).expect("Can't convert String to Url"))
        };
        let short_url = Url::parse(&db_appt_with_invitation.short_url).expect("Can't convert String to Url");
        AppointmentWithInvitation {
            id: db_appt_with_invitation.id,
            appointment_id: db_appt_with_invitation.appointment_id,
            used: db_appt_with_invitation.used,
            short_url,
            format: db_appt_with_invitation.format,
            address: db_appt_with_invitation.address,
            link,
            date: db_appt_with_invitation.date,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RemoveAppointment {
    pub id: Vec<Uuid>
}

#[derive(Debug, Deserialize)]
pub struct SendAppointment {
    pub id: Uuid,
    pub email: Vec<Email>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendAppointmentEmails {
    pub email: Option<Vec<Email>>
}