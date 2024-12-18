use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use tantivy::collector::TopDocs;
use tantivy::{directory::MmapDirectory,
              doc, query::{QueryParser, TermQuery},
              schema::{IndexRecordOption, OwnedValue, Schema, STORED, TextFieldIndexing, TextOptions, INDEXED},
              Index,
              IndexWriter,
              tokenizer::{LowerCaser, RemoveLongFilter, SimpleTokenizer, Stemmer, TextAnalyzer, TokenizerManager}
};
use tantivy::tokenizer::Language;

/// Структура для представления записи в базе данных.
/// Содержит основные данные, которые хранятся в индексе: название, теги, текст, время создания и редактирования.
#[derive(Debug)]
pub struct Record {
    /// Уникальный идентификатор записи.
    pub id: u64,

    /// Название промпта.
    pub title: String,

    /// Список тегов, по которым можно будет искать запись.
    pub tags: Vec<String>,

    /// Текст самого промпта.
    pub text: String,

    /// Время создания записи в формате UNIX (секунды с эпохи Unix).
    pub created_at: u64,

    /// Время последнего редактирования записи в формате UNIX (секунды с эпохи Unix).
    pub updated_at: u64,
}

/// Структура базы данных, управляющая индексом Tantivy.
/// Эта структура обеспечивает добавление, редактирование, удаление и поиск записей в индексе.
pub struct Database {
    /// Индекс Tantivy для поиска.
    pub index: Index,

    /// Схема, определяющая поля для индекса.
    pub schema: Schema,
}

impl Database {
    /// Создаёт новый экземпляр базы данных, инициализируя индекс на основе указанного пути.
    ///
    /// # Аргументы
    /// * `index_path` - Путь к директории, где будет храниться индекс.
    ///
    /// # Возвращает
    /// Новый экземпляр `Database` с настроенным индексом и схемой.
    pub fn new(index_path: &str) -> Self {
        // Строим схему для индекса
        let mut schema_builder = Schema::builder();

        // Регистрируем токенизаторы
        let tokenizer_manager = TokenizerManager::default();

        // Создаем мультиязычный токенизатор
        let multilang_tokenizer = TextAnalyzer::builder(SimpleTokenizer::default())
            .filter(RemoveLongFilter::limit(40))  // Ограничиваем длину токенов
            .filter(LowerCaser)  // Приводим к нижнему регистру
            .filter(Stemmer::new(Language::Russian))  // Стемминг для русского
            .filter(Stemmer::new(Language::English))  // Стемминг для английского
            .build();

        tokenizer_manager.register("multilang", multilang_tokenizer.clone());

        // Настраиваем индексацию для текстовых полей
        let text_indexing = TextFieldIndexing::default()
            .set_tokenizer("multilang")  // Используем мультиязычный токенизатор
            .set_index_option(IndexRecordOption::WithFreqsAndPositions);

        let text_options = TextOptions::default()
            .set_indexing_options(text_indexing)
            .set_stored();

        let tag_indexing = TextFieldIndexing::default()
            .set_tokenizer("raw")  // Для тегов используем raw токенизатор
            .set_index_option(IndexRecordOption::Basic);

        let tag_options = TextOptions::default()
            .set_indexing_options(tag_indexing)
            .set_stored();

        // Добавляем поля с оптимизированными настройками
        schema_builder.add_u64_field("id", INDEXED | STORED);  // Уникальный идентификатор
        schema_builder.add_text_field("title", text_options.clone());  // Полнотекстовый поиск по заголовку
        schema_builder.add_text_field("tags", tag_options);  // Точный поиск по тегам
        schema_builder.add_text_field("text", text_options);  // Полнотекстовый поиск по содержимому
        schema_builder.add_u64_field("created_at", STORED);  // Только хранение
        schema_builder.add_u64_field("updated_at", STORED);  // Только хранение

        // Строим саму схему
        let schema = schema_builder.build();

        // Применяем токенизатор к индексу
        let index = Index::open_or_create(MmapDirectory::open(Path::new(index_path)).unwrap(), schema.clone()).unwrap();
        index.tokenizers().register("multilang", multilang_tokenizer);

        // Возвращаем структуру базы данных с индексом и схемой
        Database { index, schema }
    }

