use std::ops::Deref;
use once_cell::sync::Lazy;
use rusqlite::Connection;
use crate::common::get_root;

static PATH: Lazy<String> = Lazy::new(|| {
    let root = get_root();
    let binding = root.join("database.sqlite3");
    binding.to_str().unwrap().to_string()
});

pub fn get_database_path() -> &'static String {
    PATH.deref()
}

pub fn client() -> Connection {
    Connection::open(PATH.deref().as_str()).unwrap()
}