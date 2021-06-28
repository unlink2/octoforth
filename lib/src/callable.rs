use super::compiler::*;
use super::stmt::Compiled;
use std::fmt;

pub trait CallableClone {
    fn box_clone(&self) -> Box<dyn Callable>;
}

pub trait Callable: CallableClone {
    fn call(&mut self, compiler: &mut Compiler) -> Compiled;
}

impl<T> CallableClone for T where T: 'static + Callable + Clone {
    fn box_clone(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }
}

impl PartialEq for Box<dyn Callable> {
    fn eq(&self, _other: &Box<dyn Callable>) -> bool {
        return false;
    }
}

impl fmt::Debug for dyn Callable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ file: {}, line: {}}}", file!(), line!())
    }
}

impl Clone for Box<dyn Callable> {
    fn clone(&self) -> Box<dyn Callable> {
        return self.box_clone();
    }
}


