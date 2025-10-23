use crate::{domain::Contact, prelude::AppError};

pub trait ContactStore {
    fn load(&self) -> Result<Vec<Contact>, AppError>;
    fn save(&self, contacts: &[Contact]) -> Result<(), AppError>;
    fn search(&self, name: String, domain: String, fuzzy: String) -> Result<Vec<usize>, AppError>;
}
