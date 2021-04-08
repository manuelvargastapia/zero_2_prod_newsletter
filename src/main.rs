use std::net::TcpListener;

use zero2prod::{configuration::get_configurations, startup::run};

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
    // Load configurations from file before launching the server
    let configurations = get_configurations().expect("Failed to read configuration file.");
    let address = format!("127.0.0.1:{}", configurations.application_port);

    let listener = TcpListener::bind(address).expect("Failed to bind random port");
    run(listener)?.await
}
