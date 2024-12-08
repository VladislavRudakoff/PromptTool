
#[cfg(test)]
mod tests {
    use prompt_tool_lib::database::{Database, Record};
    use serial_test::serial;
    use tempfile::TempDir;

    fn create_test_database() -> Database {
        // Создаем временную директорию для тестов
        let temp_dir = TempDir::new().unwrap();
        
        let temp_path = temp_dir.path();
        
        std::fs::create_dir_all(temp_path).expect("Failed to delete temp dir");
        Database::new(temp_path.to_str().unwrap())
    }

    #[test]
    #[serial]
    fn test_add_record() {
        let db = create_test_database();

        let record = Record {
            id: 1,
            title: "Test Title".to_string(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            text: "Test text".to_string(),
            created_at: 0,
            updated_at: 0,
        };

        let result = db.add_record(record);
        assert!(result.is_ok(), "Failed to add record");
    }

    #[test]
    #[serial]
    fn test_get_record_by_id() {
        let db = create_test_database();

        let record = Record {
            id: 1,
            title: "Test Title".to_string(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            text: "Test text".to_string(),
            created_at: 0,
            updated_at: 0,
        };

        db.add_record(record).unwrap();

        let fetched = db.get_record_by_id(1).expect("Failed to fetch record");
        assert!(fetched.is_some(), "Record not found");
        let fetched_record = fetched.unwrap();

        assert_eq!(fetched_record.id, 1);
        assert_eq!(fetched_record.title, "Test Title");
        assert_eq!(fetched_record.tags, vec!["tag1", "tag2"]);
        assert_eq!(fetched_record.text, "Test text");
    }

    #[test]
    #[serial]
    fn test_update_record() {
        let db = create_test_database();

        let record = Record {
            id: 1,
            title: "Original Title".to_string(),
            tags: vec!["tag1".to_string()],
            text: "Original text".to_string(),
            created_at: 0,
            updated_at: 0,
        };

        db.add_record(record).unwrap();

        db.update_record(1, Some("Updated text"), Some(vec!["tag2".to_string()]))
            .unwrap();

        let fetched = db.get_record_by_id(1).expect("Failed to fetch record");
        assert!(fetched.is_some(), "Record not found");
        let fetched_record = fetched.unwrap();

        assert_eq!(fetched_record.text, "Updated text");
        assert_eq!(fetched_record.tags, vec!["tag2"]);
    }

    #[test]
    #[serial]
    fn test_delete_record() {
        let db = create_test_database();

        let record = Record {
            id: 1,
            title: "Test Title".to_string(),
            tags: vec!["tag1".to_string()],
            text: "Test text".to_string(),
            created_at: 0,
            updated_at: 0,
        };

        db.add_record(record).unwrap();
        db.delete_record(1).unwrap();

        let fetched = db.get_record_by_id(1).expect("Failed to fetch record");
        assert!(fetched.is_none(), "Record was not deleted");
    }

    #[test]
    #[serial]
    fn test_search() {
        let db = create_test_database();

        let record1 = Record {
            id: 1,
            title: "Rust programming".to_string(),
            tags: vec!["tag1".to_string()],
            text: "Learn Rust".to_string(),
            created_at: 0,
            updated_at: 0,
        };

        let record2 = Record {
            id: 2,
            title: "Advanced Rust".to_string(),
            tags: vec!["tag2".to_string()],
            text: "Deep dive into Rust".to_string(),
            created_at: 0,
            updated_at: 0,
        };

        db.add_record(record1).unwrap();
        db.add_record(record2).unwrap();

        let results = db.search("Rust").expect("Failed to perform search");
        assert_eq!(results.len(), 2);
        assert!(results.contains(&"Learn Rust".to_string()));
        assert!(results.contains(&"Deep dive into Rust".to_string()));
    }
}
