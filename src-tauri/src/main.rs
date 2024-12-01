#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{State, Manager};
use std::path::PathBuf;
use prompt_tool_lib::{
    file_io::load_prompts,
    prompt::Prompt,
};

// Путь к файлу с промптами по умолчанию
const DEFAULT_PROMPT_FILE: &str = "prompts/default.toml";

/// Структура конфигурации приложения
/// Содержит настройки, которые сохраняются между запусками
#[derive(Debug, Serialize, Deserialize, Clone)]
struct AppConfig {
    // Путь к файлу с промптами
    prompt_file_path: String,
    // Горячая клавиша для быстрого доступа
    hotkey: String,
}

// Реализация значений по умолчанию для конфигурации
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            prompt_file_path: DEFAULT_PROMPT_FILE.to_string(),
            hotkey: String::new(),
        }
    }
}

/// Состояние приложения, которое хранится в памяти
/// Использует Mutex для безопасного доступа из разных потоков
struct AppState {
    config: Mutex<AppConfig>,
}

/// Команда для получения списка промптов
/// Может принимать опциональный путь к файлу
#[tauri::command]
async fn get_prompts(
    file_path: Option<String>,
    state: State<'_, AppState>
) -> Result<Vec<Prompt>, String> {
    // Если путь не указан, берем из конфигурации
    let path = file_path.unwrap_or_else(|| {
        state.config
            .lock()
            .map(|config| config.prompt_file_path.clone())
            .unwrap_or_else(|_| DEFAULT_PROMPT_FILE.to_string())
    });

    // Загружаем и возвращаем промпты
    load_prompts(&path).map_err(|e| format!("Ошибка загрузки промптов: {}", e))
}

/// Команда для установки нового пути к файлу промптов
#[tauri::command]
async fn set_prompt_file_path(
    path: String,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // Проверяем существование файла
    let path_buf = PathBuf::from(&path);
    if !path_buf.exists() {
        return Err("Файл не существует".to_string());
    }

    // Обновляем конфигурацию
    if let Ok(mut config) = state.config.lock() {
        config.prompt_file_path = path;
        
        // Сохраняем обновленную конфигурацию
        if let Some(app_dir) = app_handle.path_resolver().app_config_dir() {
            std::fs::create_dir_all(&app_dir)
                .map_err(|e| format!("Ошибка создания директории конфигурации: {}", e))?;
            
            let config_path = app_dir.join("config.json");
            let config_str = serde_json::to_string_pretty(&*config)
                .map_err(|e| format!("Ошибка сериализации конфигурации: {}", e))?;
            
            std::fs::write(config_path, config_str)
                .map_err(|e| format!("Ошибка записи конфигурации: {}", e))?;
        }
    }

    Ok(())
}

/// Команда для установки новой горячей клавиши
#[tauri::command]
async fn set_hotkey(
    new_hotkey: String,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    if let Ok(mut config) = state.config.lock() {
        config.hotkey = new_hotkey;
        
        // Сохраняем обновленную конфигурацию
        if let Some(app_dir) = app_handle.path_resolver().app_config_dir() {
            std::fs::create_dir_all(&app_dir)
                .map_err(|e| format!("Ошибка создания директории конфигурации: {}", e))?;
            
            let config_path = app_dir.join("config.json");
            let config_str = serde_json::to_string_pretty(&*config)
                .map_err(|e| format!("Ошибка сериализации конфигурации: {}", e))?;
            
            std::fs::write(config_path, config_str)
                .map_err(|e| format!("Ошибка записи конфигурации: {}", e))?;
        }
    }

    Ok(())
}

#[tauri::command]
async fn open_prompt_file_dialog() -> Result<String, String> {
    let (sender, receiver) = tokio::sync::oneshot::channel();

    // Открываем диалог выбора файла
    tauri::async_runtime::spawn(async move {
        FileDialogBuilder::new()
            .add_filter("TOML", &["toml"]) // Фильтр по расширению
            .pick_file(move |file_path| {
                if let Some(path) = file_path {
                    sender.send(Ok(path.to_string_lossy().into_owned())).ok();
                } else {
                    sender.send(Err("Файл не выбран".to_string())).ok();
                }
            });
    });

    // Ждем результата
    receiver.await.map_err(|_| "Ошибка получения пути".to_string())?
}

/// Команда для получения текущей конфигурации
#[tauri::command]
async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    state.config
        .lock()
        .map(|config| config.clone())
        .map_err(|e| format!("Ошибка получения конфигурации: {}", e))
}

/// Инициализация приложения
fn initialize_app(app_handle: &tauri::AppHandle) -> Result<(), String> {
    // Создаем директорию prompts если её нет
    let prompt_dir = PathBuf::from("prompts");
    if !prompt_dir.exists() {
        std::fs::create_dir_all(&prompt_dir)
            .map_err(|e| format!("Ошибка создания директории промптов: {}", e))?;
    }

    // Создаем default.toml если его нет
    let default_file = PathBuf::from(DEFAULT_PROMPT_FILE);
    if !default_file.exists() {
        let default_content = r#"# Файл с промптами по умолчанию
[[prompts]]
name = "Пример промпта"
content = "Это пример промпта с параметром {param1}"
parameters = ["param1"]
"#;
        std::fs::write(&default_file, default_content)
            .map_err(|e| format!("Ошибка создания файла промптов по умолчанию: {}", e))?;
    }

    // Создаем конфигурационный файл если его нет
    if let Some(app_dir) = app_handle.path_resolver().app_config_dir() {
        if !app_dir.exists() {
            std::fs::create_dir_all(&app_dir)
                .map_err(|e| format!("Ошибка создания директории конфигурации: {}", e))?;
        }
        
        let config_path = app_dir.join("config.json");
        if !config_path.exists() {
            let default_config = AppConfig::default();
            let config_str = serde_json::to_string_pretty(&default_config)
                .map_err(|e| format!("Ошибка сериализации конфигурации: {}", e))?;
            std::fs::write(config_path, config_str)
                .map_err(|e| format!("Ошибка записи конфигурации: {}", e))?;
        }
    }

    Ok(())
}

/// Главная функция приложения
fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Инициализируем приложение
            if let Err(e) = initialize_app(&app.handle()) {
                eprintln!("Ошибка инициализации приложения: {}", e);
            }
            Ok(())
        })
        .manage(AppState {
            config: Mutex::new(AppConfig::default()),
        })
        .invoke_handler(tauri::generate_handler![
            get_prompts,
            set_prompt_file_path,
            set_hotkey,
            open_prompt_file_dialog,
            get_config
        ])
        .run(tauri::generate_context!())
        .expect("Ошибка при запуске приложения");
}
