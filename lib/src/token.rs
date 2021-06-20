pub enum TokenType {
    Invalid,
    EndOfCommand,
    Eof,
    Comma,
    Slash,
    Regex
}

pub struct Token {
    pub token_type: TokenType,
    pub path: String,
    pub line: usize,
    pub start: usize,
    pub lexeme: String
}

impl Token {
    pub fn new(
        token_type: TokenType,
        lexeme: &str,
        line: usize,
        start: usize,
        path: &str) -> Self {
        Self {
            token_type,
            path: path.into(),
            lexeme: lexeme.into(),
            start,
            line
        }
    }
}
