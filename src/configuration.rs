use std::{
    convert::{TryFrom, TryInto},
    env::current_dir,
};

use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::PgConnectOptions;

use crate::domain::SubscriberEmail;

/// Struct that models our app-level configurations.
///
/// We have two grous of configuration to handle: `actix-web` server
/// configurations (e. g., port) and database connection parameters.
/// The `config` crate requires a struct.
#[derive(serde::Deserialize)]
pub struct Configurations {
    pub database: DatabaseConfigurations,
    pub application: ApplicationConfigurations,
    pub email_client: EmailClientConfigurations,
}

/// Configurable portion of the running application address.
///
/// We need a _hierarchical configuration_ to make portions of the application
/// address configurable depending on the environment. The project has a _base_
/// configuration file that keeps the values shared accross local and production
/// environment. Then, it has a collection of environment-specific files specifying
/// values for fields that require customisation. Finally, the configurations
/// depends on an environment variables, APP_ENVIRONMENT to determine the running
/// environment.
#[derive(serde::Deserialize)]
pub struct ApplicationConfigurations {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

#[derive(serde::Deserialize)]
pub struct DatabaseConfigurations {
    pub username: String,
    pub password: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseConfigurations {
    /// Generate options to connect to a Postgres database.
    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.database_name)
    }

    /// Generate options to connect to a Postgres instance, not a specific logical database.
    ///
    /// This function is useful to create isolated connections when running integration tests.
    /// The connection will allow to create a database to run migrations and perform test
    /// queries in individual test without being undeterministic.
    pub fn without_db(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password)
            .port(self.port)
    }
}

#[derive(serde::Deserialize)]
pub struct EmailClientConfigurations {
    pub base_url: String,
    pub sender_email: String,
    pub authorization_token: String,
}

impl EmailClientConfigurations {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }
}

/// The possible runtime environment for our application.
pub enum Environment {
    Local,
    Produduction,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Produduction => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Produduction),
            other => Err(format!(
                "{} is not a supported environment. Use either 'local' or 'production'.",
                other
            )),
        }
    }
}

/// Read configurations from top-level file
pub fn get_configurations() -> Result<Configurations, config::ConfigError> {
    // Initialize configuration reader
    let mut configurations = config::Config::default();

    // Compose the "default" configurations path
    let base_path = current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configurations");

    // Read the "default" configuration file
    configurations
        .merge(config::File::from(configuration_directory.join("base")).required(true))?;

    // Detect the running environment.
    // Default to local if unspecified
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");

    // Layer on the environment-specific values
    configurations.merge(
        config::File::from(configuration_directory.join(environment.as_str())).required(true),
    )?;

    // Add in configurations from environment variables (with a prefix of APP and "__" as
    // separator). E. g., "APP_APPLICATION__PORT=5001" would set Configurations.application.port
    configurations.merge(config::Environment::with_prefix("app").separator("__"))?;

    // Try to convert the configuration values it read into
    // our Configurations type
    configurations.try_into()
}