    /// Добавляет новую запись в индекс базы данных.
    ///
    /// # Аргументы
    /// * `title` - Название записи.
    /// * `tags` - Список тегов для поиска.
    /// * `text` - Текст записи.
    ///
    /// # Описание
    /// Эта функция добавляет новый документ в индекс с указанием времени создания и редактирования.
    pub fn add_record(&self, record: Record) -> Result<(), Box<dyn std::error::Error>> {
        // Создаём writer для записи данных в индекс
        let mut index_writer = self.index.writer(50_000_000).expect("Failed to create writer");

        // Создаём документ для записи в индекс
        let doc = doc!(
            self.schema.get_field("id").unwrap() => record.id,               // Добавляем идентификатор
            self.schema.get_field("title").unwrap() => record.title,         // Добавляем название
            self.schema.get_field("tags").unwrap() => record.tags.join(","), // Добавляем теги как строку
            self.schema.get_field("text").unwrap() => record.text,           // Добавляем текст
            self.schema.get_field("created_at").unwrap() => record.created_at,      // Добавляем время создания
            self.schema.get_field("updated_at").unwrap() => record.updated_at,      // Добавляем время редактирования
        );

        // Добавляем документ в индекс
        index_writer.add_document(doc).expect("Failed to add document");

        // Сохраняем изменения в индексе
        index_writer.commit().expect("Failed to commit changes");
        
        Ok(())
    }

    /// Обновляет существующую запись в индексе.
    ///
    /// # Аргументы
    /// * `id` - Уникальный идентификатор записи.
    /// * `new_text` - Новый текст для обновления.
    /// * `new_tags` - Новый список тегов для обновления.
    ///
    /// # Описание
    /// Эта функция обновляет текст и теги для записи с заданным идентификатором, а также обновляет время редактирования.
    pub fn update_record(&self, id: u64, new_text: Option<&str>, new_tags: Option<Vec<String>>) -> Result<(), Box<dyn std::error::Error>> {
        // Создаём writer для записи данных в индекс
        let mut index_writer = self.index.writer(50_000_000).expect("Failed to create writer");

        // Получаем текущее время для обновления записи
        let updated_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let reader = self.index.reader().expect("Failed to create searcher");
        let searcher = reader.searcher();

        let id_field = self.schema.get_field("id").unwrap();
        let query = TermQuery::new(
            tantivy::Term::from_field_u64(id_field, id),
            tantivy::schema::IndexRecordOption::Basic
        );

        let top_docs = searcher.search(&query, &TopDocs::with_limit(1)).expect("Search failed");
        
        if let Some((_, doc_addr)) = top_docs.first() {
            let retrieved_doc: tantivy::TantivyDocument = searcher.doc(*doc_addr).unwrap();
            
            // Извлекаем существующие значения
            let current_title = retrieved_doc
                .get_first(self.schema.get_field("title").unwrap())
                .and_then(|val| match val {
                    OwnedValue::Str(s) => Some(s.to_string()),
                    _ => None
                })
                .unwrap_or_default();

            let current_tags = retrieved_doc
                .get_first(self.schema.get_field("tags").unwrap())
                .and_then(|val| match val {
                    OwnedValue::Str(s) => Some(s.to_string()),
                    _ => None
                })
                .unwrap_or_default();

            let current_text = retrieved_doc
                .get_first(self.schema.get_field("text").unwrap())
                .and_then(|val| match val {
                    OwnedValue::Str(s) => Some(s.to_string()),
                    _ => None
                })
                .unwrap_or_default();

            let created_at = retrieved_doc
                .get_first(self.schema.get_field("created_at").unwrap())
                .and_then(|val| match val {
                    OwnedValue::U64(t) => Some(t),
                    _ => None
                })
                .unwrap_or(&u64::MIN);

            let tags = new_tags.unwrap_or_else(|| {
                current_tags.split(',')
                    .map(|s| s.to_string())
                    .collect()
            });

            let text = new_text.unwrap_or(&current_text);

            let doc = doc!(
                self.schema.get_field("id").unwrap() => id,
                self.schema.get_field("title").unwrap() => current_title,
                self.schema.get_field("tags").unwrap() => tags.join(","),
                self.schema.get_field("text").unwrap() => text,
                self.schema.get_field("created_at").unwrap() => *created_at,
                self.schema.get_field("updated_at").unwrap() => updated_at
            );

            index_writer.delete_term(tantivy::Term::from_field_u64(id_field, id));
            index_writer.add_document(doc).expect("Failed to add updated document");
            index_writer.commit().expect("Failed to commit changes");
            
            Ok(())
        } else {
            Err("Record not found".into())
        }
    }

    /// Удаляет запись из индекса по её идентификатору.
    ///
    /// # Аргументы
    /// * `id` - Уникальный идентификатор записи, которую нужно удалить.
    ///
    /// # Описание
    /// Эта функция удаляет документ из индекса по заданному идентификатору.
    pub fn delete_record(&self, id: u64) -> Result<(), Box<dyn std::error::Error>> {
        // Создаём writer для записи данных в индекс
        let mut index_writer: IndexWriter = self.index.writer(50_000_000).expect("Failed to create writer");

        let id_field = self.schema.get_field("id").unwrap();

        // Удаление по точному совпадению идентификатора
        index_writer.delete_term(tantivy::Term::from_field_u64(id_field, id));
        index_writer.commit().expect("Failed to commit changes");

        Ok(())
    }

