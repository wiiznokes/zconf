use std::{
    fs,
    io::Write,
    marker::PhantomData,
    path::{Path, PathBuf},
};

use anyhow::bail;
use atomicwrites::{AtomicFile, OverwriteBehavior::AllowOverwrite};
use serde::{Serialize, de::DeserializeOwned};

#[allow(unused_imports)]
#[macro_use]
extern crate log;

#[cfg(test)]
mod test;

/// Trait for serialization and deserialization of configuration data.
/// This trait allows the `ConfigManager` to work with different serialization formats like JSON or TOML.
pub trait SerdeAdapter<S> {
    fn serialize(data: &S) -> anyhow::Result<Box<[u8]>>
    where
        S: Serialize;
    fn deserialize(data: &[u8]) -> anyhow::Result<S>
    where
        S: DeserializeOwned;
}

#[derive(Debug)]
pub struct ConfigManager<S, SA> {
    path: PathBuf,
    data: S,
    #[cfg(feature = "watcher")]
    watcher: Option<notify::RecommendedWatcher>,
    _sa: PhantomData<SA>,
}

impl<S, SA> ConfigManager<S, SA>
where
    S: Serialize + DeserializeOwned,
    SA: SerdeAdapter<S>,
{
    /// Create a new [`ConfigManager`] with the provided path. If the config can't be deserialized,
    /// the [`Default`] implementation will be used.
    pub fn new(path: impl Into<PathBuf>) -> ConfigManager<S, SA>
    where
        S: Default,
    {
        Self::inner_new(path.into(), Box::new(|| S::default()))
    }

    /// Create a new [`ConfigManager`] with the provided path. If the config can't be deserialized,
    /// f will be used to create it.
    pub fn with_fallback<F>(path: impl Into<PathBuf>, f: F) -> ConfigManager<S, SA>
    where
        F: FnOnce() -> S + 'static,
    {
        Self::inner_new(path.into(), Box::new(f))
    }

    fn inner_new(path: PathBuf, default: Box<dyn FnOnce() -> S>) -> ConfigManager<S, SA> {
        let data = if !path.exists() {
            default()
        } else {
            match Self::deserialize(&path) {
                Ok(settings) => settings,
                Err(e) => {
                    error!("can't deserialize settings {e}");
                    default()
                }
            }
        };

        ConfigManager {
            path: path.to_path_buf(),
            data,
            #[cfg(feature = "watcher")]
            watcher: None,
            _sa: PhantomData,
        }
    }

    /// Get the path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the inner config
    pub fn data(&self) -> &S {
        &self.data
    }

    /// Update the config. This function will write to the filesystem.
    pub fn update(&mut self, f: impl FnOnce(&mut S)) {
        f(&mut self.data);

        if let Err(e) = Self::serialize(&self.path, &self.data) {
            error!("{e}");
        }
    }

    /// Same as [`Self::update`], but the error is returned.
    pub fn update_with_err(&mut self, f: impl FnOnce(&mut S)) -> anyhow::Result<()> {
        f(&mut self.data);
        Self::serialize(&self.path, &self.data)?;
        Ok(())
    }

    /// Update the date without writting to the filesystem.
    pub fn update_without_write(&mut self, f: impl FnOnce(&mut S)) {
        f(&mut self.data);
    }

    /// Deserialize the config and change the inner data.
    pub fn reload(&mut self) {
        match Self::deserialize(&self.path) {
            Ok(data) => self.data = data,
            Err(e) => error!("file: {}: {e}", self.path.display()),
        }
    }

    /// Same as [`Self::reload`], but the error is returned.
    pub fn reload_with_err(&mut self) -> anyhow::Result<()> {
        self.data = Self::deserialize(&self.path)?;
        Ok(())
    }

    fn deserialize(path: &Path) -> anyhow::Result<S> {
        let data = fs::read(path)?;

        let t = SA::deserialize(&data)?;

        Ok(t)
    }

    fn serialize(path: &Path, rust_struct: &S) -> anyhow::Result<()> {
        let data = SA::serialize(rust_struct)?;

        match path.parent() {
            Some(parent) => {
                fs::create_dir_all(parent)?;
            }
            None => bail!("no parent"),
        }

        let af = AtomicFile::new(path, AllowOverwrite);
        af.write(|f| f.write_all(&data))?;

        Ok(())
    }

    /// Watch this config path for change. The callback will be called on each change.
    #[cfg(feature = "watcher")]
    pub fn watch<F>(&mut self, mut callback: F) -> anyhow::Result<()>
    where
        F: FnMut() + Send + 'static,
    {
        use notify::{EventKind, Watcher, event::ModifyKind};

        let watch_path = if !self.path.exists() {
            match self.path.parent() {
                Some(parent) => {
                    fs::create_dir_all(parent)?;
                    parent
                }
                None => bail!("no parent"),
            }
        } else {
            &self.path
        };

        // todo: debouncer ?
        let mut watcher =
            notify::recommended_watcher(move |event_res: Result<notify::Event, notify::Error>| {
                match event_res {
                    Ok(event) => {
                        match &event.kind {
                            // Data not mutated
                            EventKind::Access(_) | EventKind::Modify(ModifyKind::Metadata(_)) => {
                                return;
                            }
                            _ => {}
                        }

                        callback();
                    }
                    Err(e) => {
                        error!("watch event error: {e}");
                    }
                }
            })?;

        watcher.watch(watch_path, notify::RecursiveMode::NonRecursive)?;

        self.watcher = Some(watcher);

        Ok(())
    }
}

#[cfg(feature = "json")]
pub struct Json;

#[cfg(feature = "json")]
impl<S: Serialize + DeserializeOwned> SerdeAdapter<S> for Json {
    fn serialize(data: &S) -> anyhow::Result<Box<[u8]>> {
        Ok(serde_json::to_string_pretty(data)?
            .into_bytes()
            .into_boxed_slice())
    }

    fn deserialize(data: &[u8]) -> anyhow::Result<S> {
        Ok(serde_json::from_slice(data)?)
    }
}

#[cfg(feature = "toml")]
pub struct Toml;

#[cfg(feature = "toml")]
impl<S: Serialize + DeserializeOwned> SerdeAdapter<S> for Toml {
    fn serialize(data: &S) -> anyhow::Result<Box<[u8]>> {
        Ok(toml::to_string_pretty(data)?
            .into_bytes()
            .into_boxed_slice())
    }

    fn deserialize(data: &[u8]) -> anyhow::Result<S> {
        Ok(toml::from_slice(data)?)
    }
}
