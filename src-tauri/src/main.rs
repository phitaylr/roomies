#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod solver;
mod pdf_generator;

use solver::{solve_from_bytes, SolveResult};
use pdf_generator::generate_pdf;

#[tauri::command]
fn solve_rooms(
    file_data: Vec<u8>, 
    room_size: usize, 
    iterations: usize,
    app: tauri::AppHandle,
) -> Result<SolveResult, String> {
    solve_from_bytes(file_data, room_size, iterations, &app)
}

use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
use std::path::PathBuf;

#[tauri::command]
async fn generate_pdf_report(
    result_json: String,
    event_name: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let result: SolveResult = serde_json::from_str(&result_json)
        .map_err(|e| format!("Failed to parse result: {}", e))?;
    
    // Get downloads folder
    let downloads_dir = app.path()
        .download_dir()
        .map_err(|e| format!("Could not find downloads folder: {}", e))?;
    
    let default_filename = format!("room_assignments_{}.pdf", 
                                  chrono::Local::now().format("%Y%m%d_%H%M%S"));
    
    // Show save dialog
    let file_path = app.dialog()
        .file()
        .set_title("Save Room Assignments PDF")
        .set_file_name(&default_filename)
        .add_filter("PDF", &["pdf"])
        .set_directory(&downloads_dir)
        .blocking_save_file();
    
match file_path {
    Some(path) => {
        // Convert to string and then to Path
        let path_str = path.to_string();
        let path_ref = std::path::Path::new(&path_str);
        generate_pdf(&result, &event_name, &result.people, path_ref)
    }
    None => Err("Save cancelled".to_string())
}
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![solve_rooms, generate_pdf_report])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}