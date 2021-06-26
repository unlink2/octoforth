use super::object::*;
use super::token::*;
use super::error::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(LiteralExpr),
}

pub trait ExprNode {
    fn accept(&mut self, visitor: &mut dyn ExprVisitor) -> BoxResult<Object>;
}

pub trait ExprVisitor {
    fn visit_literal(&mut self, expr: &mut LiteralExpr) -> BoxResult<Object>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiteralExpr {
    pub literal: Token,
}

impl LiteralExpr {
    pub fn new(literal: Token) -> Self {
        Self {
            literal
        }
    }
}

impl ExprNode for LiteralExpr {
    fn accept(&mut self, visitor: &mut dyn ExprVisitor) -> BoxResult<Object> {
        return visitor.visit_literal(self);
    }
}

