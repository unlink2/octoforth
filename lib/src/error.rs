use super::token::Token;
use std::fmt;

pub type BoxResult<T> = Result<T, Box<dyn std::error::Error>>;

pub enum MaybeErrors<T> {
    Results(T),
    Errors(Vec<Box<dyn std::error::Error>>)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ErrorType {
    InvalidToken,
    UnterminatedString,
    BadNumber,
    NumberParseError,
    UnexpectedToken,
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
            _ => "Missing description"
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
        write!(f, "{:?}", self.error_type)
    }
}
