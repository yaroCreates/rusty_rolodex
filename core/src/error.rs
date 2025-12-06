#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    Parse(String),
    Validation(String),
    Network(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Io(err) => write!(f, "I/O error: {}", err),
            AppError::Parse(msg) => write!(f, "Parse error: {}", msg),
            AppError::Validation(msg) => write!(f, "Validation failed: {}", msg),
            AppError::Network(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::Network(format!("HTTP error: {}", e))
    }
}
