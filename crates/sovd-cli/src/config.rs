use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Client configuration, loaded from config file or environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub server_url: String,
    pub auth_token: Option<String>,
    pub plugin_dirs: Vec<String>,
    pub output_format: String,
    pub json_log: bool,
    pub timeout_seconds: u64,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:8080".into(),
            auth_token: None,
            plugin_dirs: vec![],
            output_format: "text".into(),
            json_log: false,
            timeout_seconds: 30,
        }
    }
}

impl ClientConfig {
    /// Load config from the default location (~/.config/sovd-flash/config.toml).
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(config) => return config,
                    Err(e) => {
                        eprintln!("Warning: Failed to parse config file: {e}");
                    }
                },
                Err(e) => {
                    eprintln!("Warning: Failed to read config file: {e}");
                }
            }
        }
        Self::default()
    }

    /// Get the config file path.
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("sovd-flash")
            .join("config.toml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = ClientConfig::default();
        assert_eq!(cfg.server_url, "http://localhost:8080");
        assert!(cfg.auth_token.is_none());
        assert!(cfg.plugin_dirs.is_empty());
        assert_eq!(cfg.output_format, "text");
        assert!(!cfg.json_log);
        assert_eq!(cfg.timeout_seconds, 30);
    }

    #[test]
    fn config_serialization_roundtrip() {
        let cfg = ClientConfig {
            server_url: "http://my-server:9090".into(),
            auth_token: Some("secret".into()),
            plugin_dirs: vec!["/opt/plugins".into()],
            output_format: "json".into(),
            json_log: true,
            timeout_seconds: 60,
        };
        let toml_str = toml::to_string(&cfg).unwrap();
        let deserialized: ClientConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(deserialized.server_url, "http://my-server:9090");
        assert_eq!(deserialized.auth_token, Some("secret".into()));
        assert_eq!(deserialized.plugin_dirs, vec!["/opt/plugins"]);
        assert_eq!(deserialized.output_format, "json");
        assert!(deserialized.json_log);
        assert_eq!(deserialized.timeout_seconds, 60);
    }

    #[test]
    fn config_path_ends_with_config_toml() {
        let path = ClientConfig::config_path();
        assert!(path.ends_with("sovd-flash/config.toml"));
    }

    #[test]
    fn load_returns_default_when_no_file() {
        let cfg = ClientConfig::load();
        assert_eq!(cfg.server_url, "http://localhost:8080");
    }

    #[test]
    fn config_json_serialization() {
        let cfg = ClientConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let deserialized: ClientConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.server_url, cfg.server_url);
        assert_eq!(deserialized.timeout_seconds, cfg.timeout_seconds);
    }

    #[test]
    fn config_clone() {
        let cfg = ClientConfig::default();
        let cloned = cfg.clone();
        assert_eq!(cloned.server_url, cfg.server_url);
        assert_eq!(cloned.timeout_seconds, cfg.timeout_seconds);
    }
}
