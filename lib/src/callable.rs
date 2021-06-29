use super::compiler::*;
use super::stmt::Compiled;
use super::interpreter::*;
use super::error::*;
use super::token::*;
use std::fmt;
use super::stmt::Stmt;

pub trait CallableClone {
    fn box_clone(&self) -> Box<dyn Callable>;
}

/// Can implement a call and compile method that
/// Interprete should work with a local stack that is implemented using
/// rust
/// compile should output code for the target platform
pub trait Callable: CallableClone {
    fn call(&mut self, _interpreter: &mut Interpreter, _token: &Token) -> BoxResult<Compiled> {
        Ok(Compiled::new(vec![]))
    }

    fn compile(&mut self, _compiler: &mut Compiler, _token: &Token) -> Compiled {
        Compiled::new(vec![])
    }
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

/// callable with body
#[derive(Clone)]
pub struct StmtCallable {
    pub stmt: Stmt
}

impl Callable for StmtCallable {
    fn call(&mut self, interpreter: &mut Interpreter, _token: &Token) -> BoxResult<Compiled> {
        interpreter.execute(&mut self.stmt)?;
        Ok(Compiled::new(vec![]))
    }

    fn compile(&mut self, _compiler: &mut Compiler, _token: &Token) -> Compiled {
        Compiled::new(vec![])
    }
}
