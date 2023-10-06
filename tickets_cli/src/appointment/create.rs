use std::io::{self, BufRead};
use tokio::task;
use chrono::{NaiveDateTime, DateTime};
use shared::domain::{AppointmentFormat, NewAppointment};
use std::time::Duration;
use anyhow::Error;
use reqwest::Url;
use uuid::Uuid;
use shared::configuration::get_configuration;

/// Reads a single line of input from stdin asynchronously.
///
/// # Returns
///
/// - An `io::Result<String>` containing the user's input or an error.
async fn read_line() -> io::Result<String> {
    task::spawn_blocking(|| {
        let mut input = String::new();
        let stdin = io::stdin();
        stdin.lock().read_line(&mut input)?;
        Ok(input.trim().to_string())
    })
        .await
        .unwrap()
}

/// Asynchronously prompts the user to specify an appointment format.
///
/// The user is repeatedly asked to choose between ONLINE and OFFLINE formats
/// until a valid input is provided.
///
/// # Returns
///
/// - The selected `AppointmentFormat`.
async fn read_appointment_format() -> AppointmentFormat {
    loop {
        println!("Enter format [0 || online ] - ONLINE or [ 1 || offline ] - OFFLINE):");
        let type_str = read_line().await.expect("Failed to read format");
        match type_str.as_str() {
            "0" => return AppointmentFormat::ONLINE,
            "1" => return AppointmentFormat::OFFLINE,
            _ => {
                println!("Invalid format. Please enter either `online` or `offline`.");
                continue;
            },
        };
    }
}

/// Asynchronously reads non-empty input from the user.
///
/// The user will be repeatedly prompted until they provide non-empty input.
///
/// # Parameters
///
/// - `prompt`: The message that will be displayed to the user prompting them for input.
///
/// # Returns
///
/// - The user's non-empty input as a `String`.
async fn read_non_empty_input(prompt: &str) -> String {
    loop {
        println!("{}", prompt);
        let input = read_line().await.expect("Failed to read input");
        if !input.is_empty() {
            return input;
        }
        println!("Input cannot be empty. Please try again.");
    }
}

/// Asynchronously reads an optional address from the user.
///
/// The behavior of the prompt is determined by the provided `AppointmentFormat`.
///
/// # Parameters
///
/// - `format`: The format of the appointment (ONLINE or OFFLINE).
/// - `prompt`: The message that will be displayed to the user prompting them for input.
///
/// # Returns
///
/// - An optional `String` containing the user's input address or `None`.
async fn read_optional_address(format: &AppointmentFormat, prompt: &str) -> Option<String> {
    loop {
        match format {
            AppointmentFormat::ONLINE => {
                return None;
            }
            AppointmentFormat::OFFLINE => {
                println!("{}", prompt);
                let input = read_line().await.expect("Failed to read input");
                return match input.as_str() {
                    "none" => continue,
                    _ => Some(input),
                };
            }
        };
    }
}

/// Asynchronously reads a URI input from the user.
///
/// The behavior of the prompt is determined by the provided `AppointmentFormat`.
///
/// # Parameters
///
/// - `format`: The format of the appointment (ONLINE or OFFLINE).
/// - `prompt`: The message that will be displayed to the user prompting them for input.
///
/// # Returns
///
/// - An optional `Url` containing the user's input URI or `None`.
async fn read_uri_input(format: &AppointmentFormat, prompt: &str) -> Option<Url> {
    loop {
        match format {
            AppointmentFormat::OFFLINE => {
                return None;
            }
            AppointmentFormat::ONLINE => {
                println!("{}", prompt);
                let input = read_line().await.expect("Failed to read input");
                match input.as_str().trim() {
                    "none" => return None,
                    _ => {
                        match Url::parse(input.as_str()) {
                            Ok(uri) => return Some(uri),
                            Err(e) => {
                                println!("Error: {}", e);
                                println!("Please enter a valid URI or 'none'.");
                            }
                        }
                    }
                }
            }
        }
    }
}


/// Asynchronously reads a duration input in minutes from the user.
///
/// The user will be repeatedly prompted until they provide a valid duration input
/// in minutes (as an integer). The input is then converted to a `Duration` representation
/// in seconds.
///
/// # Parameters
///
/// - `prompt`: The message that will be displayed to the user prompting them to enter the duration in minutes.
/// - `error_msg`: The message that will be displayed to the user if the input is not a valid integer.
///
/// # Returns
///
/// - A `Duration` representation of the user input in seconds.
///
/// # Examples
///
/// ```rust, no_run
/// # use std::time::Duration;
/// # async fn main() {
/// let duration = read_duration_in_minutes_input("Please enter a duration in minutes:", "Invalid input! Please enter a number.").await;
/// println!("Parsed duration in seconds: {:?}", duration.as_secs());
/// # }
/// ```
async fn read_duration_in_minutes_input(prompt: &str, error_msg: &str) -> Duration {
    loop {
        println!("{}", prompt);
        let input = read_line().await.expect("Failed to read input");
        match input.parse::<u64>() {
            Ok(value) => return Duration::from_secs(value * 60), // Convert minutes to seconds
            Err(_) => println!("{}", error_msg),
        }
    }
}

