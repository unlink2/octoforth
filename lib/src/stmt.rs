use super::object::*;
use super::token::*;
use super::error::*;
use super::expr::*;

#[derive(Clone)]
pub enum Stmt {
    ExprStmt(ExprStmt),
}

pub trait StmtNode {
    fn accept(&mut self, visitor: &mut dyn StmtVisitor) -> BoxResult<String>;
}

pub trait StmtVisitor {
    fn visit_expr_stmt(&mut self, expr: &mut ExprStmt) -> BoxResult<String>;
}

#[derive(Clone)]
pub struct ExprStmt {
    pub expr: Expr,
}

impl ExprStmt {
    pub fn new(expr: Expr) -> Self {
        Self {
            expr
        }
    }
}

impl StmtNode for ExprStmt {
    fn accept(&mut self, visitor: &mut dyn StmtVisitor) -> BoxResult<String> {
        return visitor.visit_expr_stmt(self);
    }
}


