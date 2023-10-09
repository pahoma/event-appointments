mod appointment;
mod validation;
mod invitation;

use crate::error::CustomError;
use actix_web::{HttpResponse, Result, web};
use actix_web::web::Data;
use sqlx::{PgPool, Postgres, Transaction};
use anyhow::Context;
use uuid::Uuid;

pub use appointment::*;
pub use validation::*;
pub use invitation::*;


pub(crate) async fn open_transaction(pool: Data<PgPool>) -> Result<Transaction<'static, Postgres>, CustomError> {
    let tx= pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;
    Ok(tx)
}

pub(crate) async fn commit_transaction(transaction: Transaction<'_, Postgres>, msg: &'static str) -> Result<(), CustomError> {
    transaction
        .commit()
        .await
        .context(msg)?;
    Ok(())
}


/// Asynchronously reads and returns the content of a specified HTML template.
///
/// Given the name of a template (without the `.html` extension), this function
/// will search for the template in the `./../templates/` directory, read its
/// content, and return it as a `String`.
///
/// # Arguments
///
/// * `template_name` - A string slice that holds the name of the template
///   (without the `.html` extension).
///
/// # Returns
///
/// * `Ok(String)` - The content of the template as a string.
/// * `Err(tokio::io::Error)` - An error occurred while trying to read the template.
///
/// # Examples
///
/// ```no_run
/// use web_server::routes::render_template;
/// use tokio::runtime;
///
/// let rt = runtime::Runtime::new().unwrap();
/// let _ = rt.block_on(async {
///     let content = render_template("email").await?;
///     Ok::<(), tokio::io::Error>(())
/// });
/// ```
///
/// # Note
///
/// This function expects the templates to be located in `./../templates/` relative
/// to the binary's location and assumes a `.html` file extension for the templates.
pub async fn render_template(template_name: &str) -> Result<String, tokio::io::Error> {
    tokio::fs::read_to_string(
        format!("./../templates/{}.html",
                template_name
        )
    ).await
}

