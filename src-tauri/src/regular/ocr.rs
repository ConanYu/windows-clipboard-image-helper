use std::ops::Deref;
use std::path::PathBuf;
use anyhow::Result;
use log::error;
use once_cell::sync::Lazy;
use rusqlite::named_params;
use tokio::sync::Mutex;
use crate::analyzer::ocr::{analyze, status};
use crate::client::sqlite::client;
use crate::common::get_root;
use crate::model::OCR;
use crate::settings;

static LOCK: Lazy<Mutex<()>> = Lazy::new(|| {
    Mutex::new(())
});

static CACHE_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let root = get_root();
    root.join("cache.2.png")
});

fn get_one_without_ocr() -> Result<(i32, Vec<u8>)> {
    let c = client();
    let mut id = -1;
    let mut image: Vec<u8> = vec![];
    let mut stmt = c.prepare("SELECT id, image FROM image WHERE ocr IS NULL LIMIT 1")?;
    let mut rows = stmt.query(named_params! {})?;
    while let Some(row) = rows.next()? {
        id = row.get(0)?;
        image = row.get(1)?;
    }
    return Ok((id, image));
}

fn update_ocr(id: &i32, ocr: &OCR) -> Result<()> {
    let c = client();
    c.execute("UPDATE image SET ocr = ?2 WHERE id = ?1", (&id, &serde_json::to_string(&ocr)?))?;
    Ok(())
}

async fn ocr_inner() -> Result<()> {
    // OCR未就绪
    if status().await? <= 100.0 {
        return Ok(());
    }
    let (id, image) = get_one_without_ocr()?;
    if id == -1 {
        return Ok(());
    }
    let r;
    {
        let _lock = LOCK.lock().await;
        std::fs::write(CACHE_PATH.deref(), image.as_slice())?;
        r = analyze(CACHE_PATH.deref()).await?;
    }
    update_ocr(&id, &r)?;
    Ok(())
}

pub async fn ocr() {
    loop {
        let settings = settings::get_settings();
        if settings.ocr_feature.is_some_and(|x| x) {
            if let Err(err) = ocr_inner().await {
                error!("regular ocr with error: {}", err.to_string());
            }
        }
        // 每10秒检查有没有图片没有进行过OCR并对其进行OCR检查
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
}