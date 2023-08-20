use anyhow::{bail, Result};
use chrono::Local;
use color_space::{CompareCie2000, Lab, Rgb};
use format_sql_query::QuotedData;
use image::load_from_memory;
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
    let mut stmt = client.prepare("SELECT id, sum FROM image ORDER BY mtime DESC LIMIT 1")?;
    let mut rows = stmt.query(named_params! {})?;
    let mut last_sum = "-".to_string();
    let mut id = 0;
    while let Some(row) = rows.next()? {
        id = row.get(0)?;
        last_sum = row.get(1)?;
    }
    let now = Local::now().timestamp_millis();
    if &last_sum == sum {
        // 设置修改时间
        client.execute(r#"UPDATE image SET mtime = ?2 WHERE id = ?1"#, (&id, &now))?;
        return Ok(());
    }
    // 正式开始插入图片
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

pub fn delete_image(image_id: i32) -> Result<()> {
    let client = client();
    client.execute(r#"DELETE FROM image WHERE id = ?1"#, (&image_id, ))?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorFilter {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    // 覆盖比例 范围：[0,1]
    pub cover_ratio_from: f64,
    pub cover_ratio_to: f64,
    // 可接受的DeltaE 范围：[0,100]
    pub difference: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetImageRequest {
    // 上一次返回图片中最小的mtime
    pub mtime: Option<i64>,
    // 返回图片的最大数量
    pub limit: Option<i64>,
    pub id: Option<Vec<i64>>,
    pub text: Option<Vec<String>>,
    pub date_range_from: Option<i64>,
    pub date_range_to: Option<i64>,
    pub color_filter: Option<ColorFilter>,
}

fn gen_where_sql(request: &GetImageRequest) -> String {
    let mut sql = "".to_string();
    if let Some(mtime) = &request.mtime {
        sql.push_str(format!(" AND mtime < {} ", mtime).as_str());
    }
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
    if let Some(date_range_from) = &request.date_range_from {
        sql.push_str(format!(" AND ctime >= {} ", date_range_from).as_str());
    }
    if let Some(date_range_to) = &request.date_range_to {
        sql.push_str(format!(" AND ctime <= {} ", date_range_to).as_str());
    }
    sql
}

fn get_image_inner(request: &GetImageRequest) -> Result<Vec<Image>> {
    let client = client();
    // 对请求做进一步处理
    let limit = request.limit.or(Some(16)).unwrap();

    // 构造SQL
    let mut sql = r#" SELECT id, image, ocr, size, width, height, ctime, mtime, sum FROM image WHERE 1 = 1 "#.to_string();
    sql.push_str(gen_where_sql(request).as_str());
    sql.push_str(" ORDER BY mtime DESC LIMIT :limit");
    let mut stmt = client.prepare(sql.as_str())?;
    let mut rows = stmt.query(named_params! {
        ":limit": limit,
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

pub fn filter_image(image: &Image, request: &GetImageRequest) -> Result<bool> {
    if let Some(color_filter) = &request.color_filter {
        // 使用DeltaE计算颜色差异 计算颜色在图片中的比重
        // 颜色差异在维基百科中的介绍：https://zh.wikipedia.org/wiki/%E9%A2%9C%E8%89%B2%E5%B7%AE%E5%BC%82
        let image = load_from_memory(image.image.as_slice())?;
        let image = image.into_rgb8();
        let width = image.width();
        let height = image.height();
        let color = Lab::from(Rgb::new(color_filter.red.clone() as f64, color_filter.green.clone() as f64, color_filter.blue.clone() as f64));
        let mut count: i64 = 0;
        for x in 0..width {
            for y in 0..height {
                let p = image.get_pixel(x.clone(), y.clone());
                let c = Lab::from(Rgb::new(p[0].clone() as f64, p[1].clone() as f64, p[2].clone() as f64));
                let delta_e = color.compare_cie2000(&c);
                if color_filter.difference >= delta_e {
                    count += 1;
                }
            }
        }
        // 覆盖率不在区间内 舍弃这个图片
        let cover_ratio = (count as f64) / (width as f64 * height as f64);
        if cover_ratio < color_filter.cover_ratio_from || cover_ratio > color_filter.cover_ratio_to {
            return Ok(false);
        }
    }
    Ok(true)
}

pub fn get_image(mut request: GetImageRequest) -> Result<Vec<Image>> {
    let mut ret = vec![];
    if let Some(limit) = request.limit {
        if limit <= 0 {
            bail!("get_image error with limit <= 0");
        }
    }
    if request.limit.is_none() {
        request.limit = Some(16);
    }
    let source_limit = request.limit.unwrap();
    loop {
        let mut mtime: Option<i64> = None;

        for image in get_image_inner(&request)? {
            match mtime {
                None => {
                    mtime = Some(image.mtime.clone());
                }
                Some(t) => {
                    mtime = Some(t.min(image.mtime.clone()));
                }
            }
            if filter_image(&image, &request)? {
                ret.push(image);
                if ret.len() as i64 >= source_limit {
                    return Ok(ret);
                }
            }
        }
        if mtime.is_none() {
            break;
        }
        request.limit = Some(request.limit.unwrap() * 2);
        request.mtime = mtime;
    }
    Ok(ret)
}