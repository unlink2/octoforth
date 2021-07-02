use super::error::*;
use super::token::{Token, TokenType};
use super::object::*;
use std::collections::HashMap;

pub struct Scanner {
    source: String,
    path: String,
    keywords: HashMap<String, TokenType>,

    current: usize,
    start: usize,
    line: usize
}

impl Scanner {
    pub fn new(source: &str, path: &str) -> Self {
        let mut keywords = HashMap::new();
        keywords.insert("then".to_string(), TokenType::Then);
        keywords.insert("if".to_string(), TokenType::If);
        keywords.insert("else".to_string(), TokenType::Else);
        keywords.insert("do".to_string(), TokenType::Do);
        keywords.insert("until".to_string(), TokenType::Until);
        keywords.insert("loop".to_string(), TokenType::Loop);
        keywords.insert(":".to_string(), TokenType::StartDefine);
        keywords.insert(";".to_string(), TokenType::EndDefine);
        keywords.insert(":i".to_string(), TokenType::StartInlineDefine);
        keywords.insert(":c".to_string(), TokenType::StartConstDefine);
        keywords.insert(":asm".to_string(), TokenType::Asm);
        keywords.insert(":use".to_string(), TokenType::Use);
        keywords.insert(":mod".to_string(), TokenType::Mod);

        Self {
            source: source.into(),
            path: path.into(),
            keywords,

            current: 0,
            start: 0,
            line: 1
        }
    }

    pub fn scan(&mut self) -> Result<Vec<Token>, ErrorList> {
        let mut tokens = vec![];
        let mut errors = vec![];
        while !self.is_at_end() {
            match self.scan_token() {
                Ok(Some(token)) => tokens.push(token),
                Ok(None) => {},
                Err(err) => errors.push(err)
            }
        }
        self.start = self.current;
        // insert eof token
        tokens.push(Token::new(
            TokenType::EndOfFile,
            Object::Nil,
            "",
            self.line,
            self.start,
            &self.path));

        if errors.len() > 0 {
            return Err(ErrorList::new(errors));
        }

        return Ok(tokens);
    }

    /// Returns either an error
    /// a token or none if the character is ignored
    fn scan_token(&mut self) -> BoxResult<Option<Token>> {
        self.start = self.current;
        let c = self.advance();

        let token = match c {
            ' ' | '\r' | '(' | ')' => return Ok(None),
            '\n' => {
                self.line += 1;
                return Ok(None);
            },
            '#' => {
                // comment until end of line
                while self.peek() != '\n'
                    && !self.is_at_end() {
                    self.advance();
                }
                return Ok(None);
            },
            '"' => match self.scan_str(c) {
                Ok(token) => token,
                Err(err) => return Err(err)
            },
            '\'' => {
                match self.scan_char(c) {
                    Ok(token) => token,
                    Err(err) => return Err(err)
                }
            },
            '\0' => {
                return Ok(None);
            },
            _ => {
                // TODO use pattern range in the future?
                if Self::is_digit(c) {
                    match self.scan_number(c) {
                        Ok(token) => token,
                        Err(err) => return Err(err)
                    }
                } else if Self::is_alpha(c) {
                    // any named token
                    while Scanner::is_alpha_numeric(self.peek()) {
                        self.advance();
                    }
                    let word = self.source[self.start..self.current]
                        .to_string()
                        .clone();

                    let mut token_type = TokenType::Word;

                    // is it keyword?
                    if self.keywords.contains_key(&word) {
                        token_type = self.keywords[&word];
                    }

                    Token::new(
                        token_type,
                        Object::Word(word.clone()),
                        &word,
                        self.line,
                        self.start,
                        &self.path)
                } else {
                    return Err(Box::new(
                            ExecError::new(
                                ErrorType::InvalidToken,
                                Token::new(
                                    TokenType::Invalid,
                                    Object::Nil,
                                    "",
                                    self.line,
                                    self.start,
                                    &self.path))));
                }
            }
        };

        return Ok(Some(token));
    }

