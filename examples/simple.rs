use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use zconf::ConfigManager;

#[derive(Debug, Serialize, Deserialize, Default)]
struct Settings {
    name: String,
    value: i32,
}

fn main() {
    let path = PathBuf::from("settings.toml");

    let mut config_manager = ConfigManager::<Settings, zconf::Toml>::new(&path);

    config_manager.update(|settings| {
        settings.name = "Example".to_string();
        settings.value = 42;
    });

    println!("Current settings: {:?}", config_manager.data());
}
