use std::collections::HashSet;
use anyhow::{anyhow, Error};
use uuid::Uuid;
use shared::domain::{Email, NewInvitation, SendAppointment, SendAppointmentEmails};
use tokio::{io};
use tokio::io::{AsyncBufReadExt, BufReader};
use std::io::ErrorKind;
use shared::configuration::get_configuration;

/// Asynchronously reads a list of emails from the standard input.
///
/// Users can input multiple emails by entering one email per line.
/// The input reading will stop when an empty line is encountered.
///
/// # Returns
///
/// - A `Result<Vec<Email>, io::Error>` representing the list of emails
///   read from the standard input or an error.
pub async fn read_emails() -> io::Result<Vec<Email>> {
    println!("Please list emails to send Appointment invitation':");

    let mut reader = BufReader::new(io::stdin());
    let mut emails = Vec::new();

    loop {
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line).await?;

        if bytes_read == 0 || line.trim().is_empty() {
            break;
        }
        let email = match Email::parse(line.trim().to_string()) {
            Ok(email) => email,
            Err(_) => return Err(io::Error::new(ErrorKind::InvalidData, "Failed to parse Email")),
        };

        emails.push(email);
    }

    Ok(emails)
}
// pub async fn read_emails() -> io::Result<Vec<Email>> {
//     println!("Please list emails to send Appointment invitation':");
//     task::spawn_blocking(|| {
//         let stdin = std::io::stdin();
//         let locked = stdin.lock();
//         let mut emails = Vec::new();
//
//         for line in locked.lines() {
//             let line = line?;
//             if line.trim().is_empty() {
//                 break;
//             }
//             let email = match Email::parse(line) {
//                 Ok(email) => email,
//                 Err(_) => return Err(io::Error::new(io::ErrorKind::InvalidData, "Failed to parse Email")),
//             };
//             emails.push(email);
//         }
//
//         Ok(emails)
//     })
//         .await
//         .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
// }

/// Asynchronously handles the sending of appointment invitation letters.
///
/// Given an appointment ID and an optional list of emails, this function prepares
/// and sends invitation letters. If no emails are provided, the user is prompted
/// to enter emails via standard input.
///
/// After ensuring the list of emails is sorted and deduplicated, an HTTP request
/// is made to send the invitations, and the resulting invitations are returned.
///
/// # Parameters
///
/// - `appt_id`: The UUID of the appointment for which the invitation letters are to be sent.
/// - `email`: An optional list of emails to which the invitations will be sent.
///
/// # Returns
///
/// - A `Result<String, Error>` containing a string representation of generated invitations
///   or an error.
pub async fn send_invitation_letter_handler(
    appt_id: Uuid,
    email: Vec<Email>
) -> Result<String, Error> {
    let configuration = get_configuration()
        .map_err(|e| anyhow!("Failed to get configuration: {}", e))?;

    let email_list = if email.is_empty() {
        read_emails().await?
    } else {
        email
    };

    let unique_emails: Vec<_> = email_list
        .into_iter().collect::<HashSet<_>>().into_iter().collect();

    let payload = SendAppointment {
        id: appt_id,
        email: unique_emails
    };

    let client = reqwest::Client::new();
    let data = SendAppointmentEmails { email: Some(payload.email) };

    let response = client.post(format!("{}/api/appointment/{}/invitation", configuration.console_cli.web_url, payload.id))
        .json(&data)
        .send()
        .await?;

    let invitations = response.json::<Vec<NewInvitation>>().await?;

    Ok(format!("Generated invitations: {:?}", invitations))
}



#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use super::*;
    use mockito::{mock, server_url};
    use std::env;
    use crate::appointment::ENV_VAR_LOCK_TEST; // Import the environment variable handling module

    #[tokio::test]
    async fn test_send_invitation_letter_handler() {
        // Set the environment variable to override the web_url to point to the mockito server
        let _guard = ENV_VAR_LOCK_TEST.lock().unwrap();
        env::set_var("APP_CONSOLE_CLI__WEB_URL", server_url().as_str());

        let test_appt_id = Uuid::parse_str("6ba7b812-9dad-11d1-80b4-00c04fd430c9").unwrap();
        let test_email = vec![Email::from_str("test@gmail.com").unwrap()];

        // Mocking the HTTP POST endpoint for sending invitations
        let mock_endpoint = format!("/api/appointment/{}/invitation", test_appt_id);
        let _mock = mock("POST", mock_endpoint.as_str())
            .with_status(200)
            .with_body(r#"[{"id":"6ba7b810-9dad-11d1-80b4-00c04fd430c8","appointment_id":"6ba7b812-9dad-11d1-80b4-00c04fd430c9","short_url":"https://example.com/shorturl"}]"#)
            .create();

        let result = send_invitation_letter_handler(test_appt_id, test_email).await;

        println!("{:?}", result);

        // Cleanup: Optionally remove the environment variable setting after the test
        env::remove_var("APP_CONSOLE_CLI__WEB_URL");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Generated invitations: [NewInvitation { id: 6ba7b810-9dad-11d1-80b4-00c04fd430c8, appointment_id: 6ba7b812-9dad-11d1-80b4-00c04fd430c9, short_url: Url { scheme: \"https\", cannot_be_a_base: false, username: \"\", password: None, host: Some(Domain(\"example.com\")), port: None, path: \"/shorturl\", query: None, fragment: None } }]");
    }
}
