use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use time::OffsetDateTime;

/// Represents the application's persistent state.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PersistState {
    /// Timestamp of the last email notification sent to user.
    #[serde(with = "time::serde::iso8601::option")]
    pub last_notif: Option<OffsetDateTime>,
}

impl PersistState {
    /// Loads the persistent state from the specified path.
    ///
    /// Returns the default state if the file is not found, or if parsing fails.
    /// Panics on other I/O errors.
    pub fn load(path: &Path) -> Self {
        match fs::read_to_string(path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(state) => {
                    rocket::info!("Loaded persistence state from {}", path.display());
                    state
                }
                Err(err) => {
                    rocket::warn!(
                        "Failed to parse persistence file at {}: {}. Using default state.",
                        path.display(),
                        err
                    );
                    Self::default()
                }
            },
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                rocket::info!(
                    "Persistence file not found at {}, using default state.",
                    path.display()
                );
                Self::default()
            }
            Err(err) => {
                panic!(
                    "Critical error reading persistence file at {}: {}",
                    path.display(),
                    err
                );
            }
        }
    }

    /// Saves the persistent state to the specified path.
    pub fn save(&self, path: &Path) {
        match toml::to_string_pretty(self) {
            Ok(content) => {
                if let Err(err) = fs::write(path, content) {
                    rocket::warn!(
                        "Error writing persistence file to {}: {}",
                        path.display(),
                        err
                    );
                }
            }
            Err(err) => {
                rocket::warn!("Error serializing persistence state: {}", err);
            }
        }
    }
}
