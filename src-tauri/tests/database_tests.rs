#[cfg(test)]
mod tests {
    use prompt_tool_lib::database::{Database, Record};
    use serial_test::serial;
    use tantivy::IndexWriter;
    use tempfile::TempDir;

    fn create_test_database() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::new(temp_dir.path().to_str().unwrap());
        (db, temp_dir)
    }

    fn clear_index(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
        let mut index_writer: IndexWriter = db.index.writer(50_000_000).expect("Failed to create writer");
        index_writer.delete_all_documents()?;
        index_writer.commit()?;
        Ok(())
    }

    #[test]
    #[serial]
    fn test_add_record() {
        let (db, _temp_dir) = create_test_database();
        clear_index(&db).unwrap();

        let record = Record {
            id: 1,
            title: "Test Title".to_string(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            text: "Test text".to_string(),
            created_at: 1000,  // фиксированное время для тестов
            updated_at: 1000,
        };

        let result = db.add_record(record);
        assert!(result.is_ok(), "Failed to add record");
    }

    #[test]
    #[serial]
    fn test_get_record_by_id() {
        let (db, _temp_dir) = create_test_database();
        clear_index(&db).unwrap();

        let record = Record {
            id: 1,
            title: "Test Title".to_string(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            text: "Test text".to_string(),
            created_at: 1000,
            updated_at: 1000,
        };

        db.add_record(record).unwrap();

        let fetched = db.get_record_by_id(1).expect("Failed to fetch record");
        assert!(fetched.is_some(), "Record not found");
        let fetched_record = fetched.unwrap();

        assert_eq!(fetched_record.id, 1);
        assert_eq!(fetched_record.title, "Test Title");
        assert_eq!(fetched_record.tags, vec!["tag1", "tag2"]);
        assert_eq!(fetched_record.text, "Test text");
        assert_eq!(fetched_record.created_at, 1000);
        assert_eq!(fetched_record.updated_at, 1000);
    }

    #[test]
    #[serial]
    fn test_update_record() {
        let (db, _temp_dir) = create_test_database();
        clear_index(&db).unwrap();

        let record = Record {
            id: 1,
            title: "Original Title".to_string(),
            tags: vec!["tag1".to_string()],
            text: "Original text".to_string(),
            created_at: 1000,
            updated_at: 1000,
        };

        db.add_record(record).unwrap();

        db.update_record(1, Some("Updated text"), Some(vec!["tag2".to_string()]))
            .unwrap();

        let fetched = db.get_record_by_id(1).expect("Failed to fetch record");
        assert!(fetched.is_some(), "Record not found");
        let fetched_record = fetched.unwrap();

        assert_eq!(fetched_record.text, "Updated text");
        assert_eq!(fetched_record.tags, vec!["tag2"]);
        assert!(fetched_record.updated_at > 1000, "Updated time should be greater than creation time");
    }

    #[test]
    #[serial]
    fn test_delete_record() {
        let (db, _temp_dir) = create_test_database();
        clear_index(&db).unwrap();

        let record = Record {
            id: 1,
            title: "Test Title".to_string(),
            tags: vec!["tag1".to_string()],
            text: "Test text".to_string(),
            created_at: 1000,
            updated_at: 1000,
        };

        db.add_record(record).unwrap();
        
        // Проверяем, что запись существует
        assert!(db.get_record_by_id(1).unwrap().is_some(), "Record should exist before deletion");
        
        // Удаляем запись
        db.delete_record(1).unwrap();
        
        // Проверяем, что запись удалена
        assert!(db.get_record_by_id(1).unwrap().is_none(), "Record should not exist after deletion");
    }

    #[test]
    #[serial]
    fn test_search() {
        let (db, _temp_dir) = create_test_database();
        clear_index(&db).unwrap();

        let records = vec![
            Record {
                id: 1,
                title: "First Title".to_string(),
                tags: vec!["tag1".to_string()],
                text: "First test text".to_string(),
                created_at: 1000,
                updated_at: 1000,
            },
            Record {
                id: 2,
                title: "Second Title".to_string(),
                tags: vec!["tag2".to_string()],
                text: "Second test text".to_string(),
                created_at: 1000,
                updated_at: 1000,
            },
        ];

        for record in records {
            db.add_record(record).unwrap();
        }

        // Поиск по тексту
        let results = db.search("First").unwrap();
        assert!(!results.is_empty(), "Should find records containing 'First'");
        assert!(results.iter().any(|r| r.contains("First")), "Results should contain 'First'");

        // Поиск по тегам
        let results = db.search("tag2").unwrap();
        assert!(!results.is_empty(), "Should find records with tag2");
        assert!(results.iter().any(|r| r.contains("Second")), "Results should contain record with tag2");
    }

    #[test]
    #[serial]
    fn test_multilanguage_search() {
        let (db, _temp_dir) = create_test_database();
        clear_index(&db).unwrap();

        let records = vec![
            Record {
                id: 1,
                title: "Тестовый заголовок".to_string(),
                tags: vec!["тест".to_string(), "русский".to_string()],
                text: "Это тестовый текст на русском языке".to_string(),
                created_at: 1000,
                updated_at: 1000,
            },
            Record {
                id: 2,
                title: "Mixed language заголовок".to_string(),
                tags: vec!["test".to_string(), "mixed".to_string()],
                text: "This is a mixed текст with русскими словами".to_string(),
                created_at: 1000,
                updated_at: 1000,
            },
        ];

        for record in records {
            db.add_record(record).unwrap();
        }

        // Поиск на русском
        let results = db.search("тестовый").unwrap();
        assert!(!results.is_empty(), "Should find records containing 'тестовый'");
        
        // Поиск смешанного текста
        let results = db.search("mixed русскими").unwrap();
        assert!(!results.is_empty(), "Should find records with mixed language");
        
        // Поиск по тегам на русском
        let results = db.search("русский").unwrap();
        assert!(!results.is_empty(), "Should find records with Russian tags");
    }
}
