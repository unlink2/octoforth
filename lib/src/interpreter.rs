use super::parser::*;
use super::stmt::*;
use super::error::*;
use super::expr::*;
use super::object::*;
use super::dictionary::*;
use super::token::*;
use super::builtins::*;
use super::callable::*;

/***
 * This interpreter is responsible
 * for evaluating constants,
 * but it is a fully working forth env
 */
pub struct Interpreter {
    stmts: Vec<Stmt>,
    pub stack: Vec<Object>,
    // contains words and compile-time words
    dictionary: Box<Dictionary>,

    halt: bool
}

impl Interpreter {
    pub fn builtins() -> Box<Dictionary> {
        let mut builtins = Box::new(Dictionary::new());

        builtins.define("+", &Object::Callable(Box::new(Add)));
        builtins.define("-", &Object::Callable(Box::new(Sub)));
        builtins.define("/", &Object::Callable(Box::new(Div)));
        builtins.define("*", &Object::Callable(Box::new(Mul)));
        builtins.define("%", &Object::Callable(Box::new(Mod)));

        builtins.define("&", &Object::Callable(Box::new(And)));
        builtins.define("|", &Object::Callable(Box::new(Or)));
        builtins.define("^", &Object::Callable(Box::new(Xor)));
        builtins.define("~", &Object::Callable(Box::new(Not)));

        builtins.define("drop", &Object::Callable(Box::new(DropTop)));
        builtins.define("dup", &Object::Callable(Box::new(Dup)));

        builtins
    }

    pub fn new(source: &str, path: &str) -> Result<Self, ErrorList> {
        let mut parser = Parser::new(source, path)?;
        let stmts = parser.parse()?;
        Ok(Self {
            stmts,
            dictionary: Self::builtins(),
            stack: vec![],
            halt: false
        })
    }

    pub fn with(stmts: Vec<Stmt>) -> Self {
        Self {
            stmts,
            dictionary: Self::builtins(),
            stack: vec![],
            halt: false
        }
    }

    pub fn pop(&mut self, token: &Token) -> BoxResult<Object> {
        match self.stack.pop() {
            Some(obj) => Ok(obj),
            _ => Err(Box::new(ExecError::new(ErrorType::StackUnderflow, token.clone())))
        }
    }

    pub fn peek(&self, token: &Token) -> BoxResult<Object> {
        match self.stack.last() {
            Some(obj) => Ok(obj.clone()),
            _ => Err(Box::new(ExecError::new(ErrorType::StackUnderflow, token.clone())))
        }
    }

    pub fn push(&mut self, obj: Object) {
        self.stack.push(obj)
    }

    pub fn interprete(&mut self) -> Result<Vec<Compiled>, ErrorList> {
        let mut output = vec![];
        let mut errors = vec![];

        for mut stmt in self.stmts.clone() {
            match self.execute(&mut stmt) {
                Ok(bytes) => {
                    output.push(bytes);
                },
                Err(err) => {
                    errors.push(err);
                    break;
                }
            }

            if self.halt {
                break;
            }
        }

        if errors.len() > 0 {
            return Err(ErrorList::new(errors));
        }

        return Ok(output);
    }

    pub fn execute(&mut self, stmt: &mut Stmt) -> BoxResult<Compiled> {
        stmt.accept(self)
    }

    fn evaluate(&mut self, expr: &mut Expr) -> BoxResult<Object> {
        expr.accept(self)
    }
}

impl StmtVisitor for Interpreter {
    fn visit_expr(&mut self, expr: &mut ExprStmt) -> BoxResult<Compiled> {
        let mut object = self.evaluate(&mut expr.expr)?;

        match &mut object {
            Object::Callable(c) => {
                // call the word in interpreted mode
                return Ok(c.call(self, &expr.expr.token())?);
            },
            Object::Number(n) => {
                // in interpreter mode numbers simply are pushed
                self.stack.push(Object::Number(*n));
                return Ok(Compiled::new(vec![]));
            }
            // TODO support other types at some point!
            _ => return Err(Box::new(ExecError::new(ErrorType::UnsupportedObject, expr.expr.token())))
        };
    }

    fn visit_block(&mut self, block: &mut BlockStmt) -> BoxResult<Compiled> {
        // replace with new env 
        let scope = Box::new(Dictionary::new());
        let prev = std::mem::replace(&mut self.dictionary, scope);
        self.dictionary.parent = Some(prev);

        for stmt in &mut block.body {
            self.execute(stmt)?;
        }
        
        // move env back
        let no_parent = None;
        let parent = std::mem::replace(&mut self.dictionary.parent, no_parent);
        let _ = std::mem::replace(&mut self.dictionary, parent.unwrap());

        Ok(Compiled::new(vec![]))
    }

    fn visit_define(&mut self, def: &mut DefineStmt) -> BoxResult<Compiled> {
        match def.mode {
            _ => self.dictionary.define(&def.name.lexeme,
                &Object::Callable(Box::new(StmtCallable {stmt: *def.body.clone()})))
        }
        Ok(Compiled::new(vec![]))
    }

