use super::callable::Callable;

pub type ObjStr = String;
pub type ObjNumber = i64;
pub type ObjReal = f64;
pub type ObjList = Vec<Object>;

#[derive(Debug, Clone)]
pub enum Object {
    Nil,
    Number(ObjNumber),
    Real(ObjReal),
    Str(ObjStr),
    Word(ObjStr),
    Callable(Box<dyn Callable>)
}

impl Object {
    pub fn truthy(&self) -> bool {
        match self {
            Object::Callable(_) => false,
            Object::Nil => false,
            Object::Number(i) => *i != 0,
            Object::Real(i) => *i != 0.0,
            Object::Str(_) => true,
            Object::Word(_) => true,
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
