use std::ops::Not;
use anyhow::{Result, bail};
use color_space::{CompareCie2000, Lab, Rgb};
use format_sql_query::QuotedData;
use image::{GenericImageView, load_from_memory};
use rusqlite::named_params;
use crate::app::{ColorFilter, GetImageRequest};
use crate::client::sqlite::client;
use crate::model::{Image, ImageData};

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

async unsafe fn do_color_filter_inner(image: &image::RgbImage, x: u32, y: u32, color: &Lab, difference: &f64) -> bool {
    let p = image.unsafe_get_pixel(x.clone(), y.clone());
    let c = Lab::from(Rgb::new(p[0].clone() as f64, p[1].clone() as f64, p[2].clone() as f64));
    let delta_e = color.compare_cie2000(&c);
    difference >= &delta_e
}

async fn do_color_filter(image: &Image, color_filter: &ColorFilter) -> Result<bool> {
    let image = load_from_memory(image.image.must_binary().as_slice())?;
    let image = image.into_rgb8();
    let width = image.width();
    let height = image.height();
    let color = Lab::from(Rgb::new(color_filter.red.clone() as f64, color_filter.green.clone() as f64, color_filter.blue.clone() as f64));
    let mut f = vec![];
    for x in 0..width {
        for y in 0..height {
            unsafe { f.push(do_color_filter_inner(&image, x.clone(), y.clone(), &color, &color_filter.difference)); }
        }
    }
    let mut count: i64 = 0;
    for f in f {
        if f.await {
            count += 1;
        }
    }
    // 覆盖率不在区间内会舍弃这个图片
    let cover_ratio = (count as f64) / (width as f64 * height as f64);
    Ok(!(cover_ratio < color_filter.cover_ratio_from || cover_ratio > color_filter.cover_ratio_to))
}

async fn filter_image(image: &Image, request: &GetImageRequest) -> Result<bool> {
    if let Some(color_filter) = &request.color_filter {
        // 使用DeltaE计算颜色差异 计算颜色在图片中的比重
        // 颜色差异在维基百科中的介绍：https://zh.wikipedia.org/wiki/%E9%A2%9C%E8%89%B2%E5%B7%AE%E5%BC%82
        if do_color_filter(image, color_filter).await?.not() {
            return Ok(false);
        }
    }
    Ok(true)
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
            image: ImageData::Binary(row.get(1)?),
            ocr: match row.get::<usize, Option<String>>(2)? {
                None => None,
                Some(ocr) => serde_json::from_str(ocr.as_str())?,
            },
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

pub async fn get_image(request: GetImageRequest) -> Result<Vec<Image>> {
    let mut request = request;
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
            if filter_image(&image, &request).await? {
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