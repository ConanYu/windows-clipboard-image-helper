use std::ops::Not;
use anyhow::Result;
use log::error;
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use tauri::{AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, Window, Wry};
use crate::client::sqlite::client;
use crate::{clipboard, settings};
use crate::analyzer::ocr;
use crate::model::Image;
use crate::settings::Settings;

pub mod image_insert;
pub mod image_search;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorFilter {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    // 覆盖比例 范围：[0,1]
    pub cover_ratio_from: f64,
    pub cover_ratio_to: f64,
    // 可接受的DeltaE 范围：[0,100]
    pub difference: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetImageRequest {
    // 上一次返回图片中最小的mtime
    pub mtime: Option<i64>,
    // 返回图片的最大数量
    pub limit: Option<i64>,
    pub id: Option<Vec<i64>>,
    pub text: Option<Vec<String>>,
    pub date_range_from: Option<i64>,
    pub date_range_to: Option<i64>,
    pub color_filter: Option<ColorFilter>,
}

#[tauri::command(rename_all = "snake_case")]
async fn get_image(request: GetImageRequest) -> Result<Vec<Image>, String> {
    match image_search::get_image(request).await {
        Ok(img) => {
            let mut resp = vec![];
            for img in img {
                resp.push(Image {
                    id: img.id,
                    image: img.image.to_base64(),
                    ocr: img.ocr,
                    size: img.size,
                    width: img.width,
                    height: img.height,
                    ctime: img.ctime,
                    mtime: img.mtime,
                    sum: img.sum,
                });
            }
            Ok(resp)
        }
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command(rename_all = "snake_case")]
async fn re_copy(image_id: i32) -> Result<(), String> {
    conv_result(clipboard::re_copy(image_id))
}

// 通过ID删除图片
#[tauri::command(rename_all = "snake_case")]
async fn delete_image(image_id: Vec<i32>) -> Result<(), String> {
    let inner = || -> Result<()> {
        let client = client();
        let image_id: Vec<String> = image_id.iter().map(|v| v.to_string()).collect();
        let sql = format!(r#"DELETE FROM image WHERE id IN ({})"#, image_id.join(", "));
        client.execute(sql.as_str(), ())?;
        Ok(())
    };
    conv_result(inner())
}

// 上传图片
#[tauri::command(rename_all = "snake_case")]
async fn upload_image(image_path: Vec<String>) -> Result<(), String> {
    tokio::spawn(async move {
        if let Err(err) = image_insert::upload_image(&image_path).await {
            error!("upload image with error: {}", err.to_string());
        }
    });
    Ok(())
}

// 关闭窗口
#[tauri::command(rename_all = "snake_case")]
async fn close_window(window: Window) -> Result<(), String> {
    conv_result(window.hide())
}

// OCR状态 可用的情况下返回大于100的值 不可用的情况下返回百分比
#[tauri::command(rename_all = "snake_case")]
pub async fn ocr_status() -> Result<f64, String> {
    conv_result(ocr::status().await)
}

// OCR 开始下载
#[tauri::command(rename_all = "snake_case")]
pub async fn ocr_prepare() -> Result<(), String> {
    tokio::spawn(async {
        if let Err(e) = ocr::prepare().await {
            error!("ocr prepare error: {:?}", e);
        }
    });
    Ok(())
}

// OCR 取消下载
#[tauri::command(rename_all = "snake_case")]
pub async fn ocr_pause_prepare() -> Result<(), String> {
    conv_result(ocr::pause_prepare().await)
}

static ESCAPE_BLUR: Lazy<std::sync::Mutex<bool>> = Lazy::new(|| {
    false.into()
});

// 免疫以及取消免疫失焦
#[tauri::command(rename_all = "snake_case")]
pub fn escape_blur(escape: bool) -> Result<(), String> {
    *ESCAPE_BLUR.lock().unwrap() = escape;
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_escape_blur() -> Result<bool, String> {
    Ok(ESCAPE_BLUR.lock().unwrap().clone())
}


pub async fn run() -> Result<()> {
    let app = tauri::Builder::default()
        // 应用系统托盘配置
        .system_tray((|| {
            let quit = CustomMenuItem::new("quit".to_string(), "退出");
            let show = CustomMenuItem::new("show".to_string(), "显示");
            let tray_menu = SystemTrayMenu::new()
                .add_item(show)
                .add_native_item(SystemTrayMenuItem::Separator)
                .add_item(quit);
            SystemTray::new().with_menu(tray_menu).with_tooltip("Windows剪切板图片工具")
        })())
        // 应用系统托盘事件
        .on_system_tray_event(|app_handle: &AppHandle<Wry>, event| {
            match event {
                SystemTrayEvent::DoubleClick { .. } => {
                    let window = app_handle.get_window("main").unwrap();
                    if !window.is_visible().unwrap() {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                }
                SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    "show" => {
                        let window = app_handle.get_window("main").unwrap();
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                    _ => {}
                },
                _ => {}
            }
        })
        // 前端命令
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            get_image,
            re_copy,
            delete_image,
            close_window,
            upload_image,
            ocr_status,
            ocr_prepare,
            ocr_pause_prepare,
            escape_blur,
            get_escape_blur,
        ])
        // APP开始时初始化
        .setup(|app| {
            let window = app.get_window("main").unwrap();
            #[cfg(debug_assertions)]
            {
                window.open_devtools();
                window.close_devtools();
            }
            // 直接隐藏后台
            window.hide().unwrap();
            Ok(())
        })
        // APP事件管理
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
            }
            tauri::WindowEvent::Focused(focus) => {
                let escape_blur = ESCAPE_BLUR.lock().unwrap();
                if escape_blur.not() && focus.not() {
                    event.window().hide().unwrap();
                }
            }
            _ => {}
        }).build(tauri::generate_context!()).unwrap();

    // 运行APP
    app.run(|_app_handler, _event| {});
    Ok(())
}