//! Module that provides CLI-based utilities and operations related to appointments.

mod cli;
mod appointment;

use once_cell::sync::Lazy;
use tokio;
use structopt::StructOpt;
use shared::configuration::get_configuration;
use shared::qr_client::QRClient;
use crate::appointment::{
    delete_appointment_handler,
    new_appointment_handler,
    generate_invitation_handler,
    send_invitation_letter_handler
};
use crate::cli::{AppointmentCommand, CliArgs, Command};

/// QRClient instance initialized lazily based on the application's configuration.
static QR_CLIENT: Lazy<QRClient> = Lazy::new(|| {
    let configuration = get_configuration().unwrap();

    QRClient::new(
        configuration.qr_client.api_url.clone(),
        configuration.qr_client.api_key.clone(),
        configuration.qr_client.base_url.clone(),
        configuration.qr_client.base_image_path.clone(),
        configuration.qr_client.timeout(),
    )
});

/// Entry point of the application.
///
/// Parses the command line arguments and executes the corresponding command.
/// Outputs the result or an error message to the console.
///
/// # Returns
///
/// * `anyhow::Result<()>` - Indicates successful execution or contains an error.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: CliArgs = cli::CliArgs::from_args();

    match execute_cmd(args).await {
        Ok(response) => println!("{}", response),
        Err(e) => eprintln!("Error: {}", e),
    }

    Ok(())
}

/// Executes the command specified in the given `CliArgs`.
///
/// Based on the parsed command line arguments, it performs various actions such as:
/// - Initializing the database
/// - Authentication-related tasks
/// - Appointment-related tasks including creation, deletion, generating QR invitations, and sending email invitations.
///
/// # Parameters
///
/// * `args: CliArgs` - Parsed command line arguments.
///
/// # Returns
///
/// * `Result<String, Box<dyn std::error::Error>>` - The result of the command's execution or an error.
async fn execute_cmd(args: CliArgs) -> Result<String, Box<dyn std::error::Error>> {
    let response = match args.cmd {
        Command::Init => {
            let _ = shared::db::initialize().await.unwrap();
            "Init".to_string()
        },
        Command::Auth => {
            todo!();
        },
        Command::Appointment(appt_cmd) => {
            match appt_cmd.appt_command {
                AppointmentCommand::Create => {
                    new_appointment_handler().await.unwrap()
                }
                AppointmentCommand::Delete { uuids} => {
                    delete_appointment_handler(uuids).await.unwrap().to_string()
                }
                AppointmentCommand::Generate { appt_id, count } => {
                    generate_invitation_handler(appt_id, count).await.unwrap()
                }
                AppointmentCommand::Send { appt_id, email } => {
                    send_invitation_letter_handler(appt_id, email).await.unwrap()
                }
            }
        }
    };

    Ok(response)
}
