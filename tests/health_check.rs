use std::net::TcpListener;

// `actix_rt::test` is the testing equivalent of `actix_web::main`
#[actix_rt::test]
async fn health_check_works() {
    // Arrange
    let address = spawn_app();
    // Create a HTTP client to perform calls to the endpoints under testing
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

// Launch application in the background
fn spawn_app() -> String {
    // A port = 0 means that the SO will automatically scan for a random available port
    // to run the server. This allows us to avoid conflicts and run multiples tests
    // concurrently. A TcpListener is used to define the address and then return it to
    // be used by the HTTP client performing the call.
    let listener =
        TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");

    let port = listener.local_addr().unwrap().port();

    let server = zero2prod::run(listener).expect("Failed to bind address.");

    // Launch the server as a background task. tokio::spawn returns a handle to the
    // spawned future (althought we have no use for it here)
    tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
