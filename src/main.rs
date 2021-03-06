use std::net::TcpListener;

use sqlx::postgres::PgPoolOptions;

use zero2prod::{
    configuration::get_configurations,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

// #[actix_web::main] is a procedural macro that allow running async code
// in main(). After expand it with cargo-expand, we can see that indeed
// the main() code passed to the compiler after #[actix_web::main] is
// synchronous. We are starting actix’s async runtime (actix_web::rt) and we
// are using it to drive the future returned by HttpServer::run to completion.
// In other words, the job of #[actix_web::main] is to give us the illusion of
// being able to define an asynchronous main while, under the hood, it just
// takes our main asynchronous body and writes the necessary boilerplate to
// make it run on top of actix’s runtime.
#[actix_web::main]
/// The only job of main() is try to call run() depending on its [Result] (Ok or Error).
async fn main() -> std::io::Result<()> {
    // Setting to log the structured logs generated by the tracing crate's Span.
    let subscriber = get_subscriber("zero2prod".into(), "info".into());
    init_subscriber(subscriber);

    // Load configurations from file before launching the server
    let configurations = get_configurations().expect("Failed to read configuration file.");

    // sqlx::PgPool is built around sqlx::PgConnection to handle multiple concurrent
    // queries through a connection pool
    let connection_pool = PgPoolOptions::new().connect_lazy_with(configurations.database.with_db());

    let address = format!(
        "{}:{}",
        configurations.application.host, configurations.application.port
    );
    let listener = TcpListener::bind(address)?;

    run(listener, connection_pool)?.await?;
    Ok(())
}
