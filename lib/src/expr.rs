use super::object::*;
use super::token::*;
use super::error::*;

#[derive(Clone)]
pub enum Expr {
    List(ListExpr),
    Atom(AtomExpr)
}

pub trait ExprNode {
    fn accept(&mut self, visitor: &mut dyn ExprVisitor) -> BoxResult<Object>;
}

pub trait ExprVisitor {
    fn visit_list(&mut self, expr: &mut ListExpr) -> BoxResult<Object>;
    fn visit_atom(&mut self, expr: &mut AtomExpr) -> BoxResult<Object>;
}

#[derive(Clone)]
pub struct ListExpr {
    pub op: Token,
    pub args: Vec<Box<Expr>>,
}

impl ListExpr {
    pub fn new(op: Token, args: Vec<Box<Expr>>) -> Self {
        Self {
            op,
            args
        }
    }
}

impl ExprNode for ListExpr {
    fn accept(&mut self, visitor: &mut dyn ExprVisitor) -> BoxResult<Object> {
        return visitor.visit_list(self);
    }
}

#[derive(Clone)]
pub struct AtomExpr {
    pub atom: Token
}

impl AtomExpr {
    pub fn new(atom: Token) -> Self {
        Self {
            atom
        }
    }
}

impl ExprNode for AtomExpr {
    fn accept(&mut self, visitor: &mut dyn ExprVisitor) -> BoxResult<Object> {
        return visitor.visit_atom(self);
    }
}
