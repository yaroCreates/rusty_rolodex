use std::{collections::HashSet, env};

use chrono::Utc;

use crate::{
    domain::{ConflictResolution, Contact},
    error::AppError,
};

pub fn merge_contact_data(local: &Contact, imported: &Contact) -> Contact {
    let mut merged = local.clone();

    // Use the more complete name
    if imported.name.len() > local.name.len() {
        merged.name = imported.name.clone();
    }

    // Use non-empty email, prefer the one from newer contact
    if !imported.email.is_empty()
        && (local.email.is_empty() || imported.updated_at > local.updated_at)
    {
        merged.email = imported.email.clone();
    }

    // Merge phone numbers (unique)
    let mut all_phones: HashSet<String> = local.phone.iter().cloned().collect();
    all_phones.extend(imported.phone.iter().cloned());
    merged.phone = all_phones.into_iter().collect();

    // Merge tags (unique)
    let mut all_tags: HashSet<String> = local.tags.iter().cloned().collect();
    all_tags.extend(imported.tags.iter().cloned());
    merged.tags = all_tags.into_iter().collect();

    // Use earlier created_at
    if imported.created_at < local.created_at {
        merged.created_at = imported.created_at;
    }

    // Use latest updated_at
    merged.updated_at = Utc::now();

    merged
}

pub fn resolve_conflict(local: &Contact, imported: &Contact) -> ConflictResolution {
    // 1. Check timestamps
    if imported.updated_at > local.updated_at {
        // Imported is newer
        if is_more_complete(imported, local) {
            return ConflictResolution::UseImported;
        } else if is_more_complete(local, imported) {
            return ConflictResolution::Merge;
        } else {
            return ConflictResolution::UseImported;
        }
    } else if local.updated_at > imported.updated_at {
        // Local is newer
        if is_more_complete(local, imported) {
            return ConflictResolution::KeepLocal;
        } else if is_more_complete(imported, local) {
            return ConflictResolution::Merge;
        } else {
            return ConflictResolution::KeepLocal;
        }
    }

    // 2. Same timestamp - compare completeness
    if is_more_complete(imported, local) {
        ConflictResolution::UseImported
    } else if is_more_complete(local, imported) {
        ConflictResolution::KeepLocal
    } else {
        // Equal completeness - merge
        ConflictResolution::Merge
    }
}

pub fn completeness_score(contact: &Contact) -> usize {
    let mut score = 0;

    // Check name completeness
    if !contact.name.is_empty() {
        score += 1;
    }
    // if contact.name.contains(' ') {
    //     score += 1; // Has full name
    // }

    // Check email
    if !contact.email.is_empty() {
        score += 2;
    }

    // Phone numbers
    score += contact.phone.len() * 2;

    // Tags
    score += contact.tags.len();

    score
}

pub fn is_more_complete(a: &Contact, b: &Contact) -> bool {
    let a_score = completeness_score(a);
    let b_score = completeness_score(b);
    a_score > b_score
}

pub fn get_key(key: &str) -> Result<String, AppError> {
    env::var(key).map_err(|_e| AppError::Parse("env key not found".to_string()))
}

