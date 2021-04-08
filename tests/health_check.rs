use std::net::TcpListener;

use sqlx::PgPool;

use zero2prod::configuration::get_configurations;
use zero2prod::startup::run;

// `actix_rt::test` is the testing equivalent of `actix_web::main`
#[actix_rt::test]
async fn health_check_works() {
    // Arrange
    let test_app = spawn_app().await;
    // Create a HTTP client to perform calls to the endpoints under testing
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", &test_app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

// Launch application in the background
async fn spawn_app() -> TestApp {
    // A port = 0 means that the SO will automatically scan for a random available port
    // to run the server. This allows us to avoid conflicts and run multiples tests
    // concurrently. A TcpListener is used to define the address and then return it to
    // be used by the HTTP client performing the call.
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");

    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let configurations = get_configurations().expect("Failed to read configurations.");
    let connection_pool = PgPool::connect(&configurations.database.generate_connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    let server = run(listener, connection_pool.clone()).expect("Failed to bind address.");

    // Launch the server as a background task. tokio::spawn returns a handle to the
    // spawned future (althought we have no use for it here)
    tokio::spawn(server);

    TestApp {
        address,
        db_pool: connection_pool,
    }
}

#[actix_rt::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=nicolas%20bourbaki&email=nick_bourbaki%40gmail.com";

    // Act
    let response = client
        .post(&format!("{}/subscriptions", &test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    // Verify

    // This macro needs a DATABASE_URL env var defined, so we need to
    // add a .env at top-level to compile this code. Note that it's
    // pushed to repo, because we need it to run the CI pipeline
    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "nick_bourbaki@gmail.com");
    assert_eq!(saved.name, "nicolas bourbaki");
}

// This is an example of table-driven test (aka parametrised test). It is particularly
// helpful when dealing with bad inputs - instead of duplicating test logic several
// times we can simply run the same assertion against a collection of known invalid
// bodies that we expect to fail in the same way
#[actix_rt::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=nicolas%20bourbaki", "missing the email"),
        ("email=nick_bourbaki%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
