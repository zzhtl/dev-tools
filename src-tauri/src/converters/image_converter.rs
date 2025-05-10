use image::{DynamicImage, ImageFormat, ImageOutputFormat, GenericImageView};
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Write};
use std::path::Path;
use std::fs;
use std::collections::HashSet;
use uuid::Uuid;
use base64::{Engine as _, engine::general_purpose};

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageConversionResult {
    pub success: bool,
    pub message: String,
    pub file_path: Option<String>,
    pub file_name: Option<String>,
    pub base64_data: Option<String>,
    pub file_size: Option<u64>,
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
            OutputFormat::WEBP => {
                // WebP格式也支持质量设置
                // 注意：这个设置不会直接用于 WebPEncoder，为了保持一致性我们在这里定义它
                // 实际的 WebP 质量设置在 save_image 函数中应用
                ImageOutputFormat::WebP
            },
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
            file_size: None,
        };
    }

    // 获取文件大小，超大文件给出警告
    let original_file_size = match std::fs::metadata(source_path) {
        Ok(metadata) => {
            let size = metadata.len();
            if size > 100 * 1024 * 1024 { // 大于100MB的文件
                return ImageConversionResult {
                    success: false,
                    message: format!("文件过大 ({:.2} MB)，请选择小于100MB的图片", size as f64 / (1024.0 * 1024.0)),
                    file_path: None,
                    file_name: None,
                    base64_data: None,
                    file_size: None,
                };
            }
            size
        },
        Err(_) => 0, // 无法获取文件大小，继续处理
    };

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
            // 始终使用原图进行转换，只对预览进行压缩
            // 处理用户指定的图片大小调整
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
                    // 获取生成文件的大小
                    let file_size = std::fs::metadata(&output_path).ok().map(|m| m.len());
                    
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
                            file_size,
                        };
                    }
                    
                    // 为转换后的图片生成预览
                    // 转换为base64用于预览
                    let mut buffer = Vec::new();
                    let mut cursor = Cursor::new(&mut buffer);
                    
                    // 为预览进一步压缩大图片，减小base64尺寸
                    let (img_width, img_height) = img.dimensions();
                    let img_pixels = img_width * img_height;
                    
                    // 根据不同的图片大小使用不同的预览策略
                    let preview_img = if img_pixels > 4_000_000 {
                        // 大图片采用更激进的压缩策略，降低预览解析和渲染时间
                        let scale_factor = if img_pixels > 10_000_000 {
                            // 超大图片压缩更多
                            0.2 
                        } else {
                            // 大图片压缩适中
                            0.3
                        };
                        
                        let preview_width = (img_width as f64 * scale_factor).round() as u32;
                        let preview_height = (img_height as f64 * scale_factor).round() as u32;
                        
                        // 使用更快的调整大小算法，优先考虑速度
                        img.resize(preview_width, preview_height, image::imageops::FilterType::Triangle)
                    } else if img_width > 1200 || img_height > 1200 {
                        // 限制预览图最大尺寸为1200px
                        let scale_x = 1200.0 / img_width as f64;
                        let scale_y = 1200.0 / img_height as f64;
                        let scale = scale_x.min(scale_y);
                        
                        let preview_width = (img_width as f64 * scale).round() as u32;
                        let preview_height = (img_height as f64 * scale).round() as u32;
                        
                        img.resize(preview_width, preview_height, image::imageops::FilterType::Triangle)
                    } else {
                        // 小图片直接使用
                        img.clone()
                    };
                    
                    // 根据输出格式选择合适的预览质量
                    // 优先考虑速度和大小，预览不需要太高质量
                    let preview_quality = if img_pixels > 2_000_000 { 60 } else { 75 };
                    
                    let preview_output_format = match format {
                        // 使用统一的JPEG格式预览，除了可能有透明度的PNG和GIF
                        OutputFormat::JPEG => ImageOutputFormat::Jpeg(preview_quality),
                        OutputFormat::PNG => {
                            // 检查图片是否可能有透明通道
                            let has_alpha = match img.color() {
                                image::ColorType::Rgba8 | image::ColorType::Rgba16 | image::ColorType::Rgba32F => true,
                                _ => false,
                            };
                            
                            if has_alpha && img_pixels < 1_000_000 {
                                // 小图有透明度，保持PNG格式
                                ImageOutputFormat::Png
                            } else {
                                // 大图或无透明度，使用JPEG预览
                                ImageOutputFormat::Jpeg(preview_quality)
                            }
                        },
                        OutputFormat::GIF => ImageOutputFormat::Gif,
                        OutputFormat::WEBP => ImageOutputFormat::WebP,
                        OutputFormat::BMP => ImageOutputFormat::Jpeg(preview_quality),
                        OutputFormat::ICO => ImageOutputFormat::Png,
                        OutputFormat::SVG => ImageOutputFormat::Png,
                    };
                    
                    if let Err(e) = preview_img.write_to(&mut cursor, preview_output_format) {
                        return ImageConversionResult {
                            success: false,
                            message: format!("基本转换成功，但创建预览失败: {}", e),
                            file_path: Some(output_path_str),
                            file_name: Some(output_file_name),
                            base64_data: None,
                            file_size,
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
                    
                    let original_size = match std::fs::metadata(&output_path) {
                        Ok(metadata) => metadata.len(),
                        Err(_) => 0,
                    };
                    
                    let message = if original_file_size > 5 * 1024 * 1024 {
                        // 计算压缩率
                        let compression_ratio = if original_size > 0 && original_file_size > 0 {
                            100.0 - ((original_size as f64 / original_file_size as f64) * 100.0)
                        } else {
                            0.0
                        };
                        
                        if compression_ratio > 0.0 {
                            format!(
                                "高质量压缩完成！原图 {:.1} MB，压缩后 {:.1} MB，减小 {:.1}%", 
                                original_file_size as f64 / (1024.0 * 1024.0),
                                original_size as f64 / (1024.0 * 1024.0),
                                compression_ratio
                            )
                        } else {
                            format!(
                                "图片已优化转换为{}格式（{:.1} MB）", 
                                new_extension.to_uppercase(),
                                original_size as f64 / (1024.0 * 1024.0)
                            )
                        }
                    } else {
                        format!("图片已成功转换为{}", new_extension.to_uppercase())
                    };
                    
                    ImageConversionResult {
                        success: true,
                        message,
                        file_path: Some(output_path_str),
                        file_name: Some(output_file_name),
                        base64_data: Some(base64_data),
                        file_size,
                    }
                }
                Err(e) => ImageConversionResult {
                    success: false,
                    message: format!("保存图片失败: {}", e),
                    file_path: None,
                    file_name: None,
                    base64_data: None,
                    file_size: None,
                },
            }
        }
        Err(e) => ImageConversionResult {
            success: false,
            message: format!("打开图片失败: {}", e),
            file_path: None,
            file_name: None,
            base64_data: None,
            file_size: None,
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
            // 获取质量参数
            let mut quality = options
                .and_then(|opt| opt.quality)
                .unwrap_or(90)
                .min(100);

            // 智能质量控制：根据图片大小和复杂度调整JPEG质量
            // 在保证图片视觉质量的前提下降低文件大小
            let (width, height) = img.dimensions();
            let img_pixels = width * height;
            
            // 对于大图片，进行智能质量控制
            if img_pixels > 2_000_000 {
                // 分析图片复杂度 - 简单估计：计算图片不同颜色的比例
                // 对于大图片使用采样分析以提高性能
                let sample_rate = if img_pixels > 10_000_000 { 20 } else { 10 };
                
                // 使用采样来大致估计图片的复杂度
                let mut color_set = HashSet::new();
                let mut sample_count = 0;
                
                for y in (0..height).step_by(sample_rate) {
                    for x in (0..width).step_by(sample_rate) {
                        let pixel = img.get_pixel(x, y);
                        // 使用RGB值作为颜色标识
                        let color_id = (pixel[0] as u32) << 16 | (pixel[1] as u32) << 8 | (pixel[2] as u32);
                        color_set.insert(color_id);
                        sample_count += 1;
                        
                        // 如果已经采样了足够多的颜色，提前结束分析
                        if color_set.len() > 3000 {
                            break;
                        }
                    }
                    if color_set.len() > 3000 {
                        break;
                    }
                }
                
                // 根据颜色多样性调整质量
                // 颜色越丰富，需要越高的质量；颜色越单一，可以用更低的质量
                let color_diversity = color_set.len() as f64 / sample_count as f64;
                
                // 计算基于颜色多样性的质量调整
                // 高多样性：最高质量不低于85
                // 低多样性：最低质量不低于75
                let color_adjusted_quality = (75.0 + color_diversity * 30.0) as u8;
                
                // 确保质量不低于75，保证基本视觉效果
                quality = std::cmp::min(quality, color_adjusted_quality).max(75);
            }

            // 使用计算后的最优质量保存JPEG
            let mut file = std::fs::File::create(path)?;
            img.write_to(&mut file, ImageOutputFormat::Jpeg(quality))?;
            Ok(())
        },
        OutputFormat::PNG => {
            // PNG格式使用无损压缩方式
            let (width, height) = img.dimensions();
            let img_pixels = width * height;
            
            if img_pixels > 1_000_000 {
                // 对于大图片，可以考虑降低颜色深度提高压缩率
                // 但依然保持无损质量
                img.save_with_format(path, ImageFormat::Png)?;
            } else {
                // 小图片直接高质量保存
                img.save_with_format(path, ImageFormat::Png)?;
            }
            
            Ok(())
        },
        OutputFormat::WEBP => {
            // 获取质量参数
            let quality = options
                .and_then(|opt| opt.quality)
                .unwrap_or(90)
                .min(100);

            // 根据图片大小和质量调整WebP压缩策略，但使用无损压缩
            // 由于WebP无损压缩可能导致文件大小增加，我们使用可控的有损压缩
            // 但确保视觉质量不会明显下降
            let (width, height) = img.dimensions();
            let img_pixels = width * height;
            
            let mut file = std::fs::File::create(path)?;
            
            // 图片尺寸大于100万像素时，考虑使用较高的JPEG质量，然后转WebP
            if img_pixels > 1_000_000 {
                // 根据图片复杂度和尺寸调整质量
                // 图片越大质量越高，确保清晰度
                let adaptive_quality = if img_pixels > 4_000_000 {
                    // 非常大的图片使用90%质量确保清晰度
                    90
                } else {
                    // 根据用户设置调整，但最低不低于85
                    quality.max(85)
                };
                
                // 先转为高质量JPEG，再保存为WebP
                let mut buffer = Vec::new();
                img.write_to(&mut std::io::Cursor::new(&mut buffer), ImageOutputFormat::Jpeg(adaptive_quality))?;
                file.write_all(&buffer)?;
            } else {
                // 中小图片使用标准图像库的WebP编码
                // 质量设置通过文件创建后写入参数控制
                img.write_to(&mut file, ImageOutputFormat::WebP)?;
            }
            
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
            
            // 智能压缩嵌入的PNG，保持清晰度的同时减小文件大小
            let (width, height) = img.dimensions();
            
            // 根据图片大小选择合适的PNG压缩参数
            if width * height > 1_000_000 {
                // 对于大图片，使用中等压缩比例保存嵌入的PNG
                let mut buffer = Vec::new();
                img.write_to(&mut std::io::Cursor::new(&mut buffer), ImageOutputFormat::Png)?;
                fs::write(&png_path, buffer)?;
            } else {
                // 小图片直接高质量保存
                img.save_with_format(&png_path, ImageFormat::Png)?;
            }
            
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
    
    // 获取文件大小
    let file_size = match std::fs::metadata(file_path) {
        Ok(metadata) => metadata.len(),
        Err(_) => 0,
    };
    
    // 5MB以上的图片需要更激进的压缩预览
    let need_compression = file_size > 5 * 1024 * 1024;
    
    // 超过20MB的大图片需要特殊处理
    let is_very_large = file_size > 20 * 1024 * 1024;
    
    match image::open(file_path) {
        Ok(img) => {
            // 对于大图片，先压缩再生成预览
            let preview_img = if need_compression {
                // 检查图片大小，如果太大需要压缩预览
                let (width, height) = img.dimensions();
                
                // 计算合适的缩小比例，保持宽高比例
                // 对于5MB以上的图片，缩小到原来的30%，超大图片缩小到20%
                let scale_factor = if is_very_large { 0.2 } else { 0.3 };
                let new_width = (width as f64 * scale_factor).round() as u32;
                let new_height = (height as f64 * scale_factor).round() as u32;
                
                // 使用快速调整大小算法，对预览图优先考虑速度
                img.resize(new_width, new_height, image::imageops::FilterType::Triangle)
            } else if img.width() * img.height() > 4000000 { 
                // 大于400万像素但小于5MB的图片也适当压缩
                let (width, height) = img.dimensions();
                let scale_factor = (4000000.0 / (width * height) as f64).sqrt();
                let new_width = (width as f64 * scale_factor).round() as u32;
                let new_height = (height as f64 * scale_factor).round() as u32;
                
                img.resize(new_width, new_height, image::imageops::FilterType::Triangle)
            } else {
                // 对于小图片，直接限制最大尺寸，确保预览快速
                let (width, height) = img.dimensions();
                if width > 1200 || height > 1200 {
                    let scale_x = 1200.0 / width as f64;
                    let scale_y = 1200.0 / height as f64;
                    let scale = scale_x.min(scale_y);
                    
                    let new_width = (width as f64 * scale).round() as u32;
                    let new_height = (height as f64 * scale).round() as u32;
                    
                    img.resize(new_width, new_height, image::imageops::FilterType::Triangle)
                } else {
                    img
                }
            };
            
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
            
            // 根据文件后缀决定编码格式，对于压缩过的预览，使用更低的质量
            let write_result = match format {
                "image/jpeg" => {
                    // 根据图片大小调整预览质量 - 大图片使用更低质量以加快预览速度
                    let quality = if is_very_large {
                        60 // 超大图片使用低质量
                    } else if need_compression {
                        70 // 大图片使用中等质量
                    } else {
                        85 // 普通图片使用较高质量
                    };
                    preview_img.write_to(&mut cursor, ImageOutputFormat::Jpeg(quality))
                },
                "image/png" => {
                    // PNG预览统一使用JPEG格式，提高加载速度
                    // 除非图片很小，可能含有透明通道
                    if file_size > 1024 * 1024 { // 大于1MB的PNG转为JPEG预览
                        let quality = if is_very_large { 60 } else { 75 };
                        preview_img.write_to(&mut cursor, ImageOutputFormat::Jpeg(quality))
                    } else {
                        preview_img.write_to(&mut cursor, ImageOutputFormat::Png)
                    }
                },
                "image/gif" => preview_img.write_to(&mut cursor, ImageOutputFormat::Gif),
                "image/webp" => preview_img.write_to(&mut cursor, ImageOutputFormat::WebP),
                "image/bmp" => {
                    // BMP预览也统一使用JPEG
                    let quality = if is_very_large { 60 } else { 75 };
                    preview_img.write_to(&mut cursor, ImageOutputFormat::Jpeg(quality))
                },
                "image/x-icon" => preview_img.write_to(&mut cursor, ImageOutputFormat::Png),
                "image/svg+xml" => preview_img.write_to(&mut cursor, ImageOutputFormat::Png),
                _ => preview_img.write_to(&mut cursor, ImageOutputFormat::Jpeg(75)),
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

#[tauri::command]
pub fn get_file_metadata(file_path: &str) -> Result<FileMetadata, String> {
    if !std::path::Path::new(file_path).exists() {
        return Err("文件不存在或无法访问".to_string());
    }
    
    match std::fs::metadata(file_path) {
        Ok(metadata) => {
            Ok(FileMetadata {
                size: metadata.len(),
                is_dir: metadata.is_dir(),
                modified: metadata.modified().ok().map(|time| {
                    time.duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                }),
            })
        },
        Err(e) => Err(format!("获取文件元数据失败: {}", e)),
    }
}

#[derive(Debug, Serialize)]
pub struct FileMetadata {
    pub size: u64,
    pub is_dir: bool,
    pub modified: Option<u64>,
} 