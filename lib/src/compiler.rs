use super::parser::*;
use super::stmt::*;
use super::error::*;
use super::expr::*;
use super::object::*;

pub struct Compiler {
    stmts: Vec<Stmt>,
    halt: bool
}

impl Compiler {
    pub fn new(source: &str, path: &str) -> Result<Self, ErrorList> {
        let mut parser = Parser::new(source, path)?;
        let stmts = parser.parse()?;
        Ok(Self {
            stmts,
            halt: false
        })
    }

    pub fn compile(&mut self) -> Result<Vec<u8>, ErrorList> {
        let mut output = vec![];
        let mut errors = vec![];

        for mut stmt in self.stmts.clone() {
            match self.execute(&mut stmt) {
                Ok(mut bytes) => {
                    output.append(&mut bytes);
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

    fn execute(&mut self, stmt: &mut Stmt) -> BoxResult<Vec<u8>> {
        stmt.accept(self)
    }

    fn evaluate(&mut self, expr: &mut Expr) -> BoxResult<Object> {
        expr.accept(self)
    }
}

impl StmtVisitor for Compiler {
    fn visit_expr(&mut self, expr: &mut ExprStmt) -> BoxResult<Vec<u8>> {
        // Ok(self.evaluate(expr.expr)?)
        panic!();
    }

    fn visit_block(&mut self, expr: &mut BlockStmt) -> BoxResult<Vec<u8>> {
        panic!();
    }

    fn visit_define(&mut self, expr: &mut DefineStmt) -> BoxResult<Vec<u8>> {
        panic!();
    }

}

impl ExprVisitor for Compiler {
    fn visit_literal(&mut self, expr: &mut LiteralExpr) -> BoxResult<Object> {
        Ok(expr.literal.literal.clone())
    }

    fn visit_word(&mut self, expr: &mut WordExpr) -> BoxResult<Object> {
        panic!();
    }
}
