use crate::error::BackyError;
use std::fs;

use serde::{Deserialize, Serialize};

/// Representa as configurações do usuário antes da manipulação e transformação em Config.
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    /// Lista dos arquivos que devem ser armazenados pelo backup.
    pub files: Vec<String>,
    /// Local do disco onde devem ser armazenados os backups incrementais.
    pub archive_path: String,
}

pub fn load() -> Result<Config, BackyError> {
    let mut config_path = match dirs::config_dir() {
        Some(d) => d,
        None => return Err(BackyError::NoConfigDir),
    };
    config_path.push("backy/config.toml");
    let config_file = match fs::read_to_string(config_path) {
        Ok(f) => f,
        _ => return Err(BackyError::NoConfig),
    };
    match toml::from_str(&config_file) {
        Ok(c) => Ok(c),
        Err(err) => Err(BackyError::BadConfigFormat(err)),
    }
}
