use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::Write;
use std::ops::Deref;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::sync::RwLock;
use anyhow::{bail, Result};
use futures_util::StreamExt;
use log::info;
use once_cell::sync::Lazy;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex;
use winapi::um::winbase::CREATE_NO_WINDOW;
use crate::common::get_root;
use crate::bundle::ensure_seven_zip;
use crate::model::OCR;

static OCR_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let root = get_root();
    root.join("PaddleOCR-json_v.1.3.0")
});

static CACHE_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let root = get_root();
    root.join(".PaddleOCR-json_v.1.3.0.7z.cache")
});

const TOTAL_SIZE: i32 = 100940020;

static FILE_SIZE: Lazy<Mutex<i32>> = Lazy::new(|| {
    let downloaded_size = match std::fs::metadata(CACHE_PATH.as_path()) {
        Ok(data) => data.len() as i32,
        Err(_) => 0,
    };
    Mutex::new(downloaded_size)
});

static DOWNLOADING: Lazy<Mutex<()>> = Lazy::new(|| {
    Mutex::new(())
});

pub async fn status() -> Result<f64> {
    // 已经好了 返回大于100的数
    if ready()? {
        return Ok(111.1);
    }
    let percentage = *FILE_SIZE.lock().await as f64 / TOTAL_SIZE as f64 * 100.0;
    // 不在下载中 返回负数
    if DOWNLOADING.try_lock().is_ok() {
        return Ok(percentage - 111.1);
    }
    // 正在下载中 返回大于等于零的数
    return Ok(percentage);
}

static DOWNLOAD_PAUSE_CHANNEL: Lazy<(Sender<()>, Mutex<Receiver<()>>)> = Lazy::new(|| {
    let (tx, rx) = channel(32);
    (tx, rx.into())
});

async fn download() -> Result<()> {
    if ready()? {
        return Ok(());
    }
    let downloaded_size = FILE_SIZE.lock().await.clone();
    if downloaded_size < TOTAL_SIZE {
        let client = reqwest::ClientBuilder::new().build()?;
        const URL: &str = "https://ghproxy.com/https://github.com/hiroi-sora/PaddleOCR-json/releases/download/v1.3.0/PaddleOCR-json_v.1.3.0.7z";
        let mut request = reqwest::Request::new(reqwest::Method::GET, reqwest::Url::from_str(URL)?);
        if downloaded_size > 0 {
            request.headers_mut().insert("Range", format!("bytes={}-{}", downloaded_size, TOTAL_SIZE).as_str().parse()?);
        }
        *FILE_SIZE.lock().await = downloaded_size as i32;
        let response = client.execute(request).await?;
        let mut stream = response.bytes_stream();
        let mut file = OpenOptions::new();
        file.write(true).append(true);
        if downloaded_size == 0 {
            file.create_new(true);
        }
        let mut file = file.open(CACHE_PATH.as_path())?;
        while let Some(item) = stream.next().await {
            let item = item?;
            file.write(item.as_ref())?;
            let mut file_size = FILE_SIZE.lock().await;
            *file_size = file_size.clone() + item.len() as i32;
            if DOWNLOAD_PAUSE_CHANNEL.1.lock().await.try_recv().is_ok() {
                return Ok(());
            }
        }
    }
    if let Err(err) = std::fs::remove_dir(OCR_PATH.deref()) {
        info!("remove dir, with message: {}", err.to_string());
    }
    let sz = ensure_seven_zip();
    let mut arg = std::ffi::OsString::from("-o");
    arg.push(get_root().as_os_str());
    let r = Command::new(sz.as_os_str())
        .creation_flags(CREATE_NO_WINDOW) // 不显示窗口
        .args([std::ffi::OsStr::new("x"), arg.as_os_str(), CACHE_PATH.as_os_str()])
        .output()?;
    if r.status.code().unwrap() != 0 {
        let e = String::from_utf8(r.stderr)?;
        bail!("{}", e);
    }
    return Ok(());
}

fn check_ready() -> Result<bool> {
    if DOWNLOADING.try_lock().is_err() {
        return Ok(false);
    }
    let is_dir = match std::fs::metadata(OCR_PATH.deref().as_path()) {
        Ok(data) => data.is_dir(),
        Err(_) => false,
    };
    if is_dir {
        let sz = ensure_seven_zip();
        let r = Command::new(sz.as_os_str())
            .creation_flags(CREATE_NO_WINDOW) // 不显示窗口
            .args([std::ffi::OsStr::new("h"), OCR_PATH.deref().as_os_str()])
            .output()?;
        let s = String::from_utf8(r.stdout)?;
        let s = s.split("\n");
        for l in s {
            if l.find("CRC32  for data and names").is_some() && l.find("170E28C3-00000029").is_some() {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

static READY: Lazy<RwLock<bool>> = Lazy::new(|| {
    RwLock::new(check_ready().unwrap())
});

fn ready() -> Result<bool> {
    if *READY.read().unwrap() {
        return Ok(true);
    }
    if check_ready()? {
        *READY.write().unwrap() = true;
        return Ok(true);
    }
    return Ok(false);
}

pub async fn prepare() -> Result<()> {
    if let Err(_) = DOWNLOADING.try_lock() {
        return Ok(());
    }
    if ready()? {
        return Ok(());
    }
    if let Ok(_) = DOWNLOADING.try_lock() {
        {
            let mut recv = DOWNLOAD_PAUSE_CHANNEL.1.lock().await;
            while recv.try_recv().is_ok() {}
        }
        download().await?;
    }
    Ok(())
}

pub async fn pause_prepare() -> Result<()> {
    Ok(DOWNLOAD_PAUSE_CHANNEL.0.try_send(())?)
}

pub async fn analyze(path: &Path) -> Result<OCR> {
    let root = OCR_PATH.deref();
    let exe = root.join("PaddleOCR-json.exe");
    let mut arg = OsString::new();
    arg.push(r#"-image_path=""#);
    arg.push(path.as_os_str());
    arg.push(r#"""#);
    let output = Command::new(exe.as_os_str())
        .creation_flags(CREATE_NO_WINDOW) // 不显示窗口
        .current_dir(root.as_path())
        .raw_arg(arg)
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;
    let split = stdout.split("\r\n");
    for split in split {
        if split.len() > 0 {
            if let Ok(result) = serde_json::from_str(split) {
                return Ok(result);
            }
        }
    }
    bail!(stdout);
}
