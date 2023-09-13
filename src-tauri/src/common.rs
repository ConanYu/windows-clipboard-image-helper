use std::{env, fs};
use std::ops::Deref;
use std::path::Path;
use once_cell::sync::Lazy;

static ROOT: Lazy<Box<Path>> = Lazy::new(|| {
    let mut root;
    if cfg!(debug_assertions) {
        root = env::current_dir().unwrap();
    } else {
        root = env::current_exe().unwrap();
        root.pop();
    }
    let root = root.join(".windows-clipboard-image-helper").into_boxed_path();
    if !root.is_dir() {
        fs::create_dir(root.clone()).unwrap();
    }
    root
});

pub fn get_root() -> &'static Path {
    ROOT.deref()
}