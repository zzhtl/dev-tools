use image::{DynamicImage, ImageFormat, ImageOutputFormat, GenericImageView};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::path::Path;
use std::fs;
use uuid::Uuid;
use base64::{Engine as _, engine::general_purpose};

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageConversionResult {
    pub success: bool,
    pub message: String,
    pub file_path: Option<String>,
    pub file_name: Option<String>,
    pub base64_data: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ImageConversionOptions {
    pub quality: Option<u8>,  // 质量设置 (1-100)，仅适用于JPEG和WebP
    pub resize: Option<ResizeOptions>,
}

#[derive(Debug, Deserialize)]
pub struct ResizeOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub keep_aspect_ratio: bool,
}

#[derive(Debug, Deserialize)]
pub enum OutputFormat {
    PNG,
    JPEG,
    GIF,
    WEBP,
    BMP,
    ICO,
}

impl OutputFormat {
    fn to_image_format(&self) -> ImageFormat {
        match self {
            OutputFormat::PNG => ImageFormat::Png,
            OutputFormat::JPEG => ImageFormat::Jpeg,
            OutputFormat::GIF => ImageFormat::Gif,
            OutputFormat::WEBP => ImageFormat::WebP,
            OutputFormat::BMP => ImageFormat::Bmp,
            OutputFormat::ICO => ImageFormat::Ico,
        }
    }
    
    fn to_extension(&self) -> &'static str {
        match self {
            OutputFormat::PNG => "png",
            OutputFormat::JPEG => "jpg",
            OutputFormat::GIF => "gif",
            OutputFormat::WEBP => "webp",
            OutputFormat::BMP => "bmp",
            OutputFormat::ICO => "ico",
        }
    }
    
    fn to_output_format(&self, options: Option<&ImageConversionOptions>) -> ImageOutputFormat {
        match self {
            OutputFormat::PNG => ImageOutputFormat::Png,
            OutputFormat::JPEG => {
                let quality = options
                    .and_then(|opt| opt.quality)
                    .unwrap_or(90) // 默认质量90
                    .min(100); // 确保不超过100
                ImageOutputFormat::Jpeg(quality)
            },
            OutputFormat::GIF => ImageOutputFormat::Gif,
            OutputFormat::WEBP => ImageOutputFormat::WebP,
            OutputFormat::BMP => ImageOutputFormat::Bmp,
            OutputFormat::ICO => ImageOutputFormat::Ico,
        }
    }
}