    fn scan_char(&mut self, quote: char) -> BoxResult<Token> {
        let character = self.advance();

        let c = if character == '\\' {
            let escaped = Self::unescape_char(character, self.peek());
            if escaped.0 {
                self.advance();
            }
            escaped.1
        } else {
            character
        };

        let end = self.advance();
        if end != quote {
            return Err(Box::new(
                    ExecError::new(
                        ErrorType::UnterminatedString,
                        Token::new(
                            TokenType::Invalid,
                            Object::Nil,
                            "",
                            self.line,
                            self.start,
                            &self.path))));
        }

        return Ok(Token::new(
                TokenType::Number,
                Object::Number(c as ObjNumber),
                &self.source[self.start..self.current].to_string(),
                self.line,
                self.start,
                &self.path));
    }

    fn scan_str(&mut self, quote: char) -> BoxResult<Token> {
        while self.peek() != quote && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }

            // escape
            if self.peek() == '\\' {
                self.advance();
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(Box::new(
                    ExecError::new(
                        ErrorType::UnterminatedString,
                        Token::new(
                            TokenType::Invalid,
                            Object::Nil,
                            "",
                            self.line,
                            self.start,
                            &self.path))));
        }

        // closing quote
        self.advance();

        let unescaped = Scanner::unescape(self.source[self.start+1..self.current-1].to_string());
        let lexeme = Scanner::unescape(self.source[self.start..self.current].to_string());

