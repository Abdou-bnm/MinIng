use std::fmt;

// Custom error type for handling lexing errors
#[derive(Debug, PartialEq, Clone, Default)]

pub enum CustomError {
    #[default]
    UnknownError,
    UnrecognizedToken((usize, usize)),
    InvalidNumberFormat(String, (usize, usize)),
    IntegerOverflow(String, (usize, usize)),
    FloatOverflow(String, (usize, usize)),
    IdentifierTooLong(String, (usize, usize)),
    ReDeclaredIdentifier(String, (usize, usize)),
}
impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CustomError::UnknownError => write!(f, "Unknown error"),
            CustomError::UnrecognizedToken((line, column)) => write!(f, "UnrecognizedToken at ({}:{})", column, line),
            CustomError::InvalidNumberFormat(num, (line, column)) => write!(f, "Invalid number format: {} at ({}:{})", num, line, column),
            CustomError::IntegerOverflow(num, (line, column)) => write!(f, "Integer overflow: {} at ({}:{})", num, line, column),
            CustomError::FloatOverflow(num, (line, column)) => write!(f, "Float overflow: {} at ({}:{})", num, line, column),
            CustomError::IdentifierTooLong(id, (line, column)) => write!(f, "Identifier too long: {} at ({}:{})", id, line, column),
            CustomError::ReDeclaredIdentifier(id, (line, column)) => write!(f, "Identifier Already Declared: {} at ({}:{})", id, line, column),
        }
    }
}
