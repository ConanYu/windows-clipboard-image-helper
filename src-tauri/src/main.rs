// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::RwLock;
use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose::STANDARD_NO_PAD;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, Window, Wry};
use crate::clipboard::listen;
use crate::dal::clean::regular_cleaning;
use crate::dal::model::{GetImageRequest, OCR};
use crate::dal::save_image;
use crate::settings::{Settings, CloseWindowType};

pub mod clipboard;
pub mod dal;
pub mod initialize;
pub mod ocr;
pub mod settings;

fn conv_result<T: Serialize, E: ToString>(r: Result<T, E>) -> Result<T, String> {
    match r {
        Ok(r) => Ok(r),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command(rename_all = "snake_case")]
fn get_settings() -> Settings {
    settings::get_settings()
}

#[tauri::command(rename_all = "snake_case")]
fn set_settings(settings: Settings) -> Result<(), String> {
    conv_result(settings::set_settings(settings))
}

#[derive(Debug, Serialize, Deserialize)]
struct ImageShow {
    pub id: i64,
    pub image: String,
    pub ocr: Option<OCR>,
    pub width: i32,
    pub height: i32,
    pub ctime: i64,
    pub mtime: i64,
}

#[tauri::command(rename_all = "snake_case")]
fn get_image(request: GetImageRequest) -> Result<Vec<ImageShow>, String> {
    match dal::model::get_image(request) {
        Ok(image) => {
            let mut resp = vec![];
            for image in image {
                resp.push(ImageShow {
                    id: image.id,
                    image: STANDARD_NO_PAD.encode(image.image),
                    ocr: image.ocr,
                    width: image.width,
                    height: image.height,
                    ctime: image.ctime,
                    mtime: image.mtime,
                });
            }
            Ok(resp)
        }
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command(rename_all = "snake_case")]
fn re_copy(image_id: i32) -> Result<(), String> {
    conv_result(clipboard::re_copy(image_id))
}

#[tauri::command(rename_all = "snake_case")]
fn delete_image(image_id: i32) -> Result<(), String> {
    conv_result(dal::model::delete_image(image_id))
}

#[tauri::command(rename_all = "snake_case")]
fn close_window<T>(window: Window<T>, force: bool, remember: bool) -> Result<(), String>
where T: tauri_runtime::Runtime<tauri::EventLoopMessage> {
    if remember {
        let r = settings::set_settings(Settings {
            auto_start: None,
            database_limit_type: None,
            database_limit: None,
            close_window_type: Some(match force {
                true => CloseWindowType::EXIT,
                false => CloseWindowType::BACKGROUND,
            }),
        });
        if let Err(err) = r {
            return Err(err.to_string());
        }
    }
    if force {
        std::process::exit(0);
    }
    conv_result(window.hide())
}

#[tauri::command(rename_all = "snake_case")]
fn upload_image(image_path: Vec<String>) -> Result<(), String> {
    conv_result(dal::upload_image(&image_path))
}

fn gen_tray() -> SystemTray {
    let quit = CustomMenuItem::new("quit".to_string(), "退出");
    let show = CustomMenuItem::new("show".to_string(), "显示");
    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);
    SystemTray::new().with_menu(tray_menu).with_tooltip("Windows剪切板图片工具")
}

static IDENTIFIER: Lazy<RwLock<String>> = Lazy::new(|| {
    "".to_string().into()
});

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: 这个程序不能多开 要进行一个多开检测

    initialize::init_logger()?;
    initialize::init_database()?;
    listen(save_image);
    tokio::spawn(regular_cleaning());

    let on_system_tray_event = |app_handle: &AppHandle<Wry>, event| {
        match event {
            SystemTrayEvent::DoubleClick { .. } => {
                let window = app_handle.get_window("main").unwrap();
                if !window.is_visible().unwrap() {
                    window.show().unwrap();
                }
            }
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    std::process::exit(0);
                }
                "show" => {
                    let window = app_handle.get_window("main").unwrap();
                    window.show().unwrap();
                }
                _ => {}
            },
            _ => {}
        }
    };

    let app = tauri::Builder::default()
        .system_tray(gen_tray())
        .on_system_tray_event(on_system_tray_event)
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            get_image,
            re_copy,
            delete_image,
            close_window,
            upload_image,
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }
            let _ = app;
            Ok(())
        })
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
            }
            _ => {}
        }).build(tauri::generate_context!())?;

    *IDENTIFIER.write().unwrap() = app.config().tauri.bundle.identifier.clone();

    app.run(|_app_handler, _event| {});

    Ok(())
}
