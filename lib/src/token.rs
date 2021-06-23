use super::object::Object;

#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    Invalid,
    Number,
    Real,
    Str,
    RParen,
    LParen,
    Atom,
    EndOfFile,
    Quote
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub literal: Object,
    pub path: String,
    pub line: usize,
    pub start: usize,
    pub lexeme: String
}

impl Token {
    pub fn new(
        token_type: TokenType,
        literal: Object,
        lexeme: &str,
        line: usize,
        start: usize,
        path: &str) -> Self {
        Self {
            token_type,
            literal,
            path: path.into(),
            lexeme: lexeme.into(),
            start,
            line
        }
    }
}