    /// Выполняет поиск по заданному запросу и возвращает 5 первых совпадений.
    ///
    /// # Аргументы
    /// * `query` - Строка поиска, по которой будет выполнен поиск в индексированных полях.
    ///
    /// # Возвращает
    /// Вектор строк, содержащих совпавшие фрагменты текста.
    ///
    /// # Описание
    /// Эта функция выполняет поиск по полям `title`, `text` и `tags` и возвращает 5 первых совпадений.
    pub fn search(&self, query: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // Создаём парсер для запроса по полям title, text и tags
        let query_parser = QueryParser::for_index(&self.index, vec![
            self.schema.get_field("title").unwrap(),  // Поле для поиска в заголовках
            self.schema.get_field("text").unwrap(),   // Поле для поиска в тексте
            self.schema.get_field("tags").unwrap(),   // Поле для поиска по тегам
        ]);

        // Парсим запрос
        let query = query_parser.parse_query(query).expect("Failed to parse query");

        // Создаём объект для поиска
        let searcher = self.index.reader().expect("Failed to create searcher").searcher();

        // Выполняем поиск и получаем 5 лучших совпадений
        let top_docs = searcher.search(&query, &TopDocs::with_limit(5)).expect("Search failed");

        let results = top_docs.into_iter().map(|(_, doc_addr)| {
            let doc: tantivy::TantivyDocument = searcher.doc(doc_addr).unwrap();
            doc.get_first(self.schema.get_field("text").unwrap())
                .and_then(|val| match val {
                    OwnedValue::Str(s) => Some(s.to_string()),
                    _ => None
                })
                .unwrap_or_default()
        }).collect();

        Ok(results)
    }

    /// Получает конкретную запись по её идентификатору.
    ///
    /// # Аргументы
    /// * `id` - Уникальный идентификатор записи, которую нужно получить.
    ///
    /// # Возвращает
    /// `Option<Record>` - Опциональное значение. Если запись найдена, возвращается `Some(Record)`, иначе `None`.
    ///
    /// # Описание
    /// Эта функция выполняет поиск записи по её идентификатору и возвращает соответствующие данные.
    pub fn get_record_by_id(&self, id: u64) -> Result<Option<Record>, Box<dyn std::error::Error>> {
        let reader = self.index.reader().expect("Failed to create searcher");
        let searcher = reader.searcher();

        let id_field = self.schema.get_field("id").unwrap();
        let query = TermQuery::new(
            tantivy::Term::from_field_u64(id_field, id),
            tantivy::schema::IndexRecordOption::Basic
        );

        // Получаем топ-1 результат
        let top_docs = searcher.search(&query, &TopDocs::with_limit(1)).expect("Search failed");
        
        // Если запись найдена, извлекаем её данные
        if let Some((_, doc_addr)) = top_docs.first() {
            let doc: tantivy::TantivyDocument = searcher.doc(*doc_addr)?;

            Ok(Some(Record {
                id,
                title: doc.get_first(self.schema.get_field("title").unwrap())
                    .and_then(|val| match val {
                        OwnedValue::Str(s) => Some(s.to_string()),
                        _ => None
                    })
                    .unwrap_or_default(),
                tags: doc.get_first(self.schema.get_field("tags").unwrap())
                    .and_then(|val| match val {
                        OwnedValue::Str(s) => Some(s.to_string()),
                        _ => None
                    })
                    .unwrap_or_default()
                    .split(',')
                    .map(|s| s.to_string())
                    .filter(|s| !s.is_empty())
                    .collect(),
                text: doc.get_first(self.schema.get_field("text").unwrap())
                    .and_then(|val| match val {
                        OwnedValue::Str(s) => Some(s.to_string()),
                        _ => None
                    })
                    .unwrap_or_default(),
                created_at: *doc.get_first(self.schema.get_field("created_at").unwrap())
                    .and_then(|val| match val {
                        OwnedValue::U64(t) => Some(t),
                        _ => None
                    })
                    .unwrap_or(&u64::MIN),
                updated_at: *doc.get_first(self.schema.get_field("updated_at").unwrap())
                    .and_then(|val| match val {
                        OwnedValue::U64(u) => Some(u),
                        _ => None
                    })
                    .unwrap_or(&u64::MIN),
            }))
        } else {
            Ok(None)
        }
    }
}
