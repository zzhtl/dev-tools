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
    SVG,
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
            OutputFormat::SVG => ImageFormat::Png, // SVG需要特殊处理，暂时用PNG
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
            OutputFormat::SVG => "svg",
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
            OutputFormat::SVG => ImageOutputFormat::Png, // SVG需要特殊处理
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

    // 检查文件是否为SVG格式
    let is_svg = source_path.to_lowercase().ends_with(".svg");
    
    // 加载图片
    let load_result = if is_svg {
        // SVG文件需要特殊处理
        // 目前我们不直接支持SVG输入，给出更友好的错误信息
        Err(image::ImageError::Unsupported(
            image::error::UnsupportedError::from_format_and_kind(
                image::error::ImageFormatHint::Name("SVG".to_string()),
                image::error::UnsupportedErrorKind::Format(image::error::ImageFormatHint::Name(
                    "当前版本不支持SVG输入，请选择其他格式的图片".to_string(),
                )),
            )
        ))
    } else {
        image::open(source_path)
    };

    match load_result {
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
                    // 对于SVG格式，我们需要特殊处理预览数据
                    if matches!(format, OutputFormat::SVG) {
                        // 读取生成的SVG文件
                        let svg_content = fs::read_to_string(&output_path).unwrap_or_default();
                        
                        // 将SVG内容编码为base64，创建data URL
                        let svg_base64 = general_purpose::STANDARD.encode(svg_content.as_bytes());
                        let data_url = format!("data:image/svg+xml;base64,{}", svg_base64);
                        
                        return ImageConversionResult {
                            success: true,
                            message: format!("图片已成功转换为SVG"),
                            file_path: Some(output_path_str),
                            file_name: Some(output_file_name),
                            base64_data: Some(data_url),
                        };
                    }
                    
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
                        OutputFormat::SVG => "data:image/svg+xml;base64,", // 这行实际上不会执行，因为SVG已经提前返回了
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
    match format {
        OutputFormat::JPEG => {
            // 使用指定质量保存JPEG
            img.save_with_format(path, format.to_image_format())?;
            Ok(())
        },
        OutputFormat::ICO => {
            // ICO格式需要特殊处理 - 完全重写
            // 从选项中获取尺寸信息
            let target_size = if let Some(opts) = options {
                if let Some(resize_opts) = &opts.resize {
                    resize_opts.width.unwrap_or(32) // 使用指定的宽度或默认值32
                } else {
                    32 // 默认为32x32
                }
            } else {
                32 // 默认为32x32
            };
            
            // 确保尺寸是有效的ICO尺寸
            let valid_size = match target_size {
                s if s <= 16 => 16,
                s if s <= 32 => 32,
                s if s <= 48 => 48,
                s if s <= 64 => 64,
                s if s <= 128 => 128,
                _ => 256,
            };
            
            // 调整到目标尺寸并确保是RGBA格式（支持透明度）
            let resized_img = img.resize_exact(valid_size, valid_size, image::imageops::FilterType::Lanczos3)
                                .to_rgba8();
            
            // 创建一个简单的ICO文件结构
            // ICO文件格式: https://en.wikipedia.org/wiki/ICO_(file_format)
            
            // 1. 首先将图像数据转换为BMP格式（没有压缩的ICO是BMP格式）
            let width = resized_img.width();
            let height = resized_img.height();
            let bmp_data = create_bmp_data(&resized_img)?;
            
            // 2. 创建ICO文件头 (6字节)
            let mut ico_data = Vec::new();
            // 魔术数字: 0 (保留), 1 (ICO类型), 1 (包含1个图像)
            ico_data.extend_from_slice(&[0, 0, 1, 0, 1, 0]);
            
            // 3. 图像目录 (16字节)
            let bmp_size = bmp_data.len() as u32;
            
            // 宽度和高度 (0-255, 0表示256)
            let w = if width == 256 { 0 } else { width as u8 };
            let h = if height == 256 { 0 } else { height as u8 };
            
            ico_data.push(w);  // 宽度
            ico_data.push(h);  // 高度
            ico_data.push(0);  // 调色板大小 (不使用调色板)
            ico_data.push(0);  // 保留位
            
            // Planes 必须为1
            ico_data.extend_from_slice(&[1, 0]);
            
            // 位深度 (32位 RGBA)
            ico_data.extend_from_slice(&[32, 0]);
            
            // BMP数据大小
            ico_data.extend_from_slice(&bmp_size.to_le_bytes());
            
            // BMP数据偏移量 (从文件开始算，6+16=22字节)
            ico_data.extend_from_slice(&[22, 0, 0, 0]);
            
            // 4. 添加BMP图像数据
            ico_data.extend_from_slice(&bmp_data);
            
            // 5. 写入ICO文件
            fs::write(path, ico_data).map_err(|e| {
                image::ImageError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("写入ICO文件失败: {}", e)
                ))
            })?;
            
            Ok(())
        },
        OutputFormat::SVG => {
            // 现在我们处理SVG转换，首先保存为PNG
            let png_path = path.with_extension("png");
            img.save_with_format(&png_path, ImageFormat::Png)?;
            
            // 创建一个标准SVG文件，内嵌PNG数据
            let (width, height) = img.dimensions();
            
            // 读取PNG文件并进行base64编码
            let png_data = fs::read(&png_path).map_err(|e| {
                image::ImageError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("无法读取PNG文件: {}", e)
                ))
            })?;
            
            let base64_png = general_purpose::STANDARD.encode(&png_data);
            
            // 创建更标准的SVG文件，嵌入PNG数据
            let svg_content = format!(
                r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<svg width="{width}" height="{height}" viewBox="0 0 {width} {height}" 
     xmlns="http://www.w3.org/2000/svg" 
     xmlns:xlink="http://www.w3.org/1999/xlink">
  <title>转换图片</title>
  <desc>使用工具转换的栅格图像</desc>
  <image width="{width}" height="{height}" x="0" y="0"
         xlink:href="data:image/png;base64,{base64_png}"/>
</svg>"#
            );
            
            // 写入SVG文件
            fs::write(path, &svg_content).map_err(|e| {
                image::ImageError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("无法写入SVG文件: {}", e)
                ))
            })?;
            
            // 清理临时文件
            let _ = fs::remove_file(&png_path);
            
            Ok(())
        },
        OutputFormat::WEBP => {
            // WebP格式也支持质量设置，但目前image库没有直接API
            img.save_with_format(path, format.to_image_format())
        },
        _ => img.save_with_format(path, format.to_image_format()),
    }
}

