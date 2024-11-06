use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::RwLock;
use config::{ConfigurationBuilder, DefaultConfigurationBuilder};
use config::ext::{ConfigurationBinder, JsonConfigurationExtensions};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref CONFIG: RwLock<Config> = RwLock::new(Config::default());
}

macro_rules! define {
    ($name:ident { $($field:tt)* }) => {
        #[derive(Clone, Debug, Serialize, Deserialize)]
        #[serde(rename_all(serialize = "camelCase", deserialize = "PascalCase"))]
        pub struct $name {
            $($field)*
        }
    };
}

/// Initializes the configuration.
/// Returns a copy of the configuration.
pub fn init_config() -> anyhow::Result<Config> {
    // Check if the configuration file exists.
    if !Path::new("config.json").exists() {
        // Create the configuration file.
        let config = Config::default();

        // Write the configuration to the file.
        let mut file = File::create("config.json")?;
        file.write_all(serde_json::to_string_pretty(&config)?.as_bytes())?;

        return Ok(config);
    }

    // Parse the configuration.
    let config: Config = DefaultConfigurationBuilder::new()
        .add_json_file("config.json")
        .build()
        .unwrap()
        .reify();

    // Copy the config to the global variable.
    if let Ok(mut write) = CONFIG.write() {
        *write = config.clone();
    }

    Ok(config)
}

define!(Config {
    // The name of the device/window.
    pub device_name: String,

    // Set the window size.
    pub screen_width: i32,
    pub screen_height: i32,

    // Set the window's position.
    pub window_x: i32,
    pub window_y: i32,

    // The path to the dictionary file.
    pub dictionary: String,

    // The path to the letters folder.
    pub font: String,
    
    // The server configuration.
    pub server_address: String,
    pub server_port: u16
});

impl Default for Config {
    fn default() -> Self {
        Config {
            device_name: "iPhone".to_string(),
            screen_width: 523,
            screen_height: 1135,
            window_x: 0,
            window_y: 0,
            dictionary: "words.txt".to_string(),
            font: "images".to_string(),
            server_address: "127.0.0.1".to_string(),
            server_port: 5000
        }
    }
}
