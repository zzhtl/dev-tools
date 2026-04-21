use axum::extract::Multipart;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use image::codecs::{
    bmp::BmpEncoder, ico::IcoEncoder, jpeg::JpegEncoder, png::PngEncoder, webp::WebPEncoder,
};
use image::{DynamicImage, GenericImageView, ImageEncoder, ImageFormat};
use serde::Deserialize;
use std::io::Cursor;

use crate::handlers::dns::AppError;

const MAX_UPLOAD_BYTES: usize = 100 * 1024 * 1024;

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "UPPERCASE")]
pub enum OutputFormat {
    PNG,
    JPEG,
    GIF,
    WEBP,
    BMP,
    ICO,
}

impl OutputFormat {
    fn extension(&self) -> &'static str {
        match self {
            OutputFormat::PNG => "png",
            OutputFormat::JPEG => "jpg",
            OutputFormat::GIF => "gif",
            OutputFormat::WEBP => "webp",
            OutputFormat::BMP => "bmp",
            OutputFormat::ICO => "ico",
        }
    }

    fn mime(&self) -> &'static str {
        match self {
            OutputFormat::PNG => "image/png",
            OutputFormat::JPEG => "image/jpeg",
            OutputFormat::GIF => "image/gif",
            OutputFormat::WEBP => "image/webp",
            OutputFormat::BMP => "image/bmp",
            OutputFormat::ICO => "image/x-icon",
        }
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct ResizeOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    #[serde(default)]
    pub keep_aspect_ratio: bool,
}

#[derive(Deserialize, Debug, Default)]
pub struct ConvertOptions {
    pub quality: Option<u8>,
    pub resize: Option<ResizeOptions>,
}

pub async fn formats() -> Json<Vec<&'static str>> {
    Json(vec!["PNG", "JPEG", "GIF", "WEBP", "BMP", "ICO"])
}

pub async fn convert(mut mp: Multipart) -> Result<Response, AppError> {
    let mut bytes: Option<Vec<u8>> = None;
    let mut format: Option<OutputFormat> = None;
    let mut options = ConvertOptions::default();
    let mut original_name: String = "image".into();

    while let Some(field) = mp
        .next_field()
        .await
        .map_err(|e| anyhow::anyhow!("解析 multipart 失败: {e}"))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                if let Some(fname) = field.file_name() {
                    original_name = fname.to_string();
                }
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| anyhow::anyhow!("读取上传数据失败: {e}"))?;
                if data.len() > MAX_UPLOAD_BYTES {
                    return Ok(bad_request("文件过大，最大 100MB"));
                }
                bytes = Some(data.to_vec());
            }
            "format" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| anyhow::anyhow!("读取 format 字段失败: {e}"))?;
                format = parse_format(&text);
            }
            "options" => {
                let text = field.text().await.unwrap_or_default();
                if !text.is_empty() {
                    options = serde_json::from_str(&text)
                        .map_err(|e| anyhow::anyhow!("options 解析失败: {e}"))?;
                }
            }
            _ => {}
        }
    }

    let Some(bytes) = bytes else {
        return Ok(bad_request("缺少 file 字段"));
    };
    let Some(format) = format else {
        return Ok(bad_request("缺少或无效的 format 字段"));
    };

    let img = image::load_from_memory(&bytes)
        .map_err(|e| anyhow::anyhow!("解析图片失败: {e}"))?;

    let img = if let Some(resize) = options.resize.as_ref() {
        do_resize(&img, resize)
    } else if matches!(format, OutputFormat::ICO) {
        img.resize_exact(256, 256, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };

    let encoded = encode(&img, format, options.quality)?;

    let file_stem = std::path::Path::new(&original_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");
    let out_name = format!("{file_stem}.{}", format.extension());
    let (w, h) = img.dimensions();

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(format.mime()));
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{out_name}\"")).unwrap(),
    );
    headers.insert("X-File-Name", HeaderValue::from_str(&out_name).unwrap());
    headers.insert(
        "X-File-Size",
        HeaderValue::from_str(&encoded.len().to_string()).unwrap(),
    );
    headers.insert("X-Width", HeaderValue::from_str(&w.to_string()).unwrap());
    headers.insert("X-Height", HeaderValue::from_str(&h.to_string()).unwrap());
    headers.insert(
        "Access-Control-Expose-Headers",
        HeaderValue::from_static("X-File-Name, X-File-Size, X-Width, X-Height"),
    );

    Ok((StatusCode::OK, headers, encoded).into_response())
}

