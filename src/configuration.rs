/// Struct that models our app-level configurations.
///
/// We have two grous of configuration to handle: `actix-web` server
/// configurations (e. g., port) and database connection parameters.
/// The `config` crate requires a struct.
#[derive(serde::Deserialize)]
pub struct Configurations {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

/// Read configurations from top-level file
pub fn get_configurations() -> Result<Configurations, config::ConfigError> {
    // Initialize configuration reader
    let mut configurations = config::Config::default();

    // Add configuration values from a file named `configuration`.
    // It will look for any top-level file with an extension
    // that `config` knows how to parse: yaml, json, etc.
    configurations.merge(config::File::with_name("configurations"))?;

    // Try to convert the configuration values it read into
    // our Configurations type
    configurations.try_into()
}
