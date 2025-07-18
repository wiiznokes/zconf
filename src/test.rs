use serde::{Deserialize, Serialize};

use crate::{ConfigManager, Toml};

#[derive(Debug, Serialize, Deserialize, Default)]
struct Config {
    active: bool,
    name: String,
}

#[test]
fn write() {
    let mut config: ConfigManager<Config, Toml> = ConfigManager::new("test/test1.toml");

    config.update(|c| {
        c.active = true;
    });
}

#[test]
fn read() {
    let config: ConfigManager<Config, Toml> = ConfigManager::new("test_static/test1.toml");

    assert!(config.data().active);
    assert_eq!(config.data().name, "test_read");
}
