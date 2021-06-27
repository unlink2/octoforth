use super::object::*;
use super::token::*;
use super::error::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(LiteralExpr),
    Word(WordExpr)
}

impl ExprNode for Expr {
    fn accept(&mut self, visitor: &mut dyn ExprVisitor) -> BoxResult<Object> {
        match self {
            Self::Literal(literal) => literal.accept(visitor),
            Self::Word(word) => word.accept(visitor)
        }
    }
}

pub trait ExprNode {
    fn accept(&mut self, visitor: &mut dyn ExprVisitor) -> BoxResult<Object>;
}

pub trait ExprVisitor {
    fn visit_literal(&mut self, expr: &mut LiteralExpr) -> BoxResult<Object>;
    fn visit_word(&mut self, expr: &mut WordExpr) -> BoxResult<Object>;
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

#[derive(Debug, Clone, PartialEq)]
pub struct WordExpr {
    pub name: Token,
}

impl WordExpr {
    pub fn new(name: Token) -> Self {
        Self {
            name
        }
    }
}

impl ExprNode for WordExpr {
    fn accept(&mut self, visitor: &mut dyn ExprVisitor) -> BoxResult<Object> {
        return visitor.visit_word(self);
    }
}