fn parse_format(text: &str) -> Option<OutputFormat> {
    match text.trim().to_ascii_uppercase().as_str() {
        "PNG" => Some(OutputFormat::PNG),
        "JPEG" | "JPG" => Some(OutputFormat::JPEG),
        "GIF" => Some(OutputFormat::GIF),
        "WEBP" => Some(OutputFormat::WEBP),
        "BMP" => Some(OutputFormat::BMP),
        "ICO" => Some(OutputFormat::ICO),
        _ => None,
    }
}

fn do_resize(img: &DynamicImage, opts: &ResizeOptions) -> DynamicImage {
    let (ow, oh) = img.dimensions();
    let filter = image::imageops::FilterType::Lanczos3;
    match (opts.width, opts.height) {
        (Some(w), Some(h)) if opts.keep_aspect_ratio => img.resize(w, h, filter),
        (Some(w), Some(h)) => img.resize_exact(w, h, filter),
        (Some(w), None) => {
            let h = if opts.keep_aspect_ratio {
                ((oh as f64) * (w as f64 / ow as f64)).round() as u32
            } else {
                oh
            };
            img.resize(w, h.max(1), filter)
        }
        (None, Some(h)) => {
            let w = if opts.keep_aspect_ratio {
                ((ow as f64) * (h as f64 / oh as f64)).round() as u32
            } else {
                ow
            };
            img.resize(w.max(1), h, filter)
        }
        (None, None) => img.clone(),
    }
}

fn encode(img: &DynamicImage, format: OutputFormat, quality: Option<u8>) -> anyhow::Result<Vec<u8>> {
    let mut buf = Cursor::new(Vec::<u8>::new());
    match format {
        OutputFormat::JPEG => {
            let q = quality.unwrap_or(90).min(100);
            let encoder = JpegEncoder::new_with_quality(&mut buf, q);
            let rgb = img.to_rgb8();
            encoder.write_image(
                rgb.as_raw(),
                rgb.width(),
                rgb.height(),
                image::ExtendedColorType::Rgb8,
            )?;
        }
        OutputFormat::PNG => {
            let encoder = PngEncoder::new(&mut buf);
            let rgba = img.to_rgba8();
            encoder.write_image(
                rgba.as_raw(),
                rgba.width(),
                rgba.height(),
                image::ExtendedColorType::Rgba8,
            )?;
        }
        OutputFormat::GIF => {
            img.write_to(&mut buf, ImageFormat::Gif)?;
        }
        OutputFormat::WEBP => {
            let encoder = WebPEncoder::new_lossless(&mut buf);
            let rgba = img.to_rgba8();
            encoder.write_image(
                rgba.as_raw(),
                rgba.width(),
                rgba.height(),
                image::ExtendedColorType::Rgba8,
            )?;
        }
        OutputFormat::BMP => {
            let encoder = BmpEncoder::new(&mut buf);
            let rgba = img.to_rgba8();
            encoder.write_image(
                rgba.as_raw(),
                rgba.width(),
                rgba.height(),
                image::ExtendedColorType::Rgba8,
            )?;
        }
        OutputFormat::ICO => {
            let encoder = IcoEncoder::new(&mut buf);
            let rgba = img.to_rgba8();
            encoder.write_image(
                rgba.as_raw(),
                rgba.width(),
                rgba.height(),
                image::ExtendedColorType::Rgba8,
            )?;
        }
    }
    Ok(buf.into_inner())
}

fn bad_request(msg: &str) -> Response {
    (StatusCode::BAD_REQUEST, msg.to_string()).into_response()
}
