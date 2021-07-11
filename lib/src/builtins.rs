use super::stmt::*;
use super::error::*;
use super::object::*;
use super::callable::*;
use super::interpreter::*;
use super::token::*;

/**
 * Interpreted builtins
 */

#[derive(Clone)]
pub struct Add;

impl Callable for Add {
    fn call(&mut self, interpreter: &mut Interpreter, token: &Token, _object: &Object) -> BoxResult<Compiled> {
        let y = interpreter.pop(token)?;
        let x = interpreter.pop(token)?;

        match (x, y) {
            (Object::Number(n1), Object::Number(n2)) => interpreter.push(Object::Number(n1 + n2)),
            _ => return Err(Box::new(ExecError::new(ErrorType::TypeError, token.clone())))
        };

        Ok(Compiled::new(vec![]))
    }
}

#[derive(Clone)]
pub struct Sub;

impl Callable for Sub {
    fn call(&mut self, interpreter: &mut Interpreter, token: &Token, _object: &Object) -> BoxResult<Compiled> {
        let y = interpreter.pop(token)?;
        let x = interpreter.pop(token)?;

        match (x, y) {
            (Object::Number(n1), Object::Number(n2)) => interpreter.push(Object::Number(n1 - n2)),
            _ => return Err(Box::new(ExecError::new(ErrorType::TypeError, token.clone())))
        };

        Ok(Compiled::new(vec![]))
    }
}

#[derive(Clone)]
pub struct Mul;

impl Callable for Mul {
    fn call(&mut self, interpreter: &mut Interpreter, token: &Token, _object: &Object) -> BoxResult<Compiled> {
        let y = interpreter.pop(token)?;
        let x = interpreter.pop(token)?;

        match (x, y) {
            (Object::Number(n1), Object::Number(n2)) => interpreter.push(Object::Number(n1 * n2)),
            _ => return Err(Box::new(ExecError::new(ErrorType::TypeError, token.clone())))
        };

        Ok(Compiled::new(vec![]))
    }
}

#[derive(Clone)]
pub struct Div;

impl Callable for Div {
    fn call(&mut self, interpreter: &mut Interpreter, token: &Token, _object: &Object) -> BoxResult<Compiled> {
        let y = interpreter.pop(token)?;
        let x = interpreter.pop(token)?;

        match (x, y) {
            (Object::Number(n1), Object::Number(n2)) => {
                if n2 == 0 {
                    return Err(Box::new(ExecError::new(ErrorType::DivisionByZero, token.clone())));
                }
                interpreter.push(Object::Number(n1 / n2));
            },
            _ => return Err(Box::new(ExecError::new(ErrorType::TypeError, token.clone())))
        };

        Ok(Compiled::new(vec![]))
    }
}

#[derive(Clone)]
pub struct Mod;

impl Callable for Mod {
    fn call(&mut self, interpreter: &mut Interpreter, token: &Token, _object: &Object) -> BoxResult<Compiled> {
        let y = interpreter.pop(token)?;
        let x = interpreter.pop(token)?;

        match (x, y) {
            (Object::Number(n1), Object::Number(n2)) => {
                if n2 == 0 {
                    return Err(Box::new(ExecError::new(ErrorType::DivisionByZero, token.clone())));
                }
                interpreter.push(Object::Number(n1 % n2));
            },
            _ => return Err(Box::new(ExecError::new(ErrorType::TypeError, token.clone())))
        };

        Ok(Compiled::new(vec![]))
    }
}

#[derive(Clone)]
pub struct And;

impl Callable for And {
    fn call(&mut self, interpreter: &mut Interpreter, token: &Token, _object: &Object) -> BoxResult<Compiled> {
        let y = interpreter.pop(token)?;
        let x = interpreter.pop(token)?;

        match (x, y) {
            (Object::Number(n1), Object::Number(n2)) =>
                interpreter.push(Object::Number(n1 & n2)),
            _ => return Err(Box::new(ExecError::new(ErrorType::TypeError, token.clone())))
        };

        Ok(Compiled::new(vec![]))
    }
}

#[derive(Clone)]
pub struct Or;

impl Callable for Or {
    fn call(&mut self, interpreter: &mut Interpreter, token: &Token, _object: &Object) -> BoxResult<Compiled> {
        let y = interpreter.pop(token)?;
        let x = interpreter.pop(token)?;

        match (x, y) {
            (Object::Number(n1), Object::Number(n2)) =>
                interpreter.push(Object::Number(n1 | n2)),
            _ => return Err(Box::new(ExecError::new(ErrorType::TypeError, token.clone())))
        };

        Ok(Compiled::new(vec![]))
    }
}

#[derive(Clone)]
pub struct Not;

impl Callable for Not {
    fn call(&mut self, interpreter: &mut Interpreter, token: &Token, _object: &Object) -> BoxResult<Compiled> {
        let y = interpreter.pop(token)?;

        match y {
            Object::Number(n1) =>
                interpreter.push(Object::Number(!n1)),
            _ => return Err(Box::new(ExecError::new(ErrorType::TypeError, token.clone())))
        };

        Ok(Compiled::new(vec![]))
    }
}

#[derive(Clone)]
pub struct Xor;

impl Callable for Xor {
    fn call(&mut self, interpreter: &mut Interpreter, token: &Token, _object: &Object) -> BoxResult<Compiled> {
        let y = interpreter.pop(token)?;
        let x = interpreter.pop(token)?;

        match (x, y) {
            (Object::Number(n1), Object::Number(n2)) =>
                interpreter.push(Object::Number(n1 ^ n2)),
            _ => return Err(Box::new(ExecError::new(ErrorType::TypeError, token.clone())))
        };

        Ok(Compiled::new(vec![]))
    }
}

#[derive(Clone)]
pub struct Dup;

impl Callable for Dup {
    fn call(&mut self, interpreter: &mut Interpreter, token: &Token, _object: &Object) -> BoxResult<Compiled> {
        let x = interpreter.peek(token)?;

        match x {
            Object::Number(n1) =>
                interpreter.push(Object::Number(n1)),
            _ => return Err(Box::new(ExecError::new(ErrorType::TypeError, token.clone())))
        };

        Ok(Compiled::new(vec![]))
    }
}

#[derive(Clone)]
pub struct DropTop;

impl Callable for DropTop {
    fn call(&mut self, interpreter: &mut Interpreter, token: &Token, _object: &Object) -> BoxResult<Compiled> {
        let _x = interpreter.pop(token)?;

        Ok(Compiled::new(vec![]))
    }
}

#[derive(Clone)]
pub struct Equal;

impl Callable for Equal {
    fn call(&mut self, interpreter: &mut Interpreter, token: &Token, _object: &Object) -> BoxResult<Compiled> {
        let y = interpreter.pop(token)?;
        let x = interpreter.pop(token)?;

        interpreter.push(Object::Number(if y == x { 1 } else { 0 }));

        Ok(Compiled::new(vec![]))
    }
}

#[derive(Clone)]
pub struct NotEqual;

impl Callable for NotEqual {
    fn call(&mut self, interpreter: &mut Interpreter, token: &Token, _object: &Object) -> BoxResult<Compiled> {
        let y = interpreter.pop(token)?;
        let x = interpreter.pop(token)?;

        interpreter.push(Object::Number(if y != x { 1 } else { 0 }));

        Ok(Compiled::new(vec![]))
    }
}

