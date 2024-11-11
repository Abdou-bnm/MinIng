use std::fmt;

// Custom error type for handling lexing errors
#[derive(Debug, PartialEq)]
pub enum CustomError {
    UnrecognizedToken(String),
    InvalidNumberFormat(String),
    IntegerOverflow(String),
    FloatOverflow(String),
    IdentifierTooLong(String),
}
impl From<CustomError> for () {
    fn from(_: CustomError) -> Self {
        ()
    }
}
impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CustomError::UnrecognizedToken(token) => write!(f, "Unrecognized token: {}", token),
            CustomError::InvalidNumberFormat(num) => write!(f, "Invalid number format: {}", num),
            CustomError::IntegerOverflow(num) => write!(f, "Integer overflow: {}", num),
            CustomError::FloatOverflow(num) => write!(f, "Float overflow: {}", num),
            CustomError::IdentifierTooLong(id) => write!(f, "Identifier too long: {}", id),
        }
    }
}
