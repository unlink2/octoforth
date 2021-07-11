use super::callable::Callable;
use super::error::*;
use super::token::Token;

pub type ObjStr = String;
pub type ObjNumber = i64;
pub type ObjReal = f64;
pub type ObjList = Vec<Object>;

/// expressions that call a word evaluate to this
/// and call the word with value as __ARG__
#[derive(Debug, Clone)]
pub struct TypedWord {
    pub value: Box<Object>,
    pub word: ObjStr
}

impl TypedWord {
    pub fn new(value: Object, word: &str) -> Self {
        Self {
            value: Box::new(value),
            word: word.into()
        }
    }
}

#[derive(Debug, Clone)]
pub enum Object {
    Null, // internal only
    Nil,
    Number(ObjNumber),
    Real(ObjReal),
    Str(ObjStr),
    Word(ObjStr),
    TypedWord(TypedWord),
    Callable(Box<dyn Callable>),
}

impl Object {
    pub fn truthy(&self) -> bool {
        match self {
            Object::Callable(_) => false,
            Object::Nil | Object::Null => false,
            Object::Number(i) => *i != 0,
            Object::Real(i) => *i != 0.0,
            Object::Str(_) => true,
            Object::Word(_) => true,
            Object::TypedWord(_) => true,
        }
    }

    pub fn nil(&self) -> bool {
        match self {
            Object::Nil => true,
            _ => false
        }
    }

    pub fn null(&self) -> bool {
        match self {
            Object::Null => true,
            _ => false
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Object::Callable(_) => "Callable".into(),
            Object::Nil => "nil".into(),
            Object::Null => "null".into(),
            Object::Number(i) => format!("{}", i),
            Object::Real(i) => format!("{}", i),
            Object::Str(s) => s.clone(),
            Object::Word(w) => w.clone(),
            Object::TypedWord(w) => w.word.clone()
        }
    }

    pub fn mask(&self, mask: ObjNumber, token: &Token) -> BoxResult<Object> {
        match self {
            Object::Number(n) => Ok(Object::Number(n & mask)),
            _ => Err(Box::new(ExecError::new(ErrorType::TypeError, token.clone())))
        }
    }
}

/// for some reason i had to implement PartialEq by hand
/// because of Callable
impl PartialEq for Object {
    fn eq(&self, other: &Object) -> bool {
        match (self, other) {
            (Object::Callable(_), Object::Callable(_)) => false,
            (Object::Nil, Object::Nil) => true,
            (Object::Number(i), Object::Number(j)) => i == j,
            (Object::Real(i), Object::Real(j)) => i == j,
            (Object::Str(i), Object::Str(j)) => i == j,
            (Object::Word(i), Object::Word(j)) => i == j,
            _ => false
        }
    }
}
