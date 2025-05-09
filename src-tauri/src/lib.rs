// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

mod converters;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_upload::init())
        .invoke_handler(tauri::generate_handler![greet])
        .invoke_handler(tauri::generate_handler![converters::json_converter::json_to_go_struct])
        .invoke_handler(tauri::generate_handler![converters::dns_resolver::resolve_dns])
        .invoke_handler(tauri::generate_handler![
            converters::image_converter::convert_image,
            converters::image_converter::get_supported_image_formats,
            converters::image_converter::get_image_info,
            converters::image_converter::copy_file,
            converters::image_converter::get_image_base64
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}