use super::object::*;
use super::error::*;
use std::collections::HashMap;
use super::token::*;

#[derive(Clone)]
pub struct Dictionary {
    pub words: HashMap<String, Object>,
    pub alias: HashMap<String, String>,
    pub parent: Option<Box<Dictionary>>
}

impl Dictionary {
    pub fn new() -> Self {
        Self::with(None)
    }

    pub fn with(parent: Option<Box<Dictionary>>) -> Self {
        Self {
            words: HashMap::new(),
            alias: HashMap::new(),
            parent
        }
    }

    pub fn extend(&mut self, other: &Dictionary) {
        self.words.extend(other.words.clone())
    }

    pub fn get_full_name(name: &str, prefix: &Option<String>) -> String {
        match prefix {
            Some(prefix) =>
            {
                let full_prefix = format!("{}::", prefix);
                if name.starts_with(&full_prefix) {
                    name.to_string()
                } else {
                    format!("{}{}", full_prefix, name)
                }
            },
            _ => name.into()
        }
    }

    pub fn define(&mut self, name: &str, prefix: &Option<String>, value: &Object) {
        let full_name = Self::get_full_name(name, prefix);
        self.words.insert(full_name, value.clone());
    }

    pub fn alias(&mut self, name: &str, prefix: &Option<String>) {
        let full_name = Self::get_full_name(name, prefix);
        self.alias.insert(name.into(), full_name);
    }

    pub fn resolve_full_name(&self, name: &Token, valid: Vec<&Option<String>>, default: &Option<String>) -> String {
        for prefix in valid {
            match self.get_name_and_obj(name, prefix) {
                Ok(obj) => return obj.0,
                _ => {}
            }
        }
        Self::get_full_name(&name.lexeme, default)
    }

    pub fn get_any(&self, name: &Token, valid: Vec<&Option<String>>) -> BoxResult<Object> {
        for prefix in valid {
            match self.get(name, prefix) {
                Ok(obj) => return Ok(obj),
                _ => {}
            }
        }
        return Err(Box::new(ExecError::new(ErrorType::UndefinedWord, name.clone())))
    }

    pub fn get(&self, name: &Token, prefix: &Option<String>) -> BoxResult<Object> {
        Ok(self.get_name_and_obj(name, prefix)?.1)
    }

    fn get_name_and_obj(&self, name: &Token, prefix: &Option<String>) -> BoxResult<(String, Object)> {
        let mut full_name = Self::get_full_name(&name.lexeme, prefix);

        if self.alias.contains_key(&full_name) {
            full_name = self.alias.get(&full_name).unwrap().clone();
        }

        if !self.words.contains_key(&full_name) {
            match &self.parent {
                Some(parent) => return parent.get_name_and_obj(name, prefix),
                _ => return Err(Box::new(ExecError::new(ErrorType::UndefinedWord, name.clone())))
            }
        } else {
            match self.words.get(&full_name) {
                Some(obj) => return Ok((full_name, obj.clone())),
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

        env.define("name", &None, &Object::Number(100));
        assert_eq!(env.get(&Token::new(TokenType::Word, Object::Nil, "name", 0, 0, ""), &None).unwrap(),
            Object::Number(100));
    }

    #[test]
    fn it_should_define_value_with_mod() {
        let mut env = Dictionary::new();

        env.define("name", &Some("module".into()), &Object::Number(100));
        assert_eq!(env.get(&Token::new(TokenType::Word, Object::Nil, "module::name", 0, 0, ""), &None).unwrap(),
            Object::Number(100));
    }

    #[test]
    fn it_should_define_value_with_mod_prefixed() {
        let mut env = Dictionary::new();

        env.define("name", &Some("module".into()), &Object::Number(100));
        assert_eq!(env.get(&Token::new(TokenType::Word, Object::Nil, "name", 0, 0, ""), &Some("module".into()))
            .unwrap(),
            Object::Number(100));
    }

    #[test]
    fn it_should_find_in_parent() {
        let mut parent = Dictionary::new();

        parent.define("name", &None, &Object::Number(100));

        let env = Dictionary::with(Some(Box::new(parent)));

        assert_eq!(env.get(&Token::new(TokenType::Word, Object::Nil, "name", 0, 0, ""), &None).unwrap(),
            Object::Number(100));
    }

    #[test]
    #[should_panic]
    fn it_should_throw_reference_error() {
        let env = Dictionary::new();
        assert_eq!(env.get(&Token::new(TokenType::Word, Object::Nil, "name", 0, 0, ""), &None).unwrap(),
            Object::Number(100));
    }
}
