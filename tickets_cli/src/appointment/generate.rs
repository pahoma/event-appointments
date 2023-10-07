use anyhow::{anyhow, Error};
use uuid::Uuid;
use shared::configuration::{get_configuration};
use shared::domain::{NewInvitation, SendAppointmentEmails};
use crate::QR_CLIENT;

// Sends a request to the server to create invitations
// and returns the invitations.
async fn fetch_invitations(
    web_url: String,
    appt_id: Uuid,
    count: i32
) -> Result<Vec<NewInvitation>, Error> {
    let client = reqwest::Client::new();
    let data = SendAppointmentEmails { email: None };

    let response = client.post(format!("{}/api/appointment/{}/invitation?count={}", web_url, appt_id, count))
        .json(&data)
        .send()
        .await?;

    let invitations: Vec<NewInvitation> = response.json::<Vec<NewInvitation>>().await?;
    Ok(invitations)
}

// Generates QR codes for the given list of invitations.
async fn generate_qr_codes_for_invitations(
    invitations: Vec<NewInvitation>
) -> Result<Vec<String>, Error> {
    let mut results: Vec<String> = vec![];

    for invitation in invitations {
        let result: String = QR_CLIENT.generate_qr_code(
            invitation.short_url,
            invitation.appointment_id,
            invitation.id
        ).await?;
        results.push(result);
    }

    Ok(results)
}

pub async fn generate_invitation_handler(
    appt_id: Uuid,
    count: Option<i32>
) -> Result<String, Error> {
    let configuration = get_configuration()
        .map_err(|e| anyhow!("Failed to get configuration: {}", e))?;

    let count: i32 = count.unwrap_or(1);

    let invitations = fetch_invitations(configuration.console_cli.web_url, appt_id, count).await?;
    let qr_codes = generate_qr_codes_for_invitations(invitations).await?;

    Ok(format!("Generated to: {:?}", qr_codes))
}


#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};
    use std::env;
    use crate::appointment::ENV_VAR_LOCK_TEST;

    #[tokio::test]
    async fn test_fetch_invitations() {
        let _guard = ENV_VAR_LOCK_TEST.lock().unwrap();

        // Set the environment variable to override the web_url to point to the mockito server
        env::set_var("APP_CONSOLE_CLI__WEB_URL", server_url().as_str());

        let test_appt_id = Uuid::parse_str("6ba7b812-9dad-11d1-80b4-00c04fd430c9").unwrap();

        // Mocking the HTTP POST endpoint for fetch invitations
        let mock_endpoint = format!("/api/appointment/{}/invitation?count=1", test_appt_id);
        let _mock = mock("POST", mock_endpoint.as_str())
            .with_status(200)
            .with_body(r#"[{"id":"6ba7b810-9dad-11d1-80b4-00c04fd430c8","appointment_id":"6ba7b812-9dad-11d1-80b4-00c04fd430c9","short_url":"https://example.com/shorturl"}]"#)
            .create();

        let result = fetch_invitations(server_url(), test_appt_id, 1).await;

        println!("{:?}", result);

        // Cleanup: Optionally remove the environment variable setting after the test
        env::remove_var("APP_CONSOLE_CLI__WEB_URL");

        assert!(result.is_ok());
        let invitations = result.unwrap();
        assert_eq!(invitations.len(), 1);
        assert_eq!(invitations[0].id, Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8").unwrap());
        assert_eq!(invitations[0].appointment_id, test_appt_id);
        assert_eq!(invitations[0].short_url.to_string(), "https://example.com/shorturl".to_string());
    }
}
