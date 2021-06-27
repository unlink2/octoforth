use super::object::*;
use super::error::*;
use std::collections::HashMap;
use super::token::*;

pub struct Dictionary {
    words: HashMap<String, Object>,
    pub parent: Option<Box<Dictionary>>
}

impl Dictionary {
    pub fn new() -> Self {
        Self::with(None)
    }

    pub fn with(parent: Option<Box<Dictionary>>) -> Self {
        Self {
            words: HashMap::new(),
            parent
        }
    }

    pub fn define(&mut self, name: &str, value: &Object) {
        self.words.insert(name.into(), value.clone());
    }

    pub fn get(&self, name: &Token) -> BoxResult<Object> {
        if !self.words.contains_key(&name.lexeme) {
            match &self.parent {
                Some(parent) => return parent.get(name),
                _ => return Err(Box::new(ExecError::new(ErrorType::UndefinedWord, name.clone())))
            }
        } else {
            match self.words.get(&name.lexeme) {
                Some(obj) => return Ok(obj.clone()),
                _ => return Err(Box::new(ExecError::new(ErrorType::UndefinedWord, name.clone())))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_define_value() {
        let mut env = Dictionary::new();

        env.define("name", &Object::Number(100));
        assert_eq!(env.get(&Token::new(TokenType::Word, Object::Nil, "name", 0, 0, "")).unwrap(),
            Object::Number(100));
    }

    #[test]
    fn it_should_find_in_parent() {
        let mut parent = Dictionary::new();

        parent.define("name", &Object::Number(100));

        let env = Dictionary::with(Some(Box::new(parent)));

        assert_eq!(env.get(&Token::new(TokenType::Word, Object::Nil, "name", 0, 0, "")).unwrap(),
            Object::Number(100));
    }

    #[test]
    #[should_panic]
    fn it_should_throw_reference_error() {
        let mut env = Dictionary::new();
        assert_eq!(env.get(&Token::new(TokenType::Word, Object::Nil, "name", 0, 0, "")).unwrap(),
            Object::Number(100));
    }
}
