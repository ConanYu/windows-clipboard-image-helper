use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose::STANDARD_NO_PAD;
use serde::{Deserialize, Serialize};

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
#[serde(untagged)]
pub enum ImageData {
    Binary(Vec<u8>),
    Base64(String),
}

impl ImageData {
    pub fn must_binary(&self) -> &Vec<u8> {
        match self {
            Self::Binary(binary) => binary,
            _ => panic!("must_binary failed"),
        }
    }

    pub fn to_binary(self) -> Result<Self> {
        match self {
            Self::Binary(_) => Ok(self),
            Self::Base64(base64) => Ok(Self::Binary(STANDARD_NO_PAD.decode(base64.as_bytes())?)),
        }
    }

    pub fn to_base64(self) -> Self {
        match self {
            Self::Binary(binary) => Self::Base64(STANDARD_NO_PAD.encode(binary)),
            Self::Base64(_) => self,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub id: i64,
    pub image: ImageData,
    pub ocr: Option<OCR>,
    pub size: i64,
    pub width: i32,
    pub height: i32,
    pub ctime: i64,
    pub mtime: i64,
    pub sum: String,
}