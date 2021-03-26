// `actix_rt::test` is the testing equivalent of `actix_web::main`
#[actix_rt::test]
async fn health_check_works() {
    // Arrange
    spawn_app();
    // Create a HTTP client to perform calls to the endpoints under testing
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get("http://127.0.0.1:8080/health_check")
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

// Launch application in the background
fn spawn_app() {
    let server = zero2prod::run().expect("Failed to bind address.");

    // Launch the server as a background task. tokio::spawn returns a handle to the
    // spawned future (althought we have no use for it here)
    tokio::spawn(server);
}