    fn visit_if(&mut self, stmt: &mut IfStmt) -> BoxResult<Compiled> {
        // if simply checks top of stack
        let value = self.pop(&stmt.token())?;

        if value.truthy() {
            return Ok(self.execute(&mut stmt.then_block)?);
        } else {
            return match &mut stmt.else_block {
                Some(else_stmt) => Ok(self.execute(else_stmt)?),
                _ => Ok(Compiled::new(vec![]))

            };
        }

    }

    fn visit_loop(&mut self, stmt: &mut LoopStmt) -> BoxResult<Compiled> {
        // while top of stack is truthy loop
        while self.peek(&stmt.token())?.truthy() {
            self.execute(&mut stmt.block)?;
        }

        Ok(Compiled::new(vec![]))
    }
}

impl ExprVisitor for Interpreter {
    fn visit_literal(&mut self, expr: &mut LiteralExpr) -> BoxResult<Object> {
        Ok(expr.literal.literal.clone())
    }

    fn visit_word(&mut self, expr: &mut WordExpr) -> BoxResult<Object> {
        self.dictionary.get(&expr.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_call_add() {
        let mut interpreter = Interpreter::new("1 2 +", "").unwrap();
        let _ = interpreter.interprete().unwrap();

        // value on stack should be Some(3)
        assert_eq!(interpreter.stack.len(), 1);
        assert_eq!(interpreter.stack.pop(), Some(Object::Number(3)));
    }

    #[test]
    fn it_should_call_many_words() {
        let mut interpreter = Interpreter::new("1 2 3 + -", "").unwrap();
        let _ = interpreter.interprete().unwrap();

        assert_eq!(interpreter.stack.len(), 1);
        assert_eq!(interpreter.stack.pop(), Some(Object::Number(-4)));
    }

    #[test]
    fn it_should_define_and_call_a_new_word() {
        let mut interpreter = Interpreter::new(": addsub + - ; 1 2 3 addsub", "").unwrap();
        let _ = interpreter.interprete().unwrap();

        assert_eq!(interpreter.stack.len(), 1);
        assert_eq!(interpreter.stack.pop(), Some(Object::Number(-4)));
    }

    #[test]
    fn it_should_use_if() {
        let mut interpreter = Interpreter::new("1 if 2 then", "").unwrap();
        let _ = interpreter.interprete().unwrap();

        assert_eq!(interpreter.stack.len(), 1);
        assert_eq!(interpreter.stack.pop(), Some(Object::Number(2)));
    }

    #[test]
    fn it_should_not_use_if() {
        let mut interpreter = Interpreter::new("0 if 2 then", "").unwrap();
        let _ = interpreter.interprete().unwrap();

        assert_eq!(interpreter.stack.len(), 0);
    }

    #[test]
    fn it_should_skip_else() {
        let mut interpreter = Interpreter::new("1 if 2 else 3 then", "").unwrap();
        let _ = interpreter.interprete().unwrap();

        assert_eq!(interpreter.stack.len(), 1);
        assert_eq!(interpreter.stack.pop(), Some(Object::Number(2)));
    }

    #[test]
    fn it_should_skip_if() {
        let mut interpreter = Interpreter::new("0 if 2 else 3 then", "").unwrap();
        let _ = interpreter.interprete().unwrap();

        assert_eq!(interpreter.stack.len(), 1);
        assert_eq!(interpreter.stack.pop(), Some(Object::Number(3)));
    }

    #[test]
    fn it_should_loop() {
        let mut interpreter = Interpreter::new("10 loop 1 - until", "").unwrap();
        let _ = interpreter.interprete().unwrap();

        assert_eq!(interpreter.stack.len(), 1);
        assert_eq!(interpreter.stack.pop(), Some(Object::Number(0)));
    }

    #[test]
    fn it_should_call_add_and_typeerror() {
        let mut interpreter = Interpreter::new("\"Hi\" 1 +", "").unwrap();
        let errors = match interpreter.interprete() {
            Err(err) => err.errors,
            _ => panic!("Should error!"),
        };

        let errors_id: Vec<String> = errors.iter().map(|x| format!("{:?}", x)).collect();
        assert_eq!(errors_id, vec!["type: UnsupportedObject; lexeme: \"Hi\"".to_string()]);
    }

    #[test]
    fn it_should_call_add_and_underflow() {
        let mut interpreter = Interpreter::new("1 +", "").unwrap();
        let errors = match interpreter.interprete() {
            Err(err) => err.errors,
            _ => panic!("Should error!"),
        };

        let errors_id: Vec<String> = errors.iter().map(|x| format!("{:?}", x)).collect();
        assert_eq!(errors_id, vec!["type: StackUnderflow; lexeme: +".to_string()]);
    }

    #[test]
    fn it_should_cause_division_by_zero() {
        let mut interpreter = Interpreter::new("1 0 /", "").unwrap();
        let errors = match interpreter.interprete() {
            Err(err) => err.errors,
            _ => panic!("Should error!"),
        };

        let errors_id: Vec<String> = errors.iter().map(|x| format!("{:?}", x)).collect();
        assert_eq!(errors_id, vec!["type: DivisionByZero; lexeme: /".to_string()]);
    }
}