/// Asynchronously reads a datetime input from the user.
///
/// The user will be repeatedly prompted until they provide a valid datetime input.
/// The function supports two formats for datetime:
/// 1. Direct "yyyy/mm/dd hh:mm" format.
/// 2. RFC3339 format, which includes timezone information. The timezone will be stripped
///    and a `NaiveDateTime` representation will be returned.
///
/// # Parameters
///
/// - `prompt`: The message that will be displayed to the user prompting them to enter the datetime.
/// - `error_msg`: The message that will be displayed to the user if the input is invalid.
///
/// # Returns
///
/// - A `NaiveDateTime` representation of the user input.
///
/// # Examples
///
/// ```rust, no_run
/// # use chrono::{NaiveDateTime, DateTime};
/// # async fn main() {
/// let datetime = read_datetime_input("Please enter a datetime:", "Invalid format! Try again.").await;
/// println!("Parsed datetime: {:?}", datetime);
/// # }
/// ```
async fn read_datetime_input(prompt: &str, error_msg: &str) -> NaiveDateTime {
    loop {
        println!("{}", prompt);
        let input = read_line().await.expect("Failed to read input");

        // Try parsing it directly as "yyyy/mm/dd hh:mm" format
        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(&input, "%Y/%m/%d %H:%M") {
            return naive_dt;
        }

        // If the above parsing failed, try parsing it with timezone information (RFC3339 format) and then strip the timezone
        if let Ok(dt) = DateTime::parse_from_rfc3339(&input) {
            return dt.naive_utc(); // Extract NaiveDateTime from DateTime
        }

        // If both parsing attempts failed, print an error message
        println!("{}", error_msg);
    }
}

/// Asynchronously constructs a new appointment by reading input from stdin.
///
/// # Returns
///
/// - A `NewAppointment` constructed from user's input.
pub async fn read_new_appointment_from_stdin() -> NewAppointment {
    let title = read_non_empty_input("Enter title:").await;
    let description = read_non_empty_input("Enter description:").await;
    let format = read_appointment_format().await;
    let address = read_optional_address(&format, "Enter address:").await;
    let link = read_uri_input(&format, "Enter uri:").await;
    let date = read_datetime_input("Enter start date in 'yyyy/mm/dd hh:mm' UTC time format:", "Please enter a valid datetime.").await;
    let duration = read_duration_in_minutes_input("Enter duration in minutes:", "Please enter a valid number for duration.").await;

    NewAppointment {
        title,
        description,
        format,
        address,
        link,
        date,
        duration
    }
}

/// Asynchronously handles the process of creating a new appointment.
///
/// This function reads a new appointment from stdin, sends it to the server,
/// and returns the server's response.
///
/// # Returns
///
/// - A `Result<String, Error>` representing the server's response or an error.
pub async fn new_appointment_handler() -> Result<String, Error> {
    let appointment: NewAppointment = read_new_appointment_from_stdin().await;
    send_new_appointment_handler(appointment).await
}

async fn send_new_appointment_handler(appointment: NewAppointment) -> Result<String, Error> {
    let client = reqwest::Client::new();
    let configuration = get_configuration().unwrap();

    println!("{:?}", appointment);

    let response = client.post(format!("{}/api/appointment", configuration.console_cli.web_url))
        .json(&appointment)
        .send()
        .await?;

    Ok(format!("Response: {:?}", response.json::<Uuid>().await?))
}


#[cfg(test)]
mod tests {
    use std::env;
    use std::str::FromStr;
    use super::*;
    use mockito::{mock, Matcher, server_url};
    use crate::appointment::ENV_VAR_LOCK_TEST;

    fn mock_read_new_appointment_from_stdin() -> NewAppointment {
        NewAppointment {
            title: "Test Title".to_string(),
            description: "Test Description".to_string(),
            format: AppointmentFormat::ONLINE,
            address: None,
            link: Some(Url::from_str("http://www.rust.com/").unwrap()),
            date: NaiveDateTime::from_timestamp(1672531200, 0), // Arbitrary date
            duration: Duration::from_secs(3600), // 1 hour
        }
    }

    #[tokio::test]
    async fn test_new_appointment_handler() {
        let _guard = ENV_VAR_LOCK_TEST.lock().unwrap();
        env::set_var("APP_CONSOLE_CLI__WEB_URL", server_url().as_str());
        // env::set_var::<&str, &str>("APP_CONSOLE_CLI__WEB_URL", server_url().as_str());

        // Mock the HTTP POST endpoint for creating a new appointment
        let _mock = mock("POST", Matcher::Exact("/api/appointment".to_string()))
            .with_status(200)
            .with_body(r#""6ba7b812-9dad-11d1-80b4-00c04fd430c9""#)
            .create();

        // This assumes the user will input data correctly on the first attempt for all prompts.
        // In a real-world scenario, you'd refactor your handler to accept a custom input function for easier testing.
        let response = send_new_appointment_handler(mock_read_new_appointment_from_stdin()).await;

        // Check the results.
        assert_eq!(response.unwrap(), r#"Response: 6ba7b812-9dad-11d1-80b4-00c04fd430c9"#);

        env::remove_var("APP_CONSOLE_CLI__WEB_URL");
    }
}
