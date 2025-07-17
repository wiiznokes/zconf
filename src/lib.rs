use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::bail;
use atomicwrites::{AtomicFile, OverwriteBehavior::AllowOverwrite};
use log::error;
use serde::{Serialize, de::DeserializeOwned};

#[cfg(test)]
mod test;

#[derive(Debug)]
pub struct ConfigManager<S> {
    path: PathBuf,
    data: S,
}

impl<S> ConfigManager<S> {
    pub fn new<P: AsRef<Path>>(path: P) -> ConfigManager<S>
    where
        S: Default + DeserializeOwned + Serialize,
    {
        let path = path.as_ref();

        let data = if !path.exists() {
            S::default()
        } else {
            match deserialize(path) {
                Ok(settings) => settings,
                Err(e) => {
                    error!("can't deserialize settings {e}");
                    S::default()
                }
            }
        };

        ConfigManager {
            path: path.to_path_buf(),
            data,
        }
    }

    pub fn data(&self) -> &S {
        &self.data
    }

    pub fn update(&mut self, f: impl FnOnce(&mut S))
    where
        S: Serialize,
    {
        f(&mut self.data);

        if let Err(e) = serialize(&self.path, &self.data) {
            error!("{e}");
        }
    }

    pub fn update_without_write(&mut self, f: impl FnOnce(&mut S)) {
        f(&mut self.data);
    }

    pub fn reload(&mut self) -> anyhow::Result<()>
    where
        S: DeserializeOwned,
    {
        self.data = deserialize(&self.path)?;
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn change_path(&mut self, new_path: impl Into<PathBuf>)
    where
        S: Serialize,
    {
        self.path = new_path.into();

        if let Err(e) = serialize(&self.path, &self.data) {
            error!("{e}");
        }
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

    match path.parent() {
        Some(parent) => {
            fs::create_dir_all(parent)?;
        }
        None => bail!("no parent"),
    }

    let af = AtomicFile::new(path, AllowOverwrite);
    af.write(|f| f.write_all(str.as_bytes()))?;

    Ok(())
}
