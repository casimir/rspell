use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::SpellError;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SourceConfig {
    pub aff: String,
    pub dic: String,
}

#[derive(Deserialize)]
pub struct DictionariesConfig {
    pub directories: Vec<String>,
    pub sources: HashMap<String, SourceConfig>,
}

#[derive(Deserialize)]
pub struct Config {
    pub dictionaries: DictionariesConfig,
}

fn ensure_config_file(path: &PathBuf) -> io::Result<bool> {
    let create = !path.exists();
    if create {
        log::info!("no config file, creating one at '{}'", path.display());
        fs::create_dir_all(&path.parent().unwrap())?;
        fs::write(&path, include_str!("../files/config.toml"))?;
    }
    Ok(create)
}

/// Loads the configuration from the disk.
///
/// If the config file is absent it will be created with default values.
pub fn load_config() -> Result<Config, SpellError> {
    let path = crate::dirs().config_dir().join("config.toml");
    log::debug!("config file: {}", path.display());
    ensure_config_file(&path).map_err(SpellError::InitConfigError)?;
    let raw = fs::read_to_string(&path).map_err(SpellError::ReadConfigError)?;
    toml::from_str(&raw).map_err(SpellError::LoadConfigError)
}
