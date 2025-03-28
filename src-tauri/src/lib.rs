use csv::Reader as csv_reader;
use dirs;
use dotenv::dotenv;
use lazy_static::lazy_static;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::{env, io};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_fs::FilePath;
use tokio::time::{sleep, Duration};

lazy_static! {
    static ref GLOBAL_COPY_COUNTER: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    static ref GLOBAL_TOTAL_COUNTER: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn choice_file(win: tauri::Window) -> Option<FilePath> {
    let path = match env::var("FILTER_FILE_DIR_ABS_PATH") {
        Ok(path) => PathBuf::from(path),
        Err(_) => dirs::desktop_dir().unwrap_or_else(|| PathBuf::from("./")),
    };
    win.dialog().file().set_directory(path).blocking_pick_file()
}

#[tauri::command]
async fn choice_src_dir(win: tauri::Window) -> Option<FilePath> {
    let path = match env::var("SEED_DIR_ABS_PATH") {
        Ok(path) => PathBuf::from(path),
        Err(_) => dirs::desktop_dir().unwrap_or_else(|| PathBuf::from("./")),
    };
    win.dialog()
        .file()
        .set_directory(path)
        .blocking_pick_folder()
}

#[tauri::command]
async fn choice_dst_dir(win: tauri::Window) -> Option<FilePath> {
    let path = match env::var("SAVE_DIR_ABS_PATH") {
        Ok(path) => PathBuf::from(path),
        Err(_) => dirs::desktop_dir().unwrap_or_else(|| PathBuf::from("./")),
    };
    win.dialog()
        .file()
        .set_directory(path)
        .blocking_pick_folder()
}

enum NavFileType {
    NONE,
    CSV,
    TXT,
}

impl PartialEq<&str> for NavFileType {
    fn eq(&self, other: &&str) -> bool {
        match self {
            NavFileType::CSV => *other == "csv",
            NavFileType::TXT => *other == "txt",
            NavFileType::NONE => *other == "none",
        }
    }
}

#[derive(Serialize)]
struct CopyProgressState {
    total_files: usize,
    copied_files: usize,
}

async fn copy_txt_files(
    win: &tauri::Window,
    file_path: &str,
    src_path: &str,
    dst_path: &str,
) -> io::Result<()> {
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);
    let mut library_filename_vec = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(code) => library_filename_vec.push(code),
            Err(_) => continue,
        }
    }
    let mut src_filename_vec = Vec::new();
    if let Ok(entries) = fs::read_dir(src_path) {
        for entry in entries {
            match entry {
                Ok(entry) => {
                    let entry_path = entry.path();
                    if entry_path.is_dir() {
                        if let Some(filename) = entry_path.file_name() {
                            if let Some(filename_str) = filename.to_str() {
                                src_filename_vec.push(filename_str.to_string());
                            }
                        }
                    }
                }
                Err(e) => {}
            }
        }
    }
    let library_filename_set: HashSet<_> = library_filename_vec.into_iter().collect();
    let src_filename_set: HashSet<_> = src_filename_vec.into_iter().collect();

    let find_filename_set: HashSet<_> = library_filename_set
        .intersection(&src_filename_set)
        .collect();
    {
        let mut counter = GLOBAL_TOTAL_COUNTER.lock().unwrap();
        *counter = find_filename_set.len();
    }
    let src_path = Path::new(&src_path);
    let dst_path = Path::new(&dst_path);

    let mut find_filepath_vec = Vec::new();
    for file_dir in find_filename_set.iter() {
        let dst_dir_path = dst_path.join(file_dir);
        if !dst_dir_path.exists() {
            fs::create_dir_all(dst_dir_path)?;
        }
        let src_dir_path = src_path.join(file_dir);
        if src_dir_path.exists() && src_dir_path.is_dir() {
            if let Ok(entries) = fs::read_dir(src_dir_path) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let entry_path = entry.path();
                        if entry_path.is_file() {
                            if let Some(filename) = entry_path.file_name() {
                                if let Some(filename_str) = filename.to_str() {
                                    let src_file_abs_path =
                                        src_path.join(file_dir).join(filename_str);
                                    let dst_file_abs_path =
                                        dst_path.join(file_dir).join(filename_str);
                                    find_filepath_vec.push((src_file_abs_path, dst_file_abs_path));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    find_filepath_vec
        .par_iter()
        .for_each(|(src_file, dst_file)| {
            if src_file.exists() {
                match fs::copy(&src_file, &dst_file) {
                    Ok(_) => {
                        GLOBAL_COPY_COUNTER.fetch_add(1, Relaxed);
                    }
                    Err(_) => {}
                }
            }
        });

    sleep(Duration::from_millis(3000)).await;
    Ok(())
}

#[tauri::command]
async fn start_copy(
    win: tauri::Window,
    choice_file: &str,
    seed_dir: &str,
    target_dir: &str,
) -> Result<(), String> {
    GLOBAL_COPY_COUNTER.store(0, Ordering::SeqCst);
    {
        let mut counter = GLOBAL_TOTAL_COUNTER.lock().unwrap();
        *counter = 0;
    }
    let choice_file_path = Path::new(choice_file);
    let choice_file_path_str = choice_file_path.to_str().unwrap_or_default();
    if !choice_file_path.exists() {
        return Err(format!("`{}`,路径不存在！！！", choice_file_path_str));
    }
    // 获取后缀
    let ext = match choice_file_path.extension() {
        Some(ext) => Some(ext.to_str().unwrap_or_default()),
        None => None,
    };
    let seed_dir_path = Path::new(seed_dir);
    let seed_dir_path_str = seed_dir_path.to_str().unwrap_or_default();
    if !seed_dir_path.exists() {
        return Err(format!("`{}`,路径不存在！！！", seed_dir_path_str));
    }
    let target_dir_path = Path::new(target_dir);
    let target_dir_path_str = target_dir_path.to_str().unwrap_or_default();
    if !target_dir_path.exists() {
        return Err(format!("`{}`,路径不存在!!!", target_dir_path_str));
    }

    // 将后缀转换成enum
    let suffix = match ext {
        Some(ext) => {
            if NavFileType::TXT == ext {
                NavFileType::TXT
            } else if NavFileType::CSV == ext {
                NavFileType::CSV
            } else {
                NavFileType::NONE
            }
        }
        None => NavFileType::NONE,
    };
    match suffix {
        NavFileType::TXT => {
            match copy_txt_files(
                &win,
                choice_file_path_str,
                seed_dir_path_str,
                target_dir_path_str,
            )
            .await
            {
                Ok(_) => {}
                Err(err) => return Err(err.to_string()),
            }
        }
        NavFileType::CSV => {}
        NavFileType::NONE => {
            return Err(format!(
                "`{}`,不支持的文件后缀！！！",
                choice_file_path.to_str().unwrap_or_default()
            ))
        }
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct CopyState {
    pub total: usize,
    pub copied: usize,
    pub rate: usize,
}

#[tauri::command]
async fn copy_process() -> CopyState {
    let file_count = match env::var("PER_DIR_FILE_COUNT") {
        Ok(file_count) => match file_count.parse::<usize>() {
            Ok(count) => count,
            Err(_) => 4,
        },
        Err(_) => 4,
    };
    let total = {
        let count = GLOBAL_TOTAL_COUNTER.lock().unwrap();
        *count
    };
    let copied = GLOBAL_COPY_COUNTER.load(Relaxed) / file_count;
    let rate = copied * 100 / total;
    CopyState {
        total,
        copied,
        rate,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenv().ok();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            choice_file,
            choice_src_dir,
            choice_dst_dir,
            start_copy,
            copy_process,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
