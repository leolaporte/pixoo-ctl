use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub devices: HashMap<String, DeviceConfig>,
}

#[derive(Debug, Deserialize)]
pub struct DeviceConfig {
    pub ip: String,
}

fn config_path() -> PathBuf {
    dirs().join("config.toml")
}

fn dirs() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".config").join("pixoo-ctl")
}

pub fn load() -> Result<Config> {
    let path = config_path();
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config at {}", path.display()))?;
    let config: Config =
        toml::from_str(&content).context("Failed to parse config.toml")?;
    Ok(config)
}

pub fn resolve_devices(config: &Config, name: &str) -> Result<Vec<(String, String)>> {
    if name == "all" {
        let mut devices: Vec<_> = config
            .devices
            .iter()
            .map(|(k, v)| (k.clone(), v.ip.clone()))
            .collect();
        devices.sort_by(|a, b| a.0.cmp(&b.0));
        return Ok(devices);
    }

    match config.devices.get(name) {
        Some(dev) => Ok(vec![(name.to_string(), dev.ip.clone())]),
        None => {
            let available: Vec<_> = config.devices.keys().map(|k| k.as_str()).collect();
            bail!(
                "Unknown device '{}'. Available devices: {}",
                name,
                available.join(", ")
            );
        }
    }
}
