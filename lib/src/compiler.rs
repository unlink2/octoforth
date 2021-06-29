use super::parser::*;
use super::stmt::*;
use super::error::*;
use super::expr::*;
use super::object::*;
use super::dictionary::*;

pub struct Compiler {
    stmts: Vec<Stmt>,
    // contains words and compile-time words
    dictionary: Box<Dictionary>,

    halt: bool
}

impl Compiler {
    pub fn new(source: &str, path: &str) -> Result<Self, ErrorList> {
        let mut parser = Parser::new(source, path)?;
        let stmts = parser.parse()?;
        Ok(Self {
            stmts,
            dictionary: Box::new(Dictionary::new()),
            halt: false
        })
    }

    pub fn compile(&mut self) -> Result<Vec<Compiled>, ErrorList> {
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

impl StmtVisitor for Compiler {
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

    fn visit_if(&mut self, expr: &mut IfStmt) -> BoxResult<Compiled> {
        panic!();
    }

    fn visit_loop(&mut self, expr: &mut LoopStmt) -> BoxResult<Compiled> {
        panic!();
    }
}

impl ExprVisitor for Compiler {
    fn visit_literal(&mut self, expr: &mut LiteralExpr) -> BoxResult<Object> {
        Ok(expr.literal.literal.clone())
    }

    fn visit_word(&mut self, expr: &mut WordExpr) -> BoxResult<Object> {
        self.dictionary.get(&expr.name)
    }
}
