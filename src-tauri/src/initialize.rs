use anyhow::Result;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use log::{LevelFilter};
use crate::dal::client::client;

pub fn init_logger() -> Result<()> {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[Console] {d} - {l} - {t} - {m}{n}")))
        .build();
    let file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[File] {d} - {l} - {t} - {m}{n}")))
        .build("log.txt")?;
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("file", Box::new(file)))
        .logger(Logger::builder().appender("file").additive(false).build("app", LevelFilter::Info))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))?;
    let _ = log4rs::init_config(config)?;
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