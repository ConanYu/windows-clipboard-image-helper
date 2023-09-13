use std::{env, fs};
use std::ffi::{OsStr, OsString};
use std::ops::{Deref, Not};
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::ptr::null_mut;
use std::sync::RwLock;
use anyhow::{bail, Result};
use log::error;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use winapi::shared::minwindef::{DWORD, HKEY};
use winapi::um::winnt::REG_SZ;
use winapi::um::winreg::{HKEY_CURRENT_USER, RegDeleteValueW, RegOpenKeyW, RegQueryValueExW, RegSetValueExW};
use src_macro::Updater;
use crate::common::get_root;

unsafe fn set_auto_start(auto_start: bool) -> Result<()> {
    // DEBUG模式下无法运行
    if cfg!(debug_assertions) {
        return Ok(());
    }
    let mut hkey: HKEY = null_mut();
    // 找到开机启动注册表所在位置
    let status = RegOpenKeyW(
        HKEY_CURRENT_USER,
        OsStr::new(r"Software\Microsoft\Windows\CurrentVersion\Run").encode_wide().chain(std::iter::once(0)).collect::<Vec<_>>().as_ptr(),
        &mut hkey,
    );
    if status != 0 {
        bail!("regedit key of auto start not found");
    }
    // 查找已经设置的注册表值
    let mut dtype: DWORD = 0;
    let mut dword: DWORD = 0;
    let lp = OsStr::new("WindowsClipboardImageHelper").encode_wide().chain(std::iter::once(0)).collect::<Vec<_>>();
    let status = RegQueryValueExW(
        hkey,
        lp.as_ptr(),
        null_mut(),
        &mut dtype,
        null_mut(),
        &mut dword,
    );
    let mut value = OsString::new();
    if status == 0 {
        let mut data: Vec<u8> = vec![0; dword as usize];
        let status = RegQueryValueExW(
            hkey,
            lp.as_ptr(),
            null_mut(),
            &mut dtype,
            data.as_mut_ptr(),
            &mut dword,
        );
        if status != 0 {
            bail!("query regedit with unknown status: {}", status);
        }
        value = OsString::from(String::from_utf8(data)?);
    }
    // 设置状态和原本状态一致 直接返回
    let exe = env::current_exe()?;
    let exe = exe.as_os_str();
    if auto_start.not() && status != 0 || auto_start && value.as_os_str() == exe {
        return Ok(());
    }
    if auto_start {
        // 开机自启 设置值为当前运行程序
        let mut exe = exe.to_os_string();
        exe.push(" --auto_start");
        let value = exe.encode_wide().chain(std::iter::once(0)).collect::<Vec<_>>();
        let status = RegSetValueExW(
            hkey,
            lp.as_ptr(),
            0,
            REG_SZ,
            value.as_ptr() as *const u8,
            (value.len() * 2) as DWORD,
        );
        if status != 0 {
            bail!("set regedit with unknown status: {}", status);
        }
    } else {
        // 取消设置开机自启 删除值
        let status = RegDeleteValueW(
            hkey,
            lp.as_ptr(),
        );
        if status != 0 {
            bail!("delete regedit with unknown status: {}", status);
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseLimitType {
    MB,
    NUM,
}

#[derive(Debug, Clone, Serialize, Deserialize, Updater)]
pub struct Settings {
    pub auto_start: Option<bool>,
    pub database_limit_type: Option<DatabaseLimitType>,
    pub database_limit: Option<i64>,
    pub ocr_feature: Option<bool>,
}

fn get_settings_path() -> PathBuf {
    let root = get_root();
    root.join("settings.json")
}

static SETTINGS: Lazy<RwLock<Settings>> = Lazy::new(|| {
    let path = get_settings_path();
    let settings = if path.exists() {
        let file = fs::read(path.as_path()).unwrap();
        let settings = String::from_utf8(file).unwrap();
        let mut settings: Settings = serde_json::from_str(settings.as_str()).unwrap();
        if let Some(auto_start) = settings.auto_start {
            if auto_start {
                unsafe {
                    let result = set_auto_start(true);
                    if let Err(err) = result {
                        error!("set auto start failed with err: {}", err.to_string());
                        settings.auto_start = Some(false);
                    }
                }
            }
        }
        settings
    } else {
        let settings = Settings {
            auto_start: Some(false),
            database_limit_type: Some(DatabaseLimitType::MB),
            database_limit: Some(1024),
            ocr_feature: Some(false),
        };
        fs::write(path.as_path(), serde_json::to_string(&settings).unwrap().as_bytes()).unwrap();
        settings
    };
    RwLock::new(settings)
});

pub fn get_settings() -> Settings {
    SETTINGS.read().unwrap().clone()
}

pub fn set_settings(settings: Settings) -> Result<()> {
    let path = get_settings_path();
    let mut object = SETTINGS.write().unwrap();
    if let Some(auto_start) = &settings.auto_start {
        unsafe { set_auto_start(auto_start.clone())?; }
    }
    object.update(settings);
    fs::write(path.as_path(), serde_json::to_string(object.deref()).unwrap().as_bytes())?;
    Ok(())
}