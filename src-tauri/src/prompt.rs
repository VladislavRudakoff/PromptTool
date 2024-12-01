use serde::{Serialize, Deserialize};

// Структура, представляющая один промпт
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Prompt {
    pub name: String,      // Название промпта
    pub content: String,   // Содержание промпта (шаблон)
    pub parameters: Vec<String>, // Параметры для замены в шаблоне
}

// Структура, представляющая список промптов
#[derive(Serialize, Deserialize, Debug)]
pub struct PromptList {
    pub prompts: Vec<Prompt>,  // Список промптов
}

impl Prompt {
    // Конструктор для создания нового промпта
    pub fn new(name: String, content: String, parameters: Vec<String>) -> Self {
        Prompt {
            name,
            content,
            parameters,
        }
    }
}

