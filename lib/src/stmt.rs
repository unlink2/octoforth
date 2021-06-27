use super::object::*;
use super::token::*;
use super::error::*;
use super::expr::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(ExprStmt),
    Block(BlockStmt),
    Define(DefineStmt)
}

impl StmtNode for Stmt {
    fn accept(&mut self, visitor: &mut dyn StmtVisitor) -> BoxResult<Vec<u8>> {
        match self {
            Self::Expr(expr) => expr.accept(visitor),
            Self::Block(block) => block.accept(visitor),
            Self::Define(define) => define.accept(visitor)
        }
    }
}

pub trait StmtNode {
    fn accept(&mut self, visitor: &mut dyn StmtVisitor) -> BoxResult<Vec<u8>>;
}

pub trait StmtVisitor {
    fn visit_expr(&mut self, expr: &mut ExprStmt) -> BoxResult<Vec<u8>>;
    fn visit_block(&mut self, expr: &mut BlockStmt) -> BoxResult<Vec<u8>>;
    fn visit_define(&mut self, expr: &mut DefineStmt) -> BoxResult<Vec<u8>>;
}

#[derive(Debug, Clone, PartialEq)]
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
    fn accept(&mut self, visitor: &mut dyn StmtVisitor) -> BoxResult<Vec<u8>> {
        return visitor.visit_expr(self);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockStmt {
    pub body: Vec<Stmt>
}

impl BlockStmt {
    pub fn new(body: Vec<Stmt>) -> Self {
        Self {
            body
        }
    }
}

impl StmtNode for BlockStmt {
    fn accept(&mut self, visitor: &mut dyn StmtVisitor) -> BoxResult<Vec<u8>> {
        return visitor.visit_block(self);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefineStmt {
    pub name: Token,
    pub body: Box<Stmt>,
    pub inline: bool
}

impl DefineStmt {
    pub fn new(name: Token, body: Box<Stmt>, inline: bool) -> Self {
        Self {
            name,
            body,
            inline
        }
    }
}

impl StmtNode for DefineStmt {
    fn accept(&mut self, visitor: &mut dyn StmtVisitor) -> BoxResult<Vec<u8>> {
        return visitor.visit_define(self);
    }
}
