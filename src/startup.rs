use std::{io::Error, net::TcpListener};

use actix_web::{dev::Server, web, App, HttpServer};

use crate::routes::{health_check, subscribe};

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
pub fn run(listener: TcpListener) -> Result<Server, Error> {
    // HttpServer handles all "transport level" concerns.
    // First, establishes a connection with a client of the API. Then, an App
    // is created to handling all the application logic (routing, middlewares,
    // request handlers, etc). App takes a request as input and spit out a
    // response. App implements the "builder pattern". This allows us to chain
    // method calls one after the other to add features to the same App instance.
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
