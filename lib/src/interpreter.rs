use super::parser::*;
use super::stmt::*;
use super::error::*;
use super::expr::*;
use super::object::*;
use super::dictionary::*;

/***
 * This interpreter is responsible
 * for evaluating constants,
 * but it is a fully working forth env
 */
pub struct Interpreter {
    stmts: Vec<Stmt>,
    stack: Vec<Object>,
    // contains words and compile-time words
    dictionary: Dictionary,

    halt: bool
}

impl Interpreter {
    pub fn new(source: &str, path: &str) -> Result<Self, ErrorList> {
        let mut parser = Parser::new(source, path)?;
        let stmts = parser.parse()?;
        Ok(Self {
            stmts,
            dictionary: Dictionary::new(),
            stack: vec![],
            halt: false
        })
    }

    pub fn with(stmts: Vec<Stmt>) -> Self {
        Self {
            stmts,
            dictionary: Dictionary::new(),
            stack: vec![],
            halt: false
        }
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

    fn execute(&mut self, stmt: &mut Stmt) -> BoxResult<Compiled> {
        stmt.accept(self)
    }

    fn evaluate(&mut self, expr: &mut Expr) -> BoxResult<Object> {
        expr.accept(self)
    }
}

impl StmtVisitor for Interpreter {
    fn visit_expr(&mut self, expr: &mut ExprStmt) -> BoxResult<Compiled> {
        // Ok(self.evaluate(expr.expr)?)
        panic!();
    }

    fn visit_block(&mut self, expr: &mut BlockStmt) -> BoxResult<Compiled> {
        panic!();
    }

    fn visit_define(&mut self, expr: &mut DefineStmt) -> BoxResult<Compiled> {
        panic!();
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
