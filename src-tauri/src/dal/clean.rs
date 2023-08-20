use std::fs;
use anyhow::Result;
use log::error;
use rusqlite::named_params;
use crate::dal::client::{client, get_database_path};
use crate::settings::{get_settings, DatabaseLimitType};

pub fn get_count_of_image() -> Result<i64> {
    let client = client();
    let mut stmt = client.prepare("SELECT count(*) FROM image")?;
    let mut rows = stmt.query(named_params! {})?;
    let mut count = 0;
    while let Some(row) = rows.next()? {
        count = row.get(0)?;
    }
    Ok(count)
}

pub async fn regular_cleaning() {
    loop {
        let settings = get_settings();
        let mut need_clean = false;
        let database_limit_type = &settings.database_limit_type.or(Some(DatabaseLimitType::MB)).unwrap();
        match database_limit_type {
            DatabaseLimitType::MB => {
                let result = fs::metadata(get_database_path());
                match result {
                    Ok(data) => need_clean = (data.len() / 1024 / 1024) as i64 > settings.database_limit.or(Some(1024)).unwrap(),
                    Err(err) => error!("regular cleaning error: {}", err.to_string()),
                }
            }
            DatabaseLimitType::NUM => {
                let count = get_count_of_image();
                match count {
                    Ok(count) => need_clean = count > settings.database_limit.or(Some(1024)).unwrap(),
                    Err(err) => error!("regular cleaning error: {}", err.to_string()),
                }
            }
        }
        if need_clean {
            let client = client();
            const SQL: &str = r#"DELETE FROM image WHERE id IN (SELECT id FROM image ORDER BY mtime LIMIT 1)"#;
            if let Err(err) = client.execute(SQL, ()) {
                error!("regular cleaning error: {}", err.to_string());
            }
        }
        // 每20秒检查一次 超了只删一个
        tokio::time::sleep(std::time::Duration::from_secs(20)).await;
    }
}