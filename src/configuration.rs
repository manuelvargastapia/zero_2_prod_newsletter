use std::{
    convert::{TryFrom, TryInto},
    env::current_dir,
};

/// Struct that models our app-level configurations.
///
/// We have two grous of configuration to handle: `actix-web` server
/// configurations (e. g., port) and database connection parameters.
/// The `config` crate requires a struct.
#[derive(serde::Deserialize)]
pub struct Configurations {
    pub database: DatabaseSettings,
    pub application: ApplicationConfigurations,
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
    pub port: u16,
    pub host: String,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    /// Compose a string with connection params to connect to DB with `sqlx::PgConnection::connect`
    pub fn generate_connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }

    /// Compose a string to connect to a Postgres instance, not a specific logical database.
    ///
    /// This function is useful to create isolated connections when running integration tests.
    /// The connection will allow to create a database to run migrations and perform test
    /// queries in individual test without being undeterministic.
    pub fn generate_connection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
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
