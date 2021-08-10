use lazy_static::lazy_static;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

use zero2prod::configuration::{get_configurations, DatabaseConfigurations};
use zero2prod::startup::{get_connection_pool, Application};
use zero2prod::telemetry::{get_subscriber, init_subscriber};

// Ensure that the tracing stack is only initialised once.
lazy_static! {
    static ref TRACING: () = {
        // If TEST_LOG is set, pick all the spans that are at least debug-level,
        // otherwise, we drop everithing by passing an empty filter.
        let filter = if std::env::var("TEST_LOG").is_ok() {
            "debug"
        } else {
            ""
        };
        let subscriber = get_subscriber("test".into(), filter.into());
        init_subscriber(subscriber);
    };
}

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

// Launch application in the background
pub async fn spawn_app() -> TestApp {
    // Set up tracing stack.
    // The first time initialize is invoked the code in TRACING is executed.
    // All other invocations will instead skip execution.
    lazy_static::initialize(&TRACING);

    // Randomise configurations to ensure test isolation
    let configurations = {
        let mut c = get_configurations().expect("Failed to read configurations.");
        // Use a different database for each test case
        c.database.database_name = Uuid::new_v4().to_string();
        // Use a random OS port
        c.application.port = 0;
        c
    };

    // Create and migrate the database
    configure_database(&configurations.database).await;

    // Build the app
    let application = Application::build(configurations.clone())
        .await
        .expect("Failed to build application.");

    let address = format!("http://127.0.0.1:{}", application.port());

    // Launch the server as a background task. tokio::spawn returns a handle to the
    // spawned future (althought we have no use for it here)
    tokio::spawn(application.run_until_stopped());

    TestApp {
        address,
        db_pool: get_connection_pool(&configurations.database)
            .await
            .expect("Failed to connect to the database"),
    }
}

// Before each test we
// (i) create a new logical database with a unique name and
// (ii) run database migration on it.
//
// This is required to avoid using the same database connection for
// all the test. That is, we need to isolate the test to be able able
// to run it in a determistic way.
async fn configure_database(config: &DatabaseConfigurations) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations") // Same macro used by sqlx-cli when executing sqlx migrate run
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database.");

    connection_pool
}
