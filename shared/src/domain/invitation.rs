use uuid::Uuid;
use crate::domain::Uri;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Deserialize, Serialize)]
pub struct NewInvitation {
    pub id: Uuid,
    pub appointment_id: Uuid,
    pub short_url: Uri
}

#[derive(Debug, Deserialize)]
pub struct EditInvitation {
    pub used: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct DBInvitation {
    pub id: Uuid,
    pub appointment_id: Uuid,
    pub used: bool,
    pub short_url: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invitation {
    pub id: Uuid,
    pub appointment_id: Uuid,
    pub used: bool,
    pub short_url: Uri
}



impl From<DBInvitation> for Invitation {
    fn from(db_invitation: DBInvitation) -> Self {
        Invitation {
            id: db_invitation.id,
            appointment_id: db_invitation.appointment_id,
            used: db_invitation.used,
            short_url: Uri::parse(db_invitation.short_url).expect("Can't convert String to Uri")
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct InvitationParams {
    pub count: Option<i32>
}