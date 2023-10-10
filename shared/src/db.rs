use std::time::Duration;
use sqlx::{Error, PgPool, postgres::PgPoolOptions};
use sqlx::postgres::PgConnectOptions;

pub fn init_db(options: PgConnectOptions) -> Result<PgPool, Error> {
    let db: PgPool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .max_connections(20)
        .connect_lazy_with(options);

    Ok(db)
}

#[allow(unused)]
pub async fn migrate( db_pool: &PgPool) -> Result<(), Error> {
    sqlx::migrate!("./../migrations").run(db_pool).await?;

    Ok(())
}