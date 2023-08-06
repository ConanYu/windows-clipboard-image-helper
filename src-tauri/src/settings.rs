use std::{env, fs};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

static ROOT: Lazy<Box<Path>> = Lazy::new(|| {
    let root = env::current_dir().unwrap();
    let root = root.join(".windows-clipboard-image-helper").into_boxed_path();
    if !root.is_dir() {
        fs::create_dir(root.clone()).unwrap();
    }
    root
});

pub fn get_root() -> &'static Path {
    ROOT.deref()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseLimitType {
    MB,
    NUM,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub auto_start: bool,
    pub database_limit_type: DatabaseLimitType,
    pub database_limit: i64,
}

fn get_settings_path() -> PathBuf {
    let root = ROOT.deref().deref();
    root.join("settings.json")
}

static SETTINGS: Lazy<RwLock<Settings>> = Lazy::new(|| {
    let path = get_settings_path();
    let settings = if path.exists() {
        let file = fs::read(path.as_path()).unwrap();
        let settings = String::from_utf8(file).unwrap();
        let settings = serde_json::from_str(settings.as_str()).unwrap();
        settings
    } else {
        let settings = Settings {
            auto_start: false,
            database_limit_type: DatabaseLimitType::MB,
            database_limit: 1024,
        };
        fs::write(path.as_path(), serde_json::to_string(&settings).unwrap().as_bytes()).unwrap();
        settings
    };
    RwLock::new(settings)
});

pub fn get_settings() -> Settings {
    SETTINGS.read().unwrap().clone()
}

pub fn set_settings(settings: Settings) {
    let path = get_settings_path();
    let mut writer = SETTINGS.write().unwrap();
    fs::write(path.as_path(), serde_json::to_string(&settings).unwrap().as_bytes()).unwrap();
    *writer = settings;
}