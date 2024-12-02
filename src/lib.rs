use std::{
    fs,
    path::{Path, PathBuf},
};

use log::error;
use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug)]
pub struct ConfigManager<S> {
    path: PathBuf,
    settings: S,
}

impl<S> ConfigManager<S> {
    pub fn new<P: AsRef<Path>>(path: P) -> ConfigManager<S>
    where
        S: Default + DeserializeOwned + Serialize,
    {
        let path = path.as_ref();

        let settings = if !path.exists() {
            S::default()
        } else {
            match deserialize(&path) {
                Ok(settings) => settings,
                Err(e) => {
                    error!("can't deserialize settings {e}");
                    S::default()
                }
            }
        };

        ConfigManager {
            path: path.to_path_buf(),
            settings,
        }
    }

    pub fn settings(&self) -> &S {
        &self.settings
    }

    pub fn update(&mut self, mut f: impl FnMut(&mut S))
    where
        S: Serialize,
    {
        f(&mut self.settings);

        if let Err(e) = serialize(&self.path, &self.settings) {
            error!("{e}");
        }
    }

    pub fn reload(&mut self) -> anyhow::Result<()>
    where
        S: DeserializeOwned,
    {
        self.settings = deserialize(&self.path)?;
        Ok(())
    }
}

fn deserialize<T: DeserializeOwned>(path: &Path) -> anyhow::Result<T> {
    let str = fs::read_to_string(path)?;

    #[cfg(feature = "toml")]
    let t = toml::from_str(&str)?;

    #[cfg(feature = "json")]
    let t = json::from_str(&str)?;

    Ok(t)
}

fn serialize<T: Serialize>(path: &Path, rust_struct: &T) -> anyhow::Result<()> {
    #[cfg(feature = "toml")]
    let str = toml::to_string_pretty(rust_struct)?;

    #[cfg(feature = "json")]
    let str = json::to_string_pretty(rust_struct)?;

    fs::write(path, str)?;
    Ok(())
}
