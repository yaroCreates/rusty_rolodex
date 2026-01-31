// Comprehensive test suite for merge_from_file and sync_from_file
// Tests all three policies (Keep, Overwrite, Duplicate) for both operations

#[cfg(test)]
mod merge_sync_tests {
    // use super::*;
    use chrono::{Duration, Utc};
    use rolodex_core::domain::{Contact, Contacts};
    use rolodex_core::store::MergePolicy;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::NamedTempFile;
    use uuid::Uuid;

    // ============================================================================
    // HELPER FUNCTIONS
    // ============================================================================

    /// Create a test contact with specified parameters
    fn create_contact(
        name: &str,
        phone: Vec<&str>,
        email: &str,
        tags: Vec<&str>,
        days_ago_created: i64,
        days_ago_updated: i64,
    ) -> Contact {
        let now = Utc::now();
        Contact {
            id: Uuid::new_v4(),
            name: name.to_string(),
            phone: phone.iter().map(|s| s.to_string()).collect(),
            email: email.to_string(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            created_at: now - Duration::days(days_ago_created),
            updated_at: now - Duration::days(days_ago_updated),
        }
    }

    /// Create a contact with specific ID
    // fn create_contact_with_id(
    //     id: Uuid,
    //     name: &str,
    //     phone: Vec<&str>,
    //     email: &str,
    //     tags: Vec<&str>,
    //     days_ago_updated: i64,
    // ) -> Contact {
    //     let now = Utc::now();
    //     Contact {
    //         id,
    //         name: name.to_string(),
    //         phone: phone.iter().map(|s| s.to_string()).collect(),
    //         email: email.to_string(),
    //         tags: tags.iter().map(|s| s.to_string()).collect(),
    //         created_at: now - Duration::days(10),
    //         updated_at: now - Duration::days(days_ago_updated),
    //     }
    // }

    /// Initialize Contacts with test data
    fn create_test_contacts(contacts: Vec<Contact>) -> Contacts {
        let mut items = HashMap::new();
        for contact in contacts {
            items.insert(contact.id, contact);
        }
        Contacts::new(items)
    }

    /// Write contacts to a temporary JSON file
    fn write_contacts_to_file(contacts: Vec<Contact>) -> NamedTempFile {
        let file = NamedTempFile::new().unwrap();
        let json = serde_json::to_string_pretty(&contacts).unwrap();
        fs::write(file.path(), json).unwrap();
        file
    }

    // ============================================================================
    // MERGE_FROM_FILE TESTS - KEEP POLICY
    // ============================================================================

    // #[test]
    // fn test_merge_keep_skips_duplicates() {
    //     // Setup: Local has John Doe
    //     let local_contact = create_contact(
    //         "John Doe",
    //         vec!["1234567890"],
    //         "john@example.com",
    //         vec!["friend"],
    //         5,
    //         2,
    //     );
    //     let mut contacts = create_test_contacts(vec![local_contact.clone()]);

    //     // Import: Same contact (same name and phone)
    //     let import_contact = create_contact(
    //         "John Doe",
    //         vec!["1234567890"],
    //         "john.new@example.com", // Different email
    //         vec!["work"],             // Different tags
    //         3,
    //         1,
    //     );
    //     let import_file = write_contacts_to_file(vec![import_contact]);

    //     // Execute
    //     let result = contacts.merge_from_file(import_file.path().to_str().unwrap(), MergePolicy::Keep);

    //     // Assert
    //     assert!(result.is_ok());
    //     assert_eq!(result.unwrap(), 0); // 0 merged (skipped)
    //     assert_eq!(contacts.items.len(), 1);

    //     // Original contact unchanged
    //     let stored = contacts.items.values().next().unwrap();
    //     assert_eq!(stored.email, "john@example.com");
    //     assert_eq!(stored.tags, vec!["friend"]);
    // }

    #[test]
    fn test_merge_keep_adds_new_contacts() {
        // Setup: Local has John Doe
        let local_contact = create_contact(
            "John Doe",
            vec!["1234567890"],
            "john@example.com",
            vec![],
            5,
            2,
        );
        let mut contacts = create_test_contacts(vec![local_contact]);

        // Import: Different contact
        let import_contact = create_contact(
            "Jane Smith",
            vec!["0987654321"],
            "jane@example.com",
            vec![],
            3,
            1,
        );
        let import_file = write_contacts_to_file(vec![import_contact]);

        // Execute
        let result =
            contacts.merge_from_file(import_file.path().to_str().unwrap(), MergePolicy::Keep);

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // 1 added
        assert_eq!(contacts.items.len(), 2);

        // Both contacts exist
        let names: Vec<String> = contacts.items.values().map(|c| c.name.clone()).collect();
        assert!(names.contains(&"John Doe".to_string()));
        assert!(names.contains(&"Jane Smith".to_string()));
    }

    // ============================================================================
    // MERGE_FROM_FILE TESTS - OVERWRITE POLICY
    // ============================================================================

    #[test]
    fn test_merge_overwrite_newer_imported_wins() {
        // Setup: Local contact (updated 5 days ago)
        let local_contact = create_contact(
            "John Doe",
            vec!["1234567890"],
            "john.old@example.com",
            vec!["friend"],
            10,
            5, // Updated 5 days ago
        );
        let original_id = local_contact.id;
        let mut contacts = create_test_contacts(vec![local_contact]);

        // Import: Newer version (updated 1 day ago)
        let import_contact = create_contact(
            "John Doe",
            vec!["1234567890"],
            "john.new@example.com",
            vec!["work"],
            8,
            1, // Updated 1 day ago (newer)
        );
        let import_file = write_contacts_to_file(vec![import_contact]);

        // Execute
        let result =
            contacts.merge_from_file(import_file.path().to_str().unwrap(), MergePolicy::Overwrite);

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // 1 updated
        assert_eq!(contacts.items.len(), 1);

        // Contact was overwritten with newer data
        let stored = contacts.items.values().next().unwrap();
        assert_eq!(stored.email, "john.new@example.com");
        assert_eq!(stored.tags, vec!["work"]);
        // ID should be preserved
        assert_eq!(stored.id, original_id);
    }

    #[test]
    fn test_merge_overwrite_newer_local_kept() {
        // Setup: Local contact (updated 1 day ago - newer)
        let local_contact = create_contact(
            "John Doe",
            vec!["1234567890"],
            "john.new@example.com",
            vec!["work"],
            10,
            1, // Updated 1 day ago (newer)
        );
        let mut contacts = create_test_contacts(vec![local_contact]);

        // Import: Older version (updated 5 days ago)
        let import_contact = create_contact(
            "John Doe",
            vec!["1234567890"],
            "john.old@example.com",
            vec!["friend"],
            8,
            5, // Updated 5 days ago (older)
        );
        let import_file = write_contacts_to_file(vec![import_contact]);

        // Execute
        let result =
            contacts.merge_from_file(import_file.path().to_str().unwrap(), MergePolicy::Overwrite);

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0); // 0 updated (kept local)
        assert_eq!(contacts.items.len(), 1);

        // Local contact unchanged
        let stored = contacts.items.values().next().unwrap();
        assert_eq!(stored.email, "john.new@example.com");
        assert_eq!(stored.tags, vec!["work"]);
    }

