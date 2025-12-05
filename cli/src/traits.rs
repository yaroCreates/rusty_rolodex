use std::collections::HashMap;

use crate::{domain::Contact, prelude::AppError, store::mem::MergePolicy};

pub trait ContactStore {
    fn load(&self) -> Result<HashMap<String, Contact>, AppError>;
    fn save(&self, contacts: HashMap<String, Contact>) -> Result<(), AppError>;
    // fn search(&self, name: String, domain: String, fuzzy: String) -> Result<Vec<usize>, AppError>;
    // fn merge_from_file(&self, other_path: &str, policy: MergePolicy) -> Result<(), AppError>;
}
