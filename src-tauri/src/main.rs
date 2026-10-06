// DeskSort 后端：扫描桌面目录、按规则分类、持久化手动归类、启动文件
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};
use tauri::{AppHandle, Manager};

#[derive(Serialize, Deserialize, Clone)]
struct CatDef {
    id: String,
    name: String,
    color: String,
}

#[derive(Serialize, Clone)]
struct Item {
    name: String,
    path: String,
    glyph: String,
    cat: String,
}

const COLORS: [&str; 8] = [
    "#5b8cff", "#7c5bff", "#3ddc97", "#ffb547", "#ff6b9d", "#4dd0e1", "#ff8a65", "#a1887f",
];

fn default_cats() -> Vec<CatDef> {
    ["工作文档", "图片", "影音娱乐", "应用程序", "压缩与安装包", "其他"]
        .iter()
        .enumerate()
        .map(|(i, n)| CatDef {
            id: format!("c{}", i),
            name: n.to_string(),
            color: COLORS[i % COLORS.len()].to_string(),
        })
        .collect()
}

/// 按扩展名的默认归类规则
fn classify(path: &std::path::Path) -> String {
    if path.is_dir() {
        return "c5".into();
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "pdf" | "doc" | "docx" | "txt" | "md" | "xls" | "xlsx" | "ppt" | "pptx" | "csv" => "c0",
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "webp" | "ico" => "c1",
        "mp4" | "mkv" | "avi" | "mov" | "mp3" | "flac" | "wav" => "c2",
        "exe" | "lnk" | "app" | "bat" | "msi" | "command" => "c3",
        "zip" | "rar" | "7z" | "tar" | "gz" | "iso" | "dmg" => "c4",
        _ => "c5",
    }
    .to_string()
}

fn glyph_for(path: &std::path::Path) -> &'static str {
    if path.is_dir() {
        return "📁";
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "pdf" => "📕",
        "doc" | "docx" => "📝",
        "txt" | "md" => "📄",
        "xls" | "xlsx" | "csv" => "📊",
        "ppt" | "pptx" => "📽️",
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "ico" => "🖼️",
        "mp4" | "mkv" | "avi" | "mov" => "🎬",
        "mp3" | "flac" | "wav" => "🎵",
        "exe" | "msi" => "⚙️",
        "lnk" => "🔗",
        "app" | "command" => "🚀",
        "zip" | "rar" | "7z" | "tar" | "gz" | "iso" | "dmg" => "📦",
        _ => "📄",
    }
}

fn data_dir(app: &AppHandle) -> PathBuf {
    let dir = app.path().app_data_dir().expect("无法获取应用数据目录");
    fs::create_dir_all(&dir).ok();
    dir
}

fn load_json<T: for<'de> Deserialize<'de>>(app: &AppHandle, name: &str) -> Option<T> {
    fs::read_to_string(data_dir(app).join(name))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

fn save_json(app: &AppHandle, name: &str, v: &impl Serialize) {
    if let Ok(s) = serde_json::to_string_pretty(v) {
        fs::write(data_dir(app).join(name), s).ok();
    }
}

fn load_cats(app: &AppHandle) -> Vec<CatDef> {
    load_json(app, "categories.json").unwrap_or_else(default_cats)
}

fn load_assignments(app: &AppHandle) -> HashMap<String, String> {
    load_json(app, "assignments.json").unwrap_or_default()
}

/// 扫描系统桌面目录，返回 (分类定义, 桌面项目)
#[tauri::command]
fn scan_desktop(app: AppHandle) -> (Vec<CatDef>, Vec<Item>) {
    let cats = load_cats(&app);
    let assign = load_assignments(&app);
    let mut items = Vec::new();

    if let Some(desk) = dirs::desktop_dir() {
        if let Ok(rd) = fs::read_dir(&desk) {
            for entry in rd.flatten() {
                let p = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                let path_str = p.to_string_lossy().to_string();
                let cat = assign
                    .get(&path_str)
                    .cloned()
                    .unwrap_or_else(|| classify(&p));
                items.push(Item {
                    name,
                    path: path_str,
                    glyph: glyph_for(&p),
                    cat,
                });
            }
        }
    }
    items.sort_by(|a, b| a.cat.cmp(&b.cat).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    (cats, items)
}

/// 手动拖拽归类后持久化：path -> 分类 id
#[tauri::command]
fn set_category(app: AppHandle, path: String, cat: String) {
    let mut m = load_assignments(&app);
    m.insert(path, cat);
    save_json(&app, "assignments.json", &m);
}

/// 新建自定义分类
#[tauri::command]
fn add_category(app: AppHandle, name: String) -> CatDef {
    let mut cats = load_cats(&app);
    let id = format!("u{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0));
    let c = CatDef {
        id: id.clone(),
        name,
        color: COLORS[cats.len() % COLORS.len()].to_string(),
    };
    cats.push(c.clone());
    save_json(&app, "categories.json", &cats);
    c
}

/// 用系统默认方式打开文件 / 启动程序
#[tauri::command]
fn launch(path: String) -> Result<(), String> {
    tauri_plugin_opener::open_path(path, None::<&str>).map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            scan_desktop,
            set_category,
            add_category,
            launch
        ])
        .run(tauri::generate_context!())
        .expect("运行 DeskSort 时出错");
}
