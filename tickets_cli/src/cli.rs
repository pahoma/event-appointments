use structopt::StructOpt;
use uuid::Uuid;
use shared::domain::Email;

/// Command-line arguments for the application.
#[derive(Debug, StructOpt)]
pub struct CliArgs {
    /// Optional authentication token to authorize certain operations.
    #[structopt(short, long = "auth-token", env = "AUTH_TOKEN", hide_env_values = true)]
    pub(crate) _auth_token: Option<String>,

    /// The main command to execute.
    #[structopt(subcommand)]
    pub(crate) cmd: Command,
}

#[derive(Debug, StructOpt)]
pub(crate) enum Command {
    /// Initializes the connection to the database and executes migrations.
    Init,

    /// Handles user authentication.
    Auth,

    /// Manages appointment interactions.
    Appointment(Appointment),
}


#[derive(Debug, StructOpt)]
pub(crate) struct Appointment {
    /// Specifies the specific appointment action to perform.
    #[structopt(subcommand)]
    pub(crate) appt_command: AppointmentCommand,
}

#[derive(Debug, StructOpt)]
pub(crate) enum AppointmentCommand {
    /// Creates a new appointment.
    Create,

    /// Deletes a specified appointment.
    Delete {
        /// IDs of the appointments to delete.
        #[structopt(short)]
        uuids: Vec<Uuid>
    },

    /// Generates QR codes.
    Generate {
        /// ID of the appointment for which generate QR codes.
        #[structopt(short)]
        appt_id: Uuid,

        /// Number of QR codes to generate.
        #[structopt(short)]
        count: Option<i32>,
    },

    /// Sends emails containing QR codes for a specified appointment.
    Send {
        /// ID of the appointment for which to send QR codes.
        #[structopt(short)]
        appt_id: Uuid,

        /// Email addresses to send QR codes to.
        #[structopt(short)]
        email: Vec<Email>
    },
}