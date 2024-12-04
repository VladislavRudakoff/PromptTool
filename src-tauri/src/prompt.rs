use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use chrono::{DateTime, Utc};

/// Основная структура для хранения промпта
/// Содержит всю необходимую информацию о промпте, включая метаданные
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Prompt {
    /// Название промпта, используется для быстрой идентификации
    pub name: String,
    
    /// Содержание промпта - сам шаблон текста
    pub content: String,
    
    /// Список параметров, которые можно заменить в шаблоне
    /// Например, если в content есть {param1}, то "param1" должен быть в этом списке
    pub parameters: Vec<String>,
    
    /// Категории, к которым относится промпт
    /// Используется HashSet для быстрого поиска и уникальности категорий
    pub categories: HashSet<String>,
    
    /// Время создания промпта
    /// Если не указано при десериализации, будет использовано текущее время
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    
    /// Время последнего обновления промпта
    /// Автоматически обновляется при любом изменении промпта
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
    
    /// Теги для поиска
    /// Используются для более гибкой категоризации, чем основные категории
    pub tags: HashSet<String>,
}

/// Коллекция промптов
/// Используется для хранения и управления группой промптов
#[derive(Serialize, Deserialize, Debug)]
pub struct PromptList {
    /// Список всех промптов в коллекции
    pub prompts: Vec<Prompt>,
}

impl PromptList {
    /// Создает новую пустую коллекцию промптов
    pub fn new() -> Self {
        Self {
            prompts: Vec::new(),
        }
    }

    /// Поиск промптов по заданному фильтру
    /// Возвращает список промптов, соответствующих критериям поиска
    pub fn search(&self, filter: &SearchFilter) -> Vec<&Prompt> {
        self.prompts
            .iter()
            .filter(|prompt| prompt.matches_filter(filter))
            .collect()
    }

    /// Получает список всех уникальных категорий из всех промптов
    /// Используется для построения UI с фильтрами
    pub fn get_categories(&self) -> HashSet<&String> {
        self.prompts
            .iter()
            .flat_map(|p| p.categories.iter())
            .collect()
    }

    /// Получает список всех уникальных тегов из всех промптов
    /// Используется для построения облака тегов и фильтров
    pub fn get_tags(&self) -> HashSet<&String> {
        self.prompts
            .iter()
            .flat_map(|p| p.tags.iter())
            .collect()
    }
}

/// Структура для фильтрации промптов при поиске
/// Все поля опциональны - если поле None, этот критерий не используется при поиске
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchFilter {
    /// Текстовый поиск по имени и содержимому промпта
    pub query: Option<String>,
    
    /// Фильтр по категориям - промпт должен иметь хотя бы одну из указанных категорий
    pub categories: Option<Vec<String>>,
    
    /// Фильтр по тегам - промпт должен иметь хотя бы один из указанных тегов
    pub tags: Option<Vec<String>>,
    
    /// Начальная дата для фильтрации по времени обновления
    pub date_from: Option<DateTime<Utc>>,
    
    /// Конечная дата для фильтрации по времени обновления
    pub date_to: Option<DateTime<Utc>>,
}

impl Prompt {
    /// Создает новый промпт с указанными параметрами
    /// Автоматически устанавливает текущее время создания и обновления
    pub fn new(
        name: String, 
        content: String, 
        parameters: Vec<String>,
        categories: HashSet<String>,
        tags: HashSet<String>,
    ) -> Self {
        let now = Utc::now();
        Prompt {
            name,
            content,
            parameters,
            categories,
            tags,
            created_at: now,
            updated_at: now,
        }
    }

    /// Обновляет содержимое промпта и его параметры
    /// Автоматически обновляет время последнего изменения
    pub fn update(&mut self, content: String, parameters: Vec<String>) {
        self.content = content;
        self.parameters = parameters;
        self.updated_at = Utc::now();
    }

    /// Добавляет новую категорию к промпту
    /// Если категория уже существует, она не будет добавлена повторно (HashSet)
    pub fn add_category(&mut self, category: String) {
        self.categories.insert(category);
        self.updated_at = Utc::now();
    }

    /// Добавляет новый тег к промпту
    /// Если тег уже существует, он не будет добавлен повторно (HashSet)
    pub fn add_tag(&mut self, tag: String) {
        self.tags.insert(tag);
        self.updated_at = Utc::now();
    }

    /// Проверяет, соответствует ли промпт заданному фильтру поиска
    /// Возвращает true, если промпт соответствует всем заданным критериям
    pub fn matches_filter(&self, filter: &SearchFilter) -> bool {
        // Проверяем текстовый поиск по имени и содержимому
        if let Some(query) = &filter.query {
            let query_lower = query.to_lowercase();
            if !self.name.to_lowercase().contains(&query_lower) &&
               !self.content.to_lowercase().contains(&query_lower) {
                return false;
            }
        }

        // Проверяем соответствие категориям
        if let Some(categories) = &filter.categories {
            if !categories.iter().any(|c| self.categories.contains(c)) {
                return false;
            }
        }

        // Проверяем соответствие тегам
        if let Some(tags) = &filter.tags {
            if !tags.iter().any(|t| self.tags.contains(t)) {
                return false;
            }
        }

        // Проверяем временной диапазон
        if let Some(date_from) = filter.date_from {
            if self.updated_at < date_from {
                return false;
            }
        }

        if let Some(date_to) = filter.date_to {
            if self.updated_at > date_to {
                return false;
            }
        }

        true
    }
}
