use crate::{domain::Contact, prelude::AppError};

pub trait ContactStore {
    fn load(&self) -> Result<Vec<Contact>, AppError>;
    fn save(&self, contacts: &[Contact]) -> Result<(), AppError>;
}
