use std::fs;
use std::path::Path;
use std::io::Write;
use crate::prompt::Prompt;
use toml;
use std::fs::File;

/// Функция для загрузки промптов из файла.
pub fn load_prompts(file_path: &str) -> Result<Vec<Prompt>, String> {
    // Проверяем, существует ли файл по указанному пути
    let path = Path::new(file_path);

    if !path.exists() {
        return Err(format!("Файл не найден: {}", file_path));
    }

    // Читаем содержимое файла
    let contents = fs::read_to_string(path)
        .map_err(|e| format!("Ошибка чтения файла {}: {}", file_path, e))?;

    // Проверяем, не пустой ли файл
    if contents.trim().is_empty() {
        return Ok(Vec::new());
    }

    // Преобразуем строку в формат TOML
    let data: toml::Value = toml::de::from_str(&contents)
        .map_err(|e| format!("Ошибка разбора TOML в файле {}: {}", file_path, e))?;

    // Извлекаем промпты из структуры TOML
    let prompts = match data.get("prompts") {
        Some(prompts_value) => {
            match prompts_value.as_array() {
                Some(prompts_array) => {
                    let mut result = Vec::new();
                    for (index, prompt) in prompts_array.iter().enumerate() {
                        let name = prompt.get("name")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| format!("Отсутствует обязательное поле 'name' в промпте #{}", index + 1))?;
                            
                        let content = prompt.get("content")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| format!("Отсутствует обязательное поле 'content' в промпте '{}'", name))?;
                            
                        let parameters = prompt.get("parameters")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect())
                            .unwrap_or_default();

                        result.push(Prompt {
                            name: name.to_string(),
                            content: content.to_string(),
                            parameters,
                        });
                    }
                    result
                },
                None => return Err(format!("Поле 'prompts' в файле {} не является массивом", file_path))
            }
        },
        None => return Err(format!("В файле {} отсутствует обязательное поле 'prompts'", file_path))
    };

    Ok(prompts)
}

/// Функция для сохранения промптов в файл.
pub fn save_prompts(file_path: &str, prompts: &[Prompt]) -> Result<(), String> {
    // Создаем или открываем файл для записи
    let path = Path::new(file_path);
    let mut file = File::create(path)
        .map_err(|e| format!("Ошибка создания файла: {}", e))?;

    // Начинаем формировать строку для TOML
    let mut toml_content = String::from("# Файл с промптами\n\n");
    toml_content.push_str("prompts = [\n");

    // Проходим по всем промптам и добавляем их в строку
    for prompt in prompts {
        let prompt_toml = format!(
            "  # Промпт: {}\n  {{\n    name = {:?},\n    content = {:?},\n    parameters = {:?}\n  }},\n",
            prompt.name,
            prompt.name,
            prompt.content,
            prompt.parameters
        );
        toml_content.push_str(&prompt_toml);
    }

    toml_content.push_str("]\n");

    // Записываем строку в файл
    file.write_all(toml_content.as_bytes())
        .map_err(|e| format!("Ошибка записи в файл: {}", e))?;

    Ok(()) // Возвращаем Ok, если все прошло успешно
}
