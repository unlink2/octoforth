use super::object::Object;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenType {
    Invalid,
    Number,
    Real,
    Str,
    Word,

    If,
    Then,
    Else,
    Begin,
    Until,
    Loop,
    Do,

    Let,
    StartConstDefine,

    StartDefine,
    EndDefine,

    StartInlineDefine,

    // stack push datatype hint
    I8,
    I16,
    I32,
    I64,

    Asm, // :asm "<asm code>"
    Use, // :use "file"
    Mod, // :mod module_name
    Tick, // used to find definition of word

    EndOfFile,
}

#[derive(Debug, PartialEq, Clone)]
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

    /// returns a word attached to this token
    /// usually only used for keywords
    pub fn word(&self) -> &str {
        match self.token_type {
            TokenType::I8 => "push8",
            TokenType::I16 => "push16",
            TokenType::I32 => "push32",
            TokenType::I64 => "push64",
            _ => ""
        }
    }
}
