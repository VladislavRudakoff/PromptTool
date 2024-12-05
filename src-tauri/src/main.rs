#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use tauri_plugin_dialog::DialogExt;
use std::sync::Mutex;
use tauri::State;
use std::path::PathBuf;
use tauri::Manager;
use prompt_tool_lib::{
    file_io::load_prompts,
    prompt::{Prompt, PromptList, SearchFilter},
    error::{Result, PromptToolError},
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
    prompts: Mutex<PromptList>,
}

/// Команда для поиска промптов с фильтрацией
#[tauri::command]
async fn search_prompts(
    filter: SearchFilter,
    state: State<'_, AppState>
) -> Result<Vec<Prompt>> {
    let prompts = state.prompts.lock()
        .map_err(|_| PromptToolError::Config("Не удалось получить доступ к промптам".to_string()))?;
    
    Ok(prompts.search(&filter)
        .into_iter()
        .cloned()
        .collect())
}

/// Команда для получения списка всех категорий
#[tauri::command]
async fn get_categories(
    state: State<'_, AppState>
) -> Result<Vec<String>> {
    let prompts = state.prompts.lock()
        .map_err(|_| PromptToolError::Config("Не удалось получить доступ к промптам".to_string()))?;
    
    Ok(prompts.get_categories()
        .into_iter()
        .cloned()
        .collect())
}

/// Команда для получения списка всех тегов
#[tauri::command]
async fn get_tags(
    state: State<'_, AppState>
) -> Result<Vec<String>> {
    let prompts = state.prompts.lock()
        .map_err(|_| PromptToolError::Config("Не удалось получить доступ к промптам".to_string()))?;
    
    Ok(prompts.get_tags()
        .into_iter()
        .cloned()
        .collect())
}

/// Команда для получения списка промптов
#[tauri::command]
async fn get_prompts(
    file_path: Option<String>,
    state: State<'_, AppState>
) -> Result<Vec<Prompt>> {
    // Если путь не указан, берем из конфигурации
    let path = file_path.unwrap_or_else(|| {
        state.config
            .lock()
            .map(|config| config.prompt_file_path.clone())
            .unwrap_or_else(|_| DEFAULT_PROMPT_FILE.to_string())
    });

    // Загружаем и возвращаем промпты
    let prompt_list = load_prompts(&path)?;
    Ok(prompt_list.prompts)
}

/// Команда для установки нового пути к файлу промптов
#[tauri::command]
async fn set_prompt_file_path(
    path: String,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<()> {
    // Проверяем существование файла
    let path_buf = PathBuf::from(&path);
    if !path_buf.exists() {
        return Err(PromptToolError::Config("Файл не существует".to_string()));
    }

    // Загружаем промпты из нового файла
    let new_prompts = load_prompts(&path)?;
    
    // Обновляем состояние
    if let Ok(mut prompts) = state.prompts.lock() {
        *prompts = new_prompts;
    }

    // Обновляем конфигурацию
    if let Ok(mut config) = state.config.lock() {
        config.prompt_file_path = path;
        
        // Сохраняем обновленную конфигурацию
        let app_dir = app_handle.path().app_config_dir()
            .map_err(|_| PromptToolError::Config("Не удалось получить директорию конфигурации".to_string()))?;
        
        std::fs::create_dir_all(&app_dir)
            .map_err(PromptToolError::Io)?;
        
        let config_path = app_dir.join("config.json");
        let config_str = serde_json::to_string_pretty(&*config)
            .map_err(|_| PromptToolError::Config("Ошибка сериализации конфигурации".to_string()))?;
        
        std::fs::write(config_path, config_str)
            .map_err(PromptToolError::Io)?;
    }

    Ok(())
}

/// Команда для установки новой горячей клавиши
#[tauri::command]
async fn set_hotkey(
    new_hotkey: String,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<()> {
    if let Ok(mut config) = state.config.lock() {
        config.hotkey = new_hotkey;
        
        let app_dir = app_handle.path().app_config_dir()
            .map_err(|_| PromptToolError::Config("Не удалось получить директорию конфигурации".to_string()))?;
        
        std::fs::create_dir_all(&app_dir)
            .map_err(PromptToolError::Io)?;
        
        let config_path = app_dir.join("config.json");
        let config_str = serde_json::to_string_pretty(&*config)
            .map_err(|_| PromptToolError::Config("Ошибка сериализации конфигурации".to_string()))?;
        
        std::fs::write(config_path, config_str)
            .map_err(PromptToolError::Io)?;
    }

    Ok(())
}

#[tauri::command]
async fn open_prompt_file_dialog(app_handle: tauri::AppHandle) -> Result<String> {
    let file_path = app_handle.dialog()
        .file()
        .set_title("Выберите файл с промптами")
        .add_filter("TOML", &["toml"])
        .blocking_pick_file()
        .ok_or_else(|| PromptToolError::Config("Файл не выбран".to_string()))?;

    Ok(file_path.into_path().unwrap().to_string_lossy().into_owned())
}

/// Команда для получения текущей конфигурации
#[tauri::command]
async fn get_config(state: State<'_, AppState>) -> Result<AppConfig> {
    state.config
        .lock()
        .map(|config| config.clone())
        .map_err(|_| PromptToolError::Config("Ошибка получения конфигурации".to_string()))
}

/// Команда для сворачивания окна приложения
#[tauri::command]
async fn minimize_window(window: tauri::Window) {
    if let Err(e) = window.minimize() {
        eprintln!("Ошибка при сворачивании окна: {}", e);
    }
}

/// Инициализация приложения
fn initialize_app(app_handle: &tauri::AppHandle) -> Result<()> {
    // Создаем директорию prompts если её нет
    let prompt_dir = PathBuf::from("prompts");
    if !prompt_dir.exists() {
        std::fs::create_dir_all(&prompt_dir)
            .map_err(PromptToolError::Io)?;
    }

    // Создаем default.toml если его нет
    let default_file = prompt_dir.join("default.toml");
    if !default_file.exists() {
        let default_content = r#"prompts = [
    { name = "Example Prompt", content = "This is an example prompt", parameters = ["param1"] }
]"#;
        std::fs::write(&default_file, default_content)
            .map_err(PromptToolError::Io)?;
    }

    // Создаем конфигурационный файл если его нет
    let app_dir = app_handle.path().app_config_dir()
        .map_err(|_| PromptToolError::Config("Не удалось получить директорию конфигурации".to_string()))?;
    
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir)
            .map_err(PromptToolError::Io)?;
    }
    
    let config_path = app_dir.join("config.json");
    if !config_path.exists() {
        let default_config = AppConfig::default();
        let config_str = serde_json::to_string_pretty(&default_config)
            .map_err(|_| PromptToolError::Config("Ошибка сериализации конфигурации".to_string()))?;
        std::fs::write(config_path, config_str)
            .map_err(PromptToolError::Io)?;
    }

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            initialize_app(&app.handle())?;
            Ok(())
        })
        .manage(AppState {
            config: Mutex::new(AppConfig::default()),
            prompts: Mutex::new(PromptList::new()),
        })
        .invoke_handler(tauri::generate_handler![
            get_prompts,
            set_prompt_file_path,
            set_hotkey,
            open_prompt_file_dialog,
            get_config,
            search_prompts,
            get_categories,
            get_tags,
            minimize_window
        ])
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
