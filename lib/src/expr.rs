use super::object::*;
use super::token::*;
use super::error::*;

pub enum Expr {
    Binary(BinaryExpr)
}

pub trait ExprNode {
    fn accept(&mut self, visitor: &mut dyn ExprVisitor) -> BoxResult<Object>;
}

pub trait ExprVisitor {
    fn visit_binary(&mut self, expr: &mut BinaryExpr) -> BoxResult<Object>;
}

pub struct BinaryExpr {
    pub op: Token,
    pub left: Box<Expr>,
    pub right: Box<Expr>
}

impl ExprNode for BinaryExpr {
    fn accept(&mut self, visitor: &mut dyn ExprVisitor) -> BoxResult<Object> {
        return visitor.visit_binary(self);
    }
}

