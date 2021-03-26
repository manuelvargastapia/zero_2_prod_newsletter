use std::io::Error;

use actix_web::{dev::Server, web, App, HttpResponse, HttpServer, Responder};

/// Endpoint to  verify the application es up and ready.
///
/// **Returns 200 OK with no body**
///
/// It can be used to customize some alert system to get noitified when
/// the API is down. Or trigger a restart in the context of container
/// orchestration when the API has become unresponsive.
async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

/// Create a [Server] and return [Result] to be handled by main().
///
/// This approach allows us to write an integration testing that could create and kill
/// an instance of the app as a _background task_. Otherwise, the test will run the
/// server but it'll never stop running.
pub fn run() -> Result<Server, Error> {
    // HttpServer handles all "transport level" concerns.
    // First, establishes a connection with a client of the API. Then, an App
    // is created to handling all the application logic (routing, middlewares,
    // request handlers, etc). App takes a request as input and spit out a
    // response. App implements the "builder pattern". This allows us to chain
    // method calls one after the other to add features to the same App instance.
    let server = HttpServer::new(|| {
        App::new().route("/health_check", web::get().to(health_check))
    })
    .bind(("127.0.0.1", 8080))?
    .run();

    Ok(server)
}