#[tauri::command]
pub fn convert_image(source_path: &str, format: OutputFormat, options: Option<ImageConversionOptions>) -> ImageConversionResult {
    // 检查文件是否存在
    if !std::path::Path::new(source_path).exists() {
        return ImageConversionResult {
            success: false,
            message: "文件不存在或无法访问".to_string(),
            file_path: None,
            file_name: None,
            base64_data: None,
        };
    }

    match image::open(source_path) {
        Ok(mut img) => {
            // 处理图片大小调整
            if let Some(opts) = &options {
                if let Some(resize_opts) = &opts.resize {
                    img = resize_image(&img, resize_opts);
                }
            }
            
            let new_extension = format.to_extension();
            let file_stem = Path::new(source_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("converted");
            
            let output_file_name = format!("{}-{}.{}", file_stem, Uuid::new_v4(), new_extension);
            let temp_dir = std::env::temp_dir();
            let output_path = temp_dir.join(&output_file_name);
            
            let output_path_str = output_path.to_string_lossy().to_string();
            
            match save_image(&img, &output_path, &format, options.as_ref()) {
                Ok(_) => {
                    // 转换为base64用于预览
                    let mut buffer = Vec::new();
                    let mut cursor = Cursor::new(&mut buffer);
                    if let Err(e) = img.write_to(&mut cursor, format.to_output_format(options.as_ref())) {
                        return ImageConversionResult {
                            success: false,
                            message: format!("基本转换成功，但创建预览失败: {}", e),
                            file_path: Some(output_path_str),
                            file_name: Some(output_file_name),
                            base64_data: None,
                        };
                    }
                    
                    let base64_prefix = match format {
                        OutputFormat::PNG => "data:image/png;base64,",
                        OutputFormat::JPEG => "data:image/jpeg;base64,",
                        OutputFormat::GIF => "data:image/gif;base64,",
                        OutputFormat::WEBP => "data:image/webp;base64,",
                        OutputFormat::BMP => "data:image/bmp;base64,",
                        OutputFormat::ICO => "data:image/x-icon;base64,",
                    };
                    
                    let base64_data = format!(
                        "{}{}",
                        base64_prefix,
                        general_purpose::STANDARD.encode(&buffer)
                    );
                    
                    ImageConversionResult {
                        success: true,
                        message: format!("图片已成功转换为{}", new_extension.to_uppercase()),
                        file_path: Some(output_path_str),
                        file_name: Some(output_file_name),
                        base64_data: Some(base64_data),
                    }
                }
                Err(e) => ImageConversionResult {
                    success: false,
                    message: format!("保存图片失败: {}", e),
                    file_path: None,
                    file_name: None,
                    base64_data: None,
                },
            }
        }
        Err(e) => ImageConversionResult {
            success: false,
            message: format!("打开图片失败: {}", e),
            file_path: None,
            file_name: None,
            base64_data: None,
        },
    }
}

// 调整图片大小
fn resize_image(img: &DynamicImage, options: &ResizeOptions) -> DynamicImage {
    let (original_width, original_height) = img.dimensions();
    
    match (options.width, options.height) {
        (Some(width), Some(height)) => {
            if options.keep_aspect_ratio {
                // 保持纵横比例
                img.resize(width, height, image::imageops::FilterType::Lanczos3)
            } else {
                // 强制调整为指定尺寸
                img.resize_exact(width, height, image::imageops::FilterType::Lanczos3)
            }
        },
        (Some(width), None) => {
            // 只指定宽度
            let height = if options.keep_aspect_ratio {
                (original_height as f64 * (width as f64 / original_width as f64)).round() as u32
            } else {
                original_height
            };
            img.resize(width, height, image::imageops::FilterType::Lanczos3)
        },
        (None, Some(height)) => {
            // 只指定高度
            let width = if options.keep_aspect_ratio {
                (original_width as f64 * (height as f64 / original_height as f64)).round() as u32
            } else {
                original_width
            };
            img.resize(width, height, image::imageops::FilterType::Lanczos3)
        },
        (None, None) => {
            // 未指定大小，返回原图
            img.clone()
        }
    }
}

fn save_image(
    img: &DynamicImage,
    path: &Path,
    format: &OutputFormat,
    options: Option<&ImageConversionOptions>,
) -> Result<(), image::ImageError> {
    // 对于JPEG和WebP格式，尝试使用质量设置
    match format {
        OutputFormat::JPEG => {
            let quality = options
                .and_then(|opt| opt.quality)
                .unwrap_or(90)
                .min(100);
                
            // 使用指定质量保存JPEG
            img.save_with_format(path, format.to_image_format())?;
            Ok(())
        },
        OutputFormat::WEBP => {
            // WebP格式也支持质量设置，但目前image库没有直接API
            img.save_with_format(path, format.to_image_format())
        },
        _ => img.save_with_format(path, format.to_image_format()),
    }
}

#[tauri::command]
pub fn get_supported_image_formats() -> Vec<String> {
    vec![
        "PNG".to_string(),
        "JPEG".to_string(),
        "GIF".to_string(),
        "WEBP".to_string(),
        "BMP".to_string(),
        "ICO".to_string(),
    ]
}

#[tauri::command]
pub fn get_image_info(file_path: &str) -> Result<ImageInfo, String> {
    if !std::path::Path::new(file_path).exists() {
        return Err("文件不存在或无法访问".to_string());
    }

    match image::open(file_path) {
        Ok(img) => {
            let dimensions = img.dimensions();
            let format = match img.color() {
                image::ColorType::L8 => "8位灰度图",
                image::ColorType::La8 => "8位灰度 + Alpha",
                image::ColorType::Rgb8 => "8位 RGB",
                image::ColorType::Rgba8 => "8位 RGBA",
                image::ColorType::L16 => "16位灰度图",
                image::ColorType::La16 => "16位灰度 + Alpha",
                image::ColorType::Rgb16 => "16位 RGB",
                image::ColorType::Rgba16 => "16位 RGBA",
                image::ColorType::Rgb32F => "32位浮点 RGB",
                image::ColorType::Rgba32F => "32位浮点 RGBA",
                _ => "未知格式",
            };

            // 获取文件大小
            let metadata = std::fs::metadata(file_path).map_err(|e| e.to_string())?;
            let size_bytes = metadata.len();

            Ok(ImageInfo {
                width: dimensions.0,
                height: dimensions.1,
                color_type: format.to_string(),
                file_size: size_bytes,
                file_path: file_path.to_string(),
            })
        },
        Err(e) => Err(format!("无法打开图片文件: {}", e)),
    }
}

#[derive(Debug, Serialize)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub color_type: String,
    pub file_size: u64,
    pub file_path: String,
}

#[tauri::command]
pub fn copy_file(source_path: &str, dest_path: &str) -> Result<bool, String> {
    if !std::path::Path::new(source_path).exists() {
        return Err("源文件不存在".to_string());
    }
    
    match fs::copy(source_path, dest_path) {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("复制文件失败: {}", e))
    }
}

#[tauri::command]
pub fn get_image_base64(file_path: &str) -> Result<String, String> {
    if !std::path::Path::new(file_path).exists() {
        return Err("文件不存在或无法访问".to_string());
    }
    
    match image::open(file_path) {
        Ok(img) => {
            // 确定图片格式
            let format = match Path::new(file_path).extension().and_then(|s| s.to_str()) {
                Some("jpg") | Some("jpeg") => "image/jpeg",
                Some("png") => "image/png",
                Some("gif") => "image/gif",
                Some("webp") => "image/webp",
                Some("bmp") => "image/bmp",
                Some("ico") => "image/x-icon",
                _ => "image/jpeg", // 默认格式
            };
            
            // 转换为base64
            let mut buffer = Vec::new();
            let mut cursor = Cursor::new(&mut buffer);
            
            // 根据文件后缀决定编码格式
            let write_result = match format {
                "image/jpeg" => img.write_to(&mut cursor, ImageOutputFormat::Jpeg(90)),
                "image/png" => img.write_to(&mut cursor, ImageOutputFormat::Png),
                "image/gif" => img.write_to(&mut cursor, ImageOutputFormat::Gif),
                "image/webp" => img.write_to(&mut cursor, ImageOutputFormat::WebP),
                "image/bmp" => img.write_to(&mut cursor, ImageOutputFormat::Bmp),
                "image/x-icon" => img.write_to(&mut cursor, ImageOutputFormat::Ico),
                _ => img.write_to(&mut cursor, ImageOutputFormat::Jpeg(90)),
            };
            
            if let Err(e) = write_result {
                return Err(format!("转换图片失败: {}", e));
            }
            
            // 生成base64数据URL
            let base64_data = format!(
                "data:{};base64,{}",
                format,
                general_purpose::STANDARD.encode(&buffer)
            );
            
            Ok(base64_data)
        },
        Err(e) => Err(format!("无法打开图片文件: {}", e)),
    }
} 