// 创建未压缩的BMP数据
fn create_bmp_data(img: &image::RgbaImage) -> Result<Vec<u8>, image::ImageError> {
    let width = img.width();
    let height = img.height();
    
    // BMP文件中图像数据是从下到上存储的
    // ICO文件中的BMP数据不需要完整的BMP文件头，只需要BITMAPINFOHEADER和像素数据
    let mut bmp_data = Vec::new();
    
    // BITMAPINFOHEADER (40字节)
    let header_size = 40u32;
    bmp_data.extend_from_slice(&header_size.to_le_bytes());  // 头部大小
    bmp_data.extend_from_slice(&width.to_le_bytes());        // 宽度
    bmp_data.extend_from_slice(&(height * 2).to_le_bytes()); // 高度 (在ICO中是两倍高度)
    bmp_data.extend_from_slice(&[1, 0]);                    // 平面数
    bmp_data.extend_from_slice(&[32, 0]);                   // 位深度 (32位)
    bmp_data.extend_from_slice(&[0, 0, 0, 0]);              // 不压缩
    
    // 图像数据大小 (width * height * 4字节)
    let data_size = (width * height * 4) as u32;
    bmp_data.extend_from_slice(&data_size.to_le_bytes());
    
    // 分辨率 (不重要，使用默认值)
    bmp_data.extend_from_slice(&[0, 0, 0, 0]);              // X分辨率
    bmp_data.extend_from_slice(&[0, 0, 0, 0]);              // Y分辨率
    bmp_data.extend_from_slice(&[0, 0, 0, 0]);              // 调色板颜色数 (不使用)
    bmp_data.extend_from_slice(&[0, 0, 0, 0]);              // 重要颜色数 (不使用)
    
    // 像素数据 (从下到上，从左到右)
    for y in (0..height).rev() {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            // BGR顺序（BMP使用BGR，与RGB相反）
            bmp_data.push(pixel[2]);  // B
            bmp_data.push(pixel[1]);  // G
            bmp_data.push(pixel[0]);  // R
            bmp_data.push(pixel[3]);  // A
        }
    }
    
    Ok(bmp_data)
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
        "SVG".to_string(),
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
                Some("svg") => "image/svg+xml",
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
                "image/svg+xml" => img.write_to(&mut cursor, ImageOutputFormat::Png),
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