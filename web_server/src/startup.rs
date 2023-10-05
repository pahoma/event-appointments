use crate::routes::{appointment_routes, invitation_routes, validation_routes};
use crate::error::CustomError;
use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer, Result};
use std::net::TcpListener;
use sqlx::{Pool, Postgres};
use tracing_actix_web::TracingLogger;
use shared::email_client::EmailClient;
use shared::qr_client::QRClient;

pub fn run(listener: TcpListener, pool: Pool<Postgres>, qrclient: QRClient, email_client: EmailClient ) -> Result<Server, CustomError> {
    let address_msg = format!("Server started on {:?}", &listener.local_addr()?);
    let db_pool = web::Data::new(pool);
    let qr_client = web::Data::new(qrclient);
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/",
                   web::get().to(|| async { HttpResponse::Ok().body("/") })
            )
            .service(web::scope("/api")
                .configure(appointment_routes)
                .configure(validation_routes)
                .configure(invitation_routes)
            )
            .app_data(db_pool.clone())
            .app_data(qr_client.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    println!("{}", address_msg);
    Ok(server)
}