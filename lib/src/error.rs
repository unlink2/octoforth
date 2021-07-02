use super::token::Token;
use std::fmt;

pub type BoxResult<T> = Result<T, Box<dyn std::error::Error>>;


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ErrorType {
    InvalidToken,
    UnterminatedString,
    BadNumber,
    NumberParseError,
    UnexpectedToken,
    UnterminatedBlock,
    ExpectedName,
    UndefinedWord,
    UnsupportedObject,
    StackUnderflow,
    TypeError,
    DivisionByZero,
    InvalidString,
    IOError
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(PartialEq)]
pub struct ExecError {
    pub error_type: ErrorType,
    pub token: Token
}

impl ExecError {
    pub fn new(error_type: ErrorType, token: Token) -> Self {
        Self {
            error_type,
            token
        }
    }

    fn to_string(&self) -> &str {
        match self.error_type {
            ErrorType::InvalidToken => "Bad token",
            ErrorType::UnterminatedString => "Unterminated string",
            ErrorType::BadNumber => "Bad number",
            ErrorType::NumberParseError => "Number parser error",
            ErrorType::UnexpectedToken => "Unexpected token",
            ErrorType::UnterminatedBlock => "Unterminated block",
            ErrorType::ExpectedName => "Expected name",
            ErrorType::UndefinedWord => "Undefined word",
            ErrorType::UnsupportedObject => "Object type not supported",
            ErrorType::StackUnderflow => "Stack underflow",
            ErrorType::TypeError => "Type error",
            ErrorType::DivisionByZero => "Division by 0",
            ErrorType::InvalidString => "Invalid string",
            ErrorType::IOError => "IO Error"
        }
    }
}

impl std::error::Error for ExecError {
    fn description(&self) -> &str {
        return self.to_string();
    }
}

impl fmt::Display for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} in {}:{} ({})", self.to_string(), self.token.path, self.token.line, self.token.lexeme)
    }
}

impl fmt::Debug for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "type: {:?}; lexeme: {}", self.error_type, self.token.lexeme)
    }
}

pub struct ErrorList {
    pub errors: Vec<Box<dyn std::error::Error>>
}

impl ErrorList {
    pub fn new(errors: Vec<Box<dyn std::error::Error>>) -> Self {
        Self {
            errors
        }
    }
}

impl std::error::Error for ErrorList {
    fn description(&self) -> &str {
        return "Error list";
    }
}

impl fmt::Display for ErrorList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = "".to_string();
        for err in &self.errors[..] {
            output = format!("{}\n", err);
        }
        write!(f, "{}", output)
    }
}

impl fmt::Debug for ErrorList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error list {:?}", self.errors)
    }
}
