use std::ffi::OsString;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;
use anyhow::{bail, Result};
use winapi::um::winbase::CREATE_NO_WINDOW;
use crate::dal::model::OCR;
use crate::settings::get_root;

pub fn ocr(path: &Path) -> Result<OCR> {
    let root = get_root();
    let root = root.join("PaddleOCR-json_v.1.3.0");
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