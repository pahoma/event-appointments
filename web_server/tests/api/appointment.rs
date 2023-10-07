use crate::helpers::spawn_app;

#[cfg(test)]
mod appointment_tests {
    use std::time::Duration;
    use actix_web::http::StatusCode;
    use reqwest::Client;
    use serde_json::json;
    use uuid::Uuid;
    use shared::domain::{Appointment, NewInvitation};
    use super::*;

    fn get_appointment_data() -> serde_json::Value {
        json!({
            "title": "Test appoinment",
            "description": "Some test desctiption",
            "format": "ONLINE",
            "address": "123 Fake St.",
            "link": "https://meeting-test.com",
            "date": "2024-10-10T10:10:00",
            "duration": 6000
        })
    }


    #[actix_web::test]
    async fn test_add_appoinment() {
        let application = spawn_app().await;

        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        let new_appoinment = get_appointment_data();

        let response = client.post(&format!("{}/api/appointment", &application.address))
            .json(&new_appoinment)
            .send()
            .await
            .expect("Failed to add new apoinment");

        assert_eq!(response.status(), StatusCode::OK);

        let returned_appoinment_id: Uuid = response.json().await.expect("Failed to parse response");

        assert_ne!(returned_appoinment_id, Uuid::nil());
    }

    #[actix_web::test]
    async fn test_get_appoinment_by_id() {
        let application = spawn_app().await;

        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        let new_appoinment = get_appointment_data();

        let response = client.post(&format!("{}/api/appointment", &application.address))
            .json(&new_appoinment)
            .send()
            .await
            .expect("Failed to add new appointment");

        let appointment_id: Uuid = response.json().await.expect("Failed to parse response");

        let response = client.get(&format!("{}/api/appointment/{}", application.address, appointment_id.to_string()))
            .send()
            .await
            .expect("Failed to fetch appointment");

        assert_eq!(response.status(), StatusCode::OK);

        let returned_appointment: Appointment = response.json().await.expect("Failed to parse response");
        assert_eq!(returned_appointment.id, appointment_id);
    }

    #[actix_web::test]
    async fn test_delete_appointment() {
        let application = spawn_app().await;

        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        let new_appoinment = get_appointment_data();

        let response = client.post(&format!("{}/api/appointment", &application.address))
            .json(&new_appoinment)
            .send()
            .await
            .expect("Failed to add new appointment");

        let appointment_id: Uuid = response.json().await.expect("Failed to parse response");

        let response = client.delete(&format!("{}/api/appointment/{}", application.address, appointment_id.to_string()))
            .send()
            .await
            .expect("Failed to delete student");

        assert_eq!(response.status(), StatusCode::OK);

        let was_deleted: bool = response.json().await.expect("Failed to parse response");
        assert!(was_deleted);
    }

    #[actix_web::test]
    async fn test_create_invitation() {
        let application = spawn_app().await;

        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        let new_appoinment = get_appointment_data();

        let response = client.post(&format!("{}/api/appointment", &application.address))
            .json(&new_appoinment)
            .send()
            .await
            .expect("Failed to add new appointment");

        let appointment_id: Uuid = response.json().await.expect("Failed to parse response");

        let count = 1;

        let response = client.post(&format!("{}/api/appointment/{}/invitation?count={}", application.address, appointment_id.to_string(), count))
            .json(&json!({}))
            .send()
            .await
            .expect("Failed to add course to student");

        assert_eq!(response.status(), StatusCode::OK);

        let new_invitation: Vec<NewInvitation> = response.json().await.expect("Failed to parse response");

        assert_eq!(new_invitation.len(), count);
    }
}
