use anyhow::Result;
use chrono::Local;
use format_sql_query::QuotedData;
use rusqlite::named_params;
use serde::{Deserialize, Serialize};
use crate::dal::client::client;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OCRBox {
    pub r#box: [[u32; 2]; 4],
    pub score: f64,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OCRData {
    Box(Vec<OCRBox>),
    Text(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OCR {
    pub code: i32,
    pub data: OCRData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub id: i64,
    pub image: Vec<u8>,
    pub ocr: Option<OCR>,
    pub size: i64,
    pub width: i32,
    pub height: i32,
    pub ctime: i64,
    pub mtime: i64,
    pub sum: String,
}

pub fn insert_image(image: &Vec<u8>, width: &i32, height: &i32, ocr: &Option<OCR>, sum: &String) -> Result<()> {
    let client = client();
    // 校验是否是上一次插入的图片
    let mut stmt = client.prepare("SELECT sum FROM image ORDER BY mtime DESC LIMIT 1")?;
    let mut rows = stmt.query(named_params! {})?;
    let mut last_sum = "-".to_string();
    while let Some(row) = rows.next()? {
        last_sum = row.get(0)?;
    }
    if &last_sum == sum {
        return Ok(());
    }
    // 正式开始插入图片
    let now = Local::now().timestamp_millis();
    let ocr = match ocr {
        None => None,
        Some(ocr) => Some(serde_json::to_string(ocr)?),
    };
    let size = image.len() as i64;
    client.execute(r#"INSERT INTO image (image, ocr, size, width, height, ctime, mtime, sum)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
                   (image, &ocr, &size, width, height, &now, &now, sum))?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetImageRequest {
    pub page_no: Option<i64>,
    pub page_size: Option<i64>,
    pub id: Option<Vec<i64>>,
    pub text: Option<Vec<String>>,
}

fn gen_where_sql(request: &GetImageRequest) -> String {
    let mut sql = "".to_string();
    if let Some(id) = &request.id {
        if id.len() > 0 {
            let v: Vec<String> = id.iter().map(|v| v.to_string()).collect();
            let s = v.join(", ");
            sql.push_str(format!(" AND id IN ({}) ", s).as_str());
        }
    }
    if let Some(text) = &request.text {
        if text.len() > 0 {
            let s: Vec<String> = text.iter().map(|v| {
                format!(r#" (id IN (
                    SELECT image.id FROM image, JSON_EACH(JSON_EXTRACT(ocr, '$.data')) AS j
                    WHERE JSON_EXTRACT(image.ocr, '$.code') = 100
                        AND JSON_EXTRACT(j.value, '$.text') LIKE '%' || {} || '%'
                )) "#, QuotedData(v))
            }).collect();
            sql.push_str(format!(" AND ({}) ", s.join(" OR ")).as_str());
        }
    }
    sql
}

pub fn get_image(request: &GetImageRequest) -> Result<Vec<Image>> {
    let client = client();
    // 对请求做进一步处理
    let page_no = request.page_no.or(Some(1)).unwrap();
    let page_size = request.page_size.or(Some(16)).unwrap();
    let offset = (page_no - 1) * page_size;

    // 构造SQL
    let mut sql = r#" SELECT id, image, ocr, size, width, height, ctime, mtime, sum FROM image WHERE 1 = 1 "#.to_string();
    sql.push_str(gen_where_sql(request).as_str());
    sql.push_str("ORDER BY mtime DESC LIMIT :page_size OFFSET :offset ");
    let mut stmt = client.prepare(sql.as_str())?;
    let mut rows = stmt.query(named_params! {
        ":page_size": page_size,
        ":offset": offset,
    })?;

    // 构造返回值
    let mut ret = vec![];
    while let Some(row) = rows.next()? {
        ret.push(Image {
            id: row.get(0)?,
            image: row.get(1)?,
            ocr: serde_json::from_str(row.get::<usize, String>(2)?.as_str())?,
            size: row.get(3)?,
            width: row.get(4)?,
            height: row.get(5)?,
            ctime: row.get(6)?,
            mtime: row.get(7)?,
            sum: row.get(8)?,
        });
    }
    Ok(ret)
}