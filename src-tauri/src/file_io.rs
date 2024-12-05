use std::fs;
use std::path::Path;
use std::io::Write;
use crate::prompt::PromptList;
use crate::error::{Result, PromptToolError};
use toml;
use std::fs::File;

/// Функция для загрузки промптов из файла.
pub fn load_prompts(file_path: &str) -> Result<PromptList> {
    // Проверяем, существует ли файл по указанному пути
    let path = Path::new(file_path);

    if !path.exists() {
        return Err(PromptToolError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Файл не найден: {}", file_path),
        )));
    }

    // Читаем содержимое файла
    let contents = fs::read_to_string(path)
        .map_err(PromptToolError::Io)?;

    // Проверяем, не пустой ли файл
    if contents.trim().is_empty() {
        return Ok(PromptList { prompts: Vec::new() });
    }

    // Преобразуем строку в структуру PromptList
    let prompt_list: PromptList = toml::from_str(&contents)
        .map_err(PromptToolError::TomlParse)?;

    Ok(prompt_list)
}

/// Функция для сохранения промптов в файл.
pub fn save_prompts(file_path: &str, prompt_list: &PromptList) -> Result<()> {
    // Сериализуем промпты в TOML
    let toml_string = toml::to_string_pretty(prompt_list)
        .map_err(|e| PromptToolError::Config(format!("Ошибка сериализации: {}", e)))?;

    // Записываем в файл
    let mut file = File::create(file_path)
        .map_err(PromptToolError::Io)?;
    
    file.write_all(toml_string.as_bytes())
        .map_err(PromptToolError::Io)?;

    Ok(())
}
