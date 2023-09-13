use std::borrow::Cow;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use anyhow::{bail, Result};
use arboard::ImageData;
use image::EncodableLayout;
use log::error;
use once_cell::sync::Lazy;
use rusqlite::named_params;
use crate::client::sqlite::client;
use crate::common::get_root;

// 数据库中插入图片
pub fn insert_image(image: &Vec<u8>, width: &i32, height: &i32, sum: &String) -> Result<()> {
    let client = client();
    // 校验是否是上一次插入的图片
    let mut stmt = client.prepare("SELECT id, sum FROM image ORDER BY mtime DESC LIMIT 1")?;
    let mut rows = stmt.query(named_params! {})?;
    let mut last_sum = "-".to_string();
    let mut id = 0;
    while let Some(row) = rows.next()? {
        id = row.get(0)?;
        last_sum = row.get(1)?;
    }
    let now = chrono::Local::now().timestamp_millis();
    if &last_sum == sum {
        // 设置修改时间
        client.execute(r#"UPDATE image SET mtime = ?2 WHERE id = ?1"#, (&id, &now))?;
        return Ok(());
    }
    // 正式开始插入图片
    let size = image.len() as i64;
    client.execute(r#"INSERT INTO image (image, size, width, height, ctime, mtime, sum)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
                   (image, &size, width, height, &now, &now, sum))?;
    Ok(())
}

static LOCK: Lazy<Mutex<()>> = Lazy::new(||{
    Mutex::new(())
});

static CACHE_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let root = get_root();
    root.join("cache.1.png")
});

fn save_image_inner(data: ImageData) -> Result<()> {
    let image;
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
    }
    let sum = sha256::digest(image.as_slice());
    insert_image(&image, &(data.width.clone() as i32), &(data.height.clone() as i32), &sum)?;
    Ok(())
}

// 上传图片
pub async fn upload_image(image_path: &Vec<String>) -> Result<()> {
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

// 保存图片
pub fn save_image(data: ImageData) {
    let result = save_image_inner(data);
    if let Err(err) = result {
        error!("save image error: {}", err);
    }
}