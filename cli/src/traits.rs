use crate::{domain::Contact, prelude::AppError, store::mem::MergePolicy};

pub trait ContactStore {
    fn load(&self) -> Result<Vec<Contact>, AppError>;
    fn save(&self, contacts: &[Contact]) -> Result<(), AppError>;
    fn search(&self, name: String, domain: String, fuzzy: String) -> Result<Vec<usize>, AppError>;
    fn merge_from_file(&self, other_path: &str, policy: MergePolicy) -> Result<(), AppError>;
}
