# zconf

[![crates.io](https://img.shields.io/crates/v/zconf?style=flat-square&logo=rust)](https://crates.io/crates/zconf)
[![docs.rs](https://img.shields.io/badge/docs.rs-zconf-blue?style=flat-square&logo=docs.rs)](https://docs.rs/zconf)
[![license](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](#license)

Crate to manage configuration files

## Usage

The basic idea is to provide a safe abscraction arround updating and accessing your settings. You should not be allowed to change the config struct without writting on the filesystem (unless you want to).

```rs
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
```

In general, you will want to use a crate like [directories](https://crates.io/crates/directories) to get the path of the system config directory.

## Goal / Vision

- No macros (not that macros are always wrong, but they provide poor ide support and longer compile time)
- config are contained in one file
- simple (for example, an api that log error directly)

## Feature

- Atomic file operation, using [atomicwrites](https://crates.io/crates/atomicwrites)
- Watcher, using [notify](https://crates.io/crates/notify), see the [libcosmic example](./examples/libcosmic/)
- Various format support (`toml` and `json` are integrated with feature flag, `toml` being the default one. Uou can also provide a custom format.)
