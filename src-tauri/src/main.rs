// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;

pub mod analyzer;
pub mod app;
pub mod bundle;
pub mod client;
pub mod clipboard;
pub mod common;
pub mod initialize;
pub mod model;
pub mod regular;
pub mod settings;

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: 这个程序不能多开 要进行一个多开检测

    let _init = initialize::init_logger().unwrap();
    initialize::init_database().unwrap();
    clipboard::listen(app::image_insert::save_image);
    tokio::spawn(regular::clean::clean());
    tokio::spawn(regular::ocr::ocr());

    app::run().await?;
    Ok(())
}