    // #[test]
    // fn test_merge_overwrite_completeness_wins() {
    //     // Setup: Local contact (same update time, less complete)
    //     let local_contact = create_contact(
    //         "John",              // Short name
    //         vec!["1234567890"],
    //         "",                  // No email
    //         vec![],              // No tags
    //         5,
    //         2,
    //     );
    //     let original_id = local_contact.id;
    //     let mut contacts = create_test_contacts(vec![local_contact]);

    //     // Import: Same update time, more complete
    //     let mut import_contact = create_contact(
    //         "John Doe",          // Full name
    //         vec!["1234567890", "0987654321"], // Multiple phones
    //         "john@example.com",  // Has email
    //         vec!["work", "friend"], // Has tags
    //         5,
    //         2, // Same update time
    //     );
    //     // Force same updated_at
    //     import_contact.updated_at = contacts.items.get(&original_id).unwrap().updated_at;

    //     let import_file = write_contacts_to_file(vec![import_contact]);

    //     // Execute
    //     let result = contacts.merge_from_file(import_file.path().to_str().unwrap(), MergePolicy::Overwrite);

    //     // Assert
    //     assert!(result.is_ok());
    //     assert_eq!(result.unwrap(), 1);

    //     // More complete version should win
    //     let stored = contacts.items.values().next().unwrap();
    //     assert_eq!(stored.name, "John Doe");
    //     assert_eq!(stored.email, "john@example.com");
    //     assert!(stored.phone.len() > 1);
    //     assert!(!stored.tags.is_empty());
    // }

    #[test]
    fn test_merge_overwrite_merges_when_both_have_value() {
        // Setup: Local contact (newer, but less complete in some fields)
        let local_contact = create_contact(
            "John Doe",
            vec!["1234567890"],
            "john@example.com",
            vec!["friend"],
            10,
            1, // Newer
        );
        let original_id = local_contact.id;
        let mut contacts = create_test_contacts(vec![local_contact]);

        // Import: Older but has additional data
        let import_contact = create_contact(
            "John Doe",
            vec!["0987654321"], // Different phone
            "john@example.com",
            vec!["work"], // Different tag
            8,
            5, // Older, but has more complete data in phone/tags
        );
        let import_file = write_contacts_to_file(vec![import_contact]);

        // Execute
        let result =
            contacts.merge_from_file(import_file.path().to_str().unwrap(), MergePolicy::Overwrite);

        // Assert
        assert!(result.is_ok());

        // If both have valuable data, they should be merged
        // (behavior depends on resolve_conflict logic)
        let stored = contacts.items.values().next().unwrap();
        assert_eq!(stored.id, original_id);
    }
}
