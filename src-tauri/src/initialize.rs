use anyhow::Result;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use log::{info, LevelFilter};
use rusqlite::{Connection, named_params};
use crate::dal::client::client;
use crate::settings::get_root;

pub fn init_logger() -> Result<log4rs::Handle> {
    let root = get_root();
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {l} - {t} - {m}{n}")))
        .build(root.join("log.txt"))?;
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder()
            .appender("logfile")
            .build(LevelFilter::Info))?;
    let handle = log4rs::init_config(config)?;
    info!("----------------------------------------------------------------------------------------------------");
    info!("root path: {:?}", root.as_os_str());
    std::panic::set_hook(Box::new(|info| {
        log::error!("Panicked: {}", info);
    }));
    Ok(handle)
}

#[allow(dead_code)]
fn add_column_if_not_exist(client: &Connection, table: &str, column: &str, kind: &str) -> Result<()> {
    let f = || -> Result<()> {
        let mut stmt = client.prepare(format!("SELECT {} FROM {} LIMIT 1", column, table).as_str())?;
        let mut rows = stmt.query(named_params! {})?;
        while let Some(_) = rows.next()? {}
        Ok(())
    };
    let r = f();
    if r.is_err() {
        client.execute(format!(r"ALTER TABLE {} ADD COLUMN {} {}", table, column, kind).as_str(), ())?;
    }
    Ok(())
}

pub fn init_database() -> Result<()> {
    let client = client();
    client.execute(r#"
    CREATE TABLE IF NOT EXISTS image (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        image BLOB,
        ocr TEXT,
        size INTEGER,
        width INTEGER,
        height INTEGER,
        ctime INTEGER,
        mtime INTEGER,
        sum VARCHAR(64)
    );"#, ())?;
    client.execute(r"CREATE INDEX IF NOT EXISTS index_mtime ON image (mtime)", ())?;
    client.execute(r"CREATE INDEX IF NOT EXISTS index_sum ON image (sum)", ())?;
    Ok(())
}