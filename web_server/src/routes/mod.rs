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

const EMAIL_TEMPLATE_PATH: &str = "./../templates/email.html";
const QR_TEMPLATE_PATH: &str = "./../templates/qr.html";
pub async fn render_template(path: &str) -> Result<String, tokio::io::Error> {
    tokio::fs::read_to_string(path).await
}

