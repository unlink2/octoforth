use super::object::*;
use super::token::*;
use super::error::*;

/// An expression is an action that returns
/// an internal compilation object
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(LiteralExpr),
    Word(WordExpr),
    Unary(UnaryExpr)
}

impl ExprNode for Expr {
    fn accept(&mut self, visitor: &mut dyn ExprVisitor) -> BoxResult<Object> {
        match self {
            Self::Literal(literal) => literal.accept(visitor),
            Self::Word(word) => word.accept(visitor),
            Self::Unary(unary) => unary.accept(visitor)
        }
    }

    fn token(&self) -> Token {
        match self {
            Self::Literal(literal) => literal.literal.clone(),
            Self::Word(word) => word.name.clone(),
            Self::Unary(unary) => unary.op.clone()
        }
    }
}

pub trait ExprNode {
    fn accept(&mut self, visitor: &mut dyn ExprVisitor) -> BoxResult<Object>;
    fn token(&self) -> Token {
        Token::new(TokenType::Invalid, Object::Nil, "", 0, 0, "")
    }
}

pub trait ExprVisitor {
    fn visit_literal(&mut self, expr: &mut LiteralExpr) -> BoxResult<Object>;
    fn visit_word(&mut self, expr: &mut WordExpr) -> BoxResult<Object>;
    fn visit_unary(&mut self, expr: &mut UnaryExpr) -> BoxResult<Object>;
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

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
    pub op: Token,
    pub right: Box<Expr>
}

impl UnaryExpr {
    pub fn new(op: Token, right: Box<Expr>) -> Self {
        Self {
            op,
            right
        }
    }
}

impl ExprNode for UnaryExpr {
    fn accept(&mut self, visitor: &mut dyn ExprVisitor) -> BoxResult<Object> {
        return visitor.visit_unary(self);
    }
}
