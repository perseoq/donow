use std::fmt;

#[derive(Debug, Clone)]
pub struct DonowError {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

impl DonowError {
    pub fn new(line: usize, col: usize, message: impl Into<String>) -> Self {
        DonowError {
            line,
            col,
            message: message.into(),
        }
    }
}

impl fmt::Display for DonowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.col, self.message)
    }
}

impl std::error::Error for DonowError {}
