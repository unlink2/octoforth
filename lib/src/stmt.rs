use super::object::*;
use super::token::*;
use super::error::*;
use super::expr::*;

pub struct Compiled {
    data: Vec<u8>
}

impl Compiled {
    pub fn new(data: Vec<u8>) -> Self {
        Self {data}
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(ExprStmt),
    Block(BlockStmt),
    Define(DefineStmt)
}

impl StmtNode for Stmt {
    fn accept(&mut self, visitor: &mut dyn StmtVisitor) -> BoxResult<Compiled> {
        match self {
            Self::Expr(expr) => expr.accept(visitor),
            Self::Block(block) => block.accept(visitor),
            Self::Define(define) => define.accept(visitor)
        }
    }
}

pub trait StmtNode {
    fn accept(&mut self, visitor: &mut dyn StmtVisitor) -> BoxResult<Compiled>;
}

pub trait StmtVisitor {
    fn visit_expr(&mut self, expr: &mut ExprStmt) -> BoxResult<Compiled>;
    fn visit_block(&mut self, expr: &mut BlockStmt) -> BoxResult<Compiled>;
    fn visit_define(&mut self, expr: &mut DefineStmt) -> BoxResult<Compiled>;
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
    fn accept(&mut self, visitor: &mut dyn StmtVisitor) -> BoxResult<Compiled> {
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
    fn accept(&mut self, visitor: &mut dyn StmtVisitor) -> BoxResult<Compiled> {
        return visitor.visit_block(self);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefineStmt {
    pub name: Token,
    pub body: Box<Stmt>,
    pub mode: DefineMode
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum DefineMode {
    Regular, // acts like a function call
    Inline, // is inlined
    Constant // gets evaluated in the interpreter
}

impl DefineStmt {
    pub fn new(name: Token, body: Box<Stmt>, mode: DefineMode) -> Self {
        Self {
            name,
            body,
            mode
        }
    }
}

impl StmtNode for DefineStmt {
    fn accept(&mut self, visitor: &mut dyn StmtVisitor) -> BoxResult<Compiled> {
        return visitor.visit_define(self);
    }
}
