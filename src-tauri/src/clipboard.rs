use std::borrow::Cow;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr::{null, null_mut};
use anyhow::{bail, Result};
use arboard::{Clipboard, ImageData};
use image::EncodableLayout;
use rusqlite::named_params;
use winapi::shared::windef::HWND;
use winapi::um::winuser::{AddClipboardFormatListener, CreateWindowExW, GetMessageW, HWND_MESSAGE, MSG, WM_CLIPBOARDUPDATE};
use crate::dal::client::client;

pub fn re_copy(image_id: i32) -> Result<()> {
    let client = client();
    let mut stmt = client.prepare("SELECT image FROM image WHERE id = :image_id")?;
    let mut rows = stmt.query(named_params! {
        ":image_id": image_id,
    })?;
    let mut image: Vec<u8> = vec![];
    while let Some(row) = rows.next()? {
        image = row.get(0)?;
    }
    if image.len() == 0 {
        bail!("no such image id: {}", image_id);
    }
    let image = image::load_from_memory(image.as_slice())?;
    let image = image.into_rgba8();
    let image = ImageData {
        width: image.width() as usize,
        height: image.height() as usize,
        bytes: Cow::Borrowed(image.as_bytes()),
    };
    let mut clipboard = Clipboard::new().unwrap();
    clipboard.set_image(image)?;
    Ok(())
}

pub fn listen<F: Fn(ImageData) + Send + 'static>(callback: F) {
    std::thread::spawn(move || {
        unsafe {
            for msg in Message::new() {
                match msg.message {
                    WM_CLIPBOARDUPDATE => {
                        let mut clipboard = Clipboard::new().unwrap();
                        let image = clipboard.get_image();
                        match image {
                            Ok(image) => callback(image),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    });
}

struct Message {
    hwnd: HWND,
}

impl Message {
    pub unsafe fn new() -> Self {
        let hwnd = CreateWindowExW(
            0,
            OsStr::new("STATIC").encode_wide().chain(std::iter::once(0)).collect::<Vec<_>>().as_ptr(),
            null(),
            0,
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            null_mut(),
            null_mut(),
            null_mut(),
        );
        if hwnd == null_mut() {
            panic!("CreateWindowEx failed");
        }
        AddClipboardFormatListener(hwnd);
        Self { hwnd }
    }

    unsafe fn get(&self) -> Option<MSG> {
        let mut msg = std::mem::zeroed();
        let ret = GetMessageW(&mut msg, self.hwnd, 0, 0);
        if ret == 1 {
            Some(msg)
        } else {
            None
        }
    }
}

impl Iterator for Message {
    type Item = MSG;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe { self.get() }
    }
}