        return Ok(Token::new(
                TokenType::Str,
                Object::Str(unescaped.clone()),
                &lexeme,
                self.line,
                self.start,
                &self.path));
    }

    fn scan_number(&mut self, c: char) -> BoxResult<Token> {
        // decide if it is hex, binary or decimal
        if c == '0' && self.is_match('x') {
            self.scan_hex(c)
        } else if c == '0' && self.is_match('b') {
            self.scan_bin(c)
        } else {
            self.scan_dec(c)
        }
    }

    fn scan_hex(&mut self, _c: char) -> BoxResult<Token> {
        while Scanner::is_hex(self.peek()) {
            self.advance();
        }
        return self.get_num("0x", 2, TokenType::Number, 16);
    }

    fn scan_bin(&mut self, _c: char) -> BoxResult<Token> {
        while Scanner::is_binary(self.peek()) {
            self.advance();
        }
        return self.get_num("0b", 2, TokenType::Number, 2);
    }

    fn scan_dec(&mut self, _c: char) -> BoxResult<Token> {
        let start_offset = 0;
        let mut token_type = TokenType::Number;
        // decimal
        while Scanner::is_digit(self.peek()) {
            self.advance();
        }

        // is float?
        if self.peek() == '.' && Scanner::is_digit(self.peek_next()) {
            self.advance();
            token_type = TokenType::Real;
            while Scanner::is_digit(self.peek()) {
                self.advance();
            }
        }

        return self.get_num("", start_offset, token_type, 10);
    }

    fn get_num(&mut self, prefix: &str, start_offset: usize, token_type: TokenType, radix: u32) -> BoxResult<Token> {
        // advance to next space, tab or new line
        while (self.peek() != ' '
            && self.peek() != '\t'
            && self.peek() != '\n')
            && !self.is_at_end() {
            self.advance();
        }

        let number = self.source[self.start+start_offset..self.current].to_string().clone();

        let mut lexeme = prefix.to_string();
        lexeme.push_str(&number);

        if token_type == TokenType::Real {
            let num = match Scanner::str_to_real(&number) {
                Some(n) => n,
                _ => {
                    return Err(Box::new(
                            ExecError::new(
                                ErrorType::NumberParseError,
                                Token::new(
                                    TokenType::Invalid,
                                    Object::Nil,
                                    &lexeme,
                                    self.line,
                                    self.start,
                                    &self.path))));
                }
            };

            return Ok(Token::new(
                    token_type,
                    Object::Real(num),
                    &lexeme,
                    self.line,
                    self.start,
                    &self.path));
        } else {
            let num = match Scanner::str_to_num(&number, radix) {
                Some(n) => n,
                _ => {
                    return Err(Box::new(
                            ExecError::new(
                                ErrorType::NumberParseError,
                                Token::new(
                                    TokenType::Invalid,
                                    Object::Nil,
                                    &lexeme,
                                    self.line,
                                    self.start,
                                    &self.path)))); }
            };

            return Ok(Token::new(
                    token_type,
                    Object::Number(num),
                    &lexeme,
                    self.line,
                    self.start,
                    &self.path));
        }
    }

    fn peek(&self) -> char {
        self.source.chars().nth(self.current).unwrap_or('\0')
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        return self.source.chars().nth(self.current + 1).unwrap_or('\0');
    }

    fn is_match(&mut self, c: char) -> bool {
        if self.peek() == c {
            self.advance();
            return true;
        }
        return false;
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source.chars().nth(self.current-1).unwrap_or('\0')
    }

    fn is_at_end(&mut self) -> bool {
        self.current >= self.source.len()
    }

    fn is_alpha(c: char) -> bool {
        return (c >= 'a' && c <= 'z') ||
            (c >= 'A' && c <= 'Z')
            || c == '_'
            || c == '-'
            || c == '+'
            || c == '*'
            || c == '/'
            || c == '%'
            || c == '>'
            || c == '<'
            || c == '='
            || c == ':'
            || c == ';'
            || c == '@'
            || c == '!';
    }

    fn is_digit(c: char) -> bool {
        return c >= '0' && c <= '9';
    }

    fn is_hex(c: char) -> bool {
        return (c >= 'a' && c <= 'f') || (c >= 'A' && c<= 'F') || Scanner::is_digit(c);
    }

    fn is_binary(c: char) -> bool {
        return c == '0' || c == '1';
    }

    fn is_alpha_numeric(c: char) -> bool {
        return Scanner::is_alpha(c) || Scanner::is_digit(c);
    }

    fn unescape_char(c0: char, c1: char) -> (bool, char) {
        if c0 == '\\' {
            match c1 {
                'r' =>  return (true, '\r'),
                't' =>  return (true, '\t'),
                'n' => return (true, '\n'),
                '\\' => return (true, '\\'),
                '"' =>  return (true, '"'),
                '\'' => return (true, '\''),
                '0' => return (true, '\0'),
                _ => return (false, c0)
            }
        }

        return (false, c0);
    }

    fn unescape(input: String) -> String {
        let mut result = "".to_string();

        let mut skip = false;
        for i in 0..input.len() {
            if skip {
                skip = false;
                continue;
            }
            let unescaped = Scanner::unescape_char(
                input.chars().nth(i).unwrap_or('\0'),
                input.chars().nth(i+1).unwrap_or('\0'));

            skip = unescaped.0;
            result = format!("{}{}", result, unescaped.1);
        }

        return result;
    }

    fn str_to_real(s: &str) -> Option<ObjReal> {
        match s.parse::<ObjReal>() {
            Ok(n) => return Some(n),
            _ => return None
        }
    }

    fn str_to_num(s: &str, base: u32) -> Option<ObjNumber> {
        match isize::from_str_radix(&s, base) {
            Ok(n) => return Some(n as ObjNumber),
            _ => return None
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_unescape() {
        let unescaped = Scanner::unescape("Hello \\\"World\\\"\\nTHis.\\tIs\\nAn\\nEscaped\\rString!\\\\".to_string());

        assert_eq!(unescaped, "Hello \"World\"\nTHis.\tIs\nAn\nEscaped\rString!\\");
    }

    #[test]
    fn it_should_detect_digits() {
        for c in 'a'..='z' {
            assert!(Scanner::is_alpha(c));
            assert!(!Scanner::is_digit(c));
            assert!(Scanner::is_alpha_numeric(c));
        }
        for c in 'A'..='Z' {
            assert!(Scanner::is_alpha(c));
            assert!(!Scanner::is_digit(c));
            assert!(Scanner::is_alpha_numeric(c));
        }

        for c in '0'..='9' {
            assert!(!Scanner::is_alpha(c));
            assert!(Scanner::is_digit(c));
            assert!(Scanner::is_alpha_numeric(c));
        }

        // hex
        for c in '0'..='9' {
            assert!(Scanner::is_hex(c));
        }
        for c in 'a'..='f' {
            assert!(Scanner::is_hex(c));
        }
        for c in 'A'..='F' {
            assert!(Scanner::is_hex(c));
        }
        for c in 'g'..='z' {
            assert!(!Scanner::is_hex(c));
        }
        for c in 'G'..='Z' {
            assert!(!Scanner::is_hex(c));
        }

        // bin
        assert!(Scanner::is_binary('0'));
        assert!(Scanner::is_binary('1'));
        assert!(!Scanner::is_binary('2'));
    }

    #[test]
    fn it_should_scan_decimal_numbers() {
        let mut scanner = Scanner::new("123 456", "");

        let tokens = scanner.scan().unwrap();

        assert_eq!(tokens, vec![Token::new(
                        TokenType::Number,
                        Object::Number(123),
                        "123",
                        1,
                        0,
                        ""),
                    Token::new(
                        TokenType::Number,
                        Object::Number(456),
                        "456",
                        1,
                        4,
                        ""),
                    Token::new(
                        TokenType::EndOfFile,
                        Object::Nil,
                        "",
                        1,
                        7,
                        "")]);
    }

    #[test]
    fn it_should_scan_float_numbers() {
        let mut scanner = Scanner::new("3.1415", "");

        let tokens = scanner.scan().unwrap();

        assert_eq!(tokens, vec![Token::new(
                        TokenType::Real,
                        Object::Real(3.1415),
                        "3.1415",
                        1,
                        0,
                        ""),
                    Token::new(
                        TokenType::EndOfFile,
                        Object::Nil,
                        "",
                        1,
                        6,
                        "")]);
    }

    #[test]
    fn it_should_scan_hex_numbers() {
        let mut scanner = Scanner::new("0xa123e", "");

        let tokens = scanner.scan().unwrap();

        assert_eq!(tokens, vec![Token::new(
                        TokenType::Number,
                        Object::Number(0xa123e),
                        "0xa123e",
                        1,
                        0,
                        ""),
                    Token::new(
                        TokenType::EndOfFile,
                        Object::Nil,
                        "",
                        1,
                        7,
                        "")]);
    }

    #[test]
    fn it_should_scan_bin_numbers() {
        let mut scanner = Scanner::new("0b101", "");

        let tokens = scanner.scan().unwrap();

        assert_eq!(tokens, vec![Token::new(
                    TokenType::Number,
                    Object::Number(0b101),
                    "0b101",
                    1,
                    0,
                    ""),
                    Token::new(
                        TokenType::EndOfFile,
                        Object::Nil,
                        "",
                        1,
                        5,
                        "")]);
    }

    #[test]
    fn it_should_scan_words() {
        let mut scanner = Scanner::new("atom if", "");

        let tokens = scanner.scan().unwrap();

        assert_eq!(tokens, vec![Token::new(
                    TokenType::Word,
                    Object::Word("atom".into()),
                    "atom",
                    1,
                    0,
                    ""),
                    Token::new(
                        TokenType::If,
                        Object::Word("if".into()),
                        "if",
                        1,
                        5,
                        ""),
                    Token::new(
                        TokenType::EndOfFile,
                        Object::Nil,
                        "",
                        1,
                        7,
                        "")]);
    }

    #[test]
    fn it_not_should_scan_comments() {
        let mut scanner = Scanner::new("# comment\natom", "");

        let tokens = scanner.scan().unwrap();

        assert_eq!(tokens, vec![Token::new(
                    TokenType::Word,
                    Object::Word("atom".into()),
                    "atom",
                    2,
                    10,
                    ""),
                    Token::new(
                        TokenType::EndOfFile,
                        Object::Nil,
                        "",
                        2,
                        14,
                        "")]);
    }

    #[test]
    fn it_not_should_ignore_parens() {
        let mut scanner = Scanner::new("(\natom)", "");

        let tokens = scanner.scan().unwrap();

        assert_eq!(tokens, vec![Token::new(
                    TokenType::Word,
                    Object::Word("atom".into()),
                    "atom",
                    2,
                    2,
                    ""),
                    Token::new(
                        TokenType::EndOfFile,
                        Object::Nil,
                        "",
                        2,
                        7,
                        "")]);
    }

    #[test]
    fn it_should_scan_strings() {
        let mut scanner = Scanner::new("\"Hello World!\"", "");

        let tokens = scanner.scan().unwrap();

        assert_eq!(tokens, vec![Token::new(
                    TokenType::Str,
                    Object::Str("Hello World!".into()),
                    "\"Hello World!\"",
                    1,
                    0,
                    ""),
                    Token::new(
                        TokenType::EndOfFile,
                        Object::Nil,
                        "",
                        1,
                        14,
                        "")]);
    }

    #[test]
    fn it_should_scan_chars() {
        let mut scanner = Scanner::new("'A'", "");

        let tokens = scanner.scan().unwrap();

        assert_eq!(tokens, vec![Token::new(
                    TokenType::Number,
                    Object::Number('A' as ObjNumber),
                    "'A'",
                    1,
                    0,
                    ""),
                    Token::new(
                        TokenType::EndOfFile,
                        Object::Nil,
                        "",
                        1,
                        3,
                        "")]);
    }

    // failure tests
    #[test]
    fn it_should_not_scan_invalid_decimal_numbers() {
        let mut scanner = Scanner::new("1a23", "");

        let errors = scanner.scan().unwrap_err().errors;

        // get messages
        let errors_id: Vec<String> = errors.iter().map(|x| format!("{:?}", x)).collect();
        assert_eq!(errors_id, vec!["type: NumberParseError; lexeme: 1a23".to_string()]);
    }

    #[test]
    fn it_should_not_scan_invalid_hex_numbers() {
        let mut scanner = Scanner::new("0xag123e", "");

        let errors = scanner.scan().unwrap_err().errors;

        // get messages
        let errors_id: Vec<String> = errors.iter().map(|x| format!("{:?}", x)).collect();
        assert_eq!(errors_id, vec!["type: NumberParseError; lexeme: 0xag123e".to_string()]);
    }

    #[test]
    fn it_should_not_scan_invalid_bin_numbers() {
        let mut scanner = Scanner::new("0b102", "");

        let errors = scanner.scan().unwrap_err().errors;

        // get messages
        let errors_id: Vec<String> = errors.iter().map(|x| format!("{:?}", x)).collect();
        assert_eq!(errors_id, vec!["type: NumberParseError; lexeme: 0b102".to_string()]);
    }

    #[test]
    fn it_should_not_scan_invalid_tokens() {
        let mut scanner = Scanner::new("[", "");

        let errors = scanner.scan().unwrap_err().errors;

        // get messages
        let errors_id: Vec<String> = errors.iter().map(|x| format!("{:?}", x)).collect();
        assert_eq!(errors_id, vec!["type: InvalidToken; lexeme: ".to_string()]);
    }

    #[test]
    fn it_should_not_scan_unterminated_strings() {
        let mut scanner = Scanner::new("\"Hello World!", "");

        let errors = scanner.scan().unwrap_err().errors;

        // get messages
        let errors_id: Vec<String> = errors.iter().map(|x| format!("{:?}", x)).collect();
        assert_eq!(errors_id, vec!["type: UnterminatedString; lexeme: ".to_string()]);
    }

    #[test]
    fn it_should_not_scan_unterminated_char() {
        let mut scanner = Scanner::new("\'A1", "");

        let errors = scanner.scan().unwrap_err().errors;

        // get messages
        let errors_id: Vec<String> = errors.iter().map(|x| format!("{:?}", x)).collect();
        assert_eq!(errors_id, vec!["type: UnterminatedString; lexeme: ".to_string()]);
    }
}
