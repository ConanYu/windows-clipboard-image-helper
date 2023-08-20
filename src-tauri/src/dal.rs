use std::borrow::Cow;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use anyhow::{bail, Result};
use arboard::ImageData;
use image::EncodableLayout;
use log::{error, warn};
use once_cell::sync::Lazy;
use crate::ocr::ocr;
use crate::settings::get_root;

pub mod clean;
pub mod client;
pub mod model;

static LOCK: Lazy<Mutex<()>> = Lazy::new(|| {
    Mutex::new(())
});

static CACHE_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let root = get_root();
    root.join("cache.png")
});

fn save_image_inner(data: ImageData) -> Result<()> {
    let image;
    let ocr_result;
    {
        let lock = LOCK.lock();
        if let Err(err) = lock {
            bail!("save image lock failed, err: {}", err);
        }
        image::save_buffer(
            CACHE_PATH.as_path(),
            data.bytes.as_ref(),
            data.width as u32,
            data.height as u32,
            image::ColorType::Rgba8,
        )?;
        image = fs::read(CACHE_PATH.as_path())?;
        match ocr(CACHE_PATH.as_path()) {
            Ok(r) => {
                ocr_result = Some(r);
            }
            Err(err) => {
                ocr_result = None;
                warn!("ocr failed, err: {}", err);
            }
        }
    }
    let sum = sha256::digest(image.as_slice());
    model::insert_image(&image, &(data.width.clone() as i32), &(data.height.clone() as i32), &ocr_result, &sum)?;
    Ok(())
}

pub fn save_image(data: ImageData) {
    let result = save_image_inner(data);
    if let Err(err) = result {
        error!("save image error: {}", err);
    }
}

pub fn upload_image(image_path: &Vec<String>) -> Result<()> {
    let mut img = vec![];
    for path in image_path {
        let data = fs::read(path)?;
        img.push(image::load_from_memory(data.as_slice())?);
    }
    for img in img {
        let img = img.into_rgba8();
        let img = ImageData {
            width: img.width() as usize,
            height: img.height() as usize,
            bytes: Cow::Borrowed(img.as_bytes()),
        };
        save_image_inner(img)?;
    }
    Ok(())
}