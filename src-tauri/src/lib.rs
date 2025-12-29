// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod converters;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_upload::init())
        // 注意：所有命令必须在同一个 generate_handler! 宏中注册
        // 多次调用 invoke_handler 会覆盖前面的，只有最后一个生效
        .invoke_handler(tauri::generate_handler![
            // JSON 转换
            converters::json_converter::json_to_go_struct,
            // DNS 解析
            converters::dns_resolver::resolve_dns,
            // 图片转换
            converters::image_converter::convert_image,
            converters::image_converter::get_supported_image_formats,
            converters::image_converter::get_image_info,
            converters::image_converter::copy_file,
            converters::image_converter::get_image_base64,
            converters::image_converter::get_file_metadata
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}