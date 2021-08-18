use std::{io::Error, net::TcpListener};

use actix_web::{
    dev::Server,
    web::{self, Data},
    App, HttpServer,
};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing_actix_web::TracingLogger;

use crate::{
    configuration::{Configurations, DatabaseConfigurations},
    email_client::EmailClient,
    routes::{confirm, health_check, subscribe},
};

// App type that exposes the required data
pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Configurations) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database)
            .await
            .expect("Failed to connect to Postgres.");
        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address.");
        let email_client = EmailClient::new(
            &configuration.email_client.base_url,
            sender_email,
            &configuration.email_client.authorization_token,
        );
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            connection_pool,
            email_client,
            configuration.application.base_url,
        )?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    // Only return (either () or Error) when the app is stopped
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub async fn get_connection_pool(
    configurations: &DatabaseConfigurations,
) -> Result<PgPool, sqlx::Error> {
    // sqlx::PgPool is built around sqlx::PgConnection to handle multiple concurrent
    // queries through a connection pool
    PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_with(configurations.with_db())
        .await
}

// We need to define a wrapper type in order to retrieve the URL
// in the `subscribe`handler.
// Retrieval from the context, in actix-web, is type-based: using
// a raw `String`would expose us to conflicts.
pub struct ApplicationBaseUrl(pub String);

/// Create a [Server] and return [Result] to be handled by main().
///
/// This approach allows us to write an integration testing that could create and kill
/// an instance of the app as a _background task_. Otherwise, the test will run the
/// server but it'll never stop running.
///
/// Also, it receives [TcpListener] as parameter to be able to run tests in random
/// ports without conflicts. To run the app in tests, a listener will define the address
/// where the app will be running with a random available port. Importantly, it'll provide
/// a way to retrieve the selected port to perform the actual validation.
pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
) -> Result<Server, Error> {
    // actix-web's runtime model spin up a worker process for each available core
    // on the machine. Each worker runs its own copy of the app. Because of this,
    // HttpServer::new expect a cloneable instance of connection, so we need
    // to wrap it in an Arc in an Arc smart pointer. In this case, however, we're
    // using web::Data, which boils down to an Arc.
    let db_pool = web::Data::new(db_pool);

    let email_client = web::Data::new(email_client);
    let base_url = Data::new(ApplicationBaseUrl(base_url));

    // HttpServer handles all "transport level" concerns.
    // First, establishes a connection with a client of the API. Then, an App
    // is created to handling all the application logic (routing, middlewares,
    // request handlers, etc). App takes a request as input and spit out a
    // response. App implements the "builder pattern". This allows us to chain
    // method calls one after the other to add features to the same App instance.
    let server = HttpServer::new(move || {
        App::new()
            // wrap() allows us to pass middlewares. TracingLogger is a
            // tracing-based logger (as a replacement for log-based middlewares::Logger).
            // This is required to easily add a request_id and other useful information
            // to the logs
            .wrap(TracingLogger)
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            // Register the connection pool as part of the application state
            // (later on accessible through actix_web::web::Data extractor
            // inside every route). We can use .data() and app_data(). The former
            // would add another Arc pointer on top of the existing one.
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
