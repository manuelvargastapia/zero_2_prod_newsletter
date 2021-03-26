use actix_web::{web, App, HttpResponse, HttpServer, Responder};

/// Endpoint to  verify the application es up and ready.
///
/// Returns **200 OK** with no body.
///
/// It can be used to customize some alert system to get noitified when
/// the API is down. Or trigger a restart in the context of container
/// orchestration when the API has become unresponsive.
async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

// #[actix_web::main] is a procedurla macro that allow running async code
// in main(). After expand it with cargo-expand, we can see that indeed
// the main() code passed to the compiler after #[actix_web::main] is
// synchronous. We are starting actix’s async runtime (actix_web::rt) and we
// are using it to drive the future returned by HttpServer::run to completion.
// In other words, the job of #[actix_web::main] is to give us the illusion of
// being able to define an asynchronous main while, under the hood, it just
// takes our main asynchronous body and writes the necessary boilerplate to
// make it run on top of actix’s runtime.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // HttpServer handles all "transport level" concerns.
    // First, establishes a connection with a client of the API. Then, an App
    // is created to handling all the application logic (routing, middlewares,
    // request handlers, etc). App takes a request as input and spit out a
    // response. App implements the "builder pattern". This allows us to chain
    // method calls one after the other to add features to the same App instance.
    HttpServer::new(|| {
        App::new().route("/health_check", web::get().to(health_check))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
