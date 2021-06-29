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
    Define(DefineStmt),
    If(IfStmt),
    Loop(LoopStmt)
}

impl StmtNode for Stmt {
    fn accept(&mut self, visitor: &mut dyn StmtVisitor) -> BoxResult<Compiled> {
        match self {
            Self::Expr(expr) => expr.accept(visitor),
            Self::Block(block) => block.accept(visitor),
            Self::Define(define) => define.accept(visitor),
            Self::If(ifstmt) => ifstmt.accept(visitor),
            Self::Loop(loopstmt) => loopstmt.accept(visitor)
        }
    }
}

pub trait StmtNode {
    fn accept(&mut self, visitor: &mut dyn StmtVisitor) -> BoxResult<Compiled>;

    fn token(&self) -> Token {
        Token::new(TokenType::Invalid, Object::Nil, "", 0, 0, "")
    }
}

pub trait StmtVisitor {
    fn visit_expr(&mut self, expr: &mut ExprStmt) -> BoxResult<Compiled>;
    fn visit_block(&mut self, expr: &mut BlockStmt) -> BoxResult<Compiled>;
    fn visit_define(&mut self, expr: &mut DefineStmt) -> BoxResult<Compiled>;
    fn visit_if(&mut self, expr: &mut IfStmt) -> BoxResult<Compiled>;
    fn visit_loop(&mut self, expr: &mut LoopStmt) -> BoxResult<Compiled>;
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
    pub token: Token,
    pub body: Vec<Stmt>
}

impl BlockStmt {
    pub fn new(body: Vec<Stmt>, token: Token) -> Self {
        Self {
            token,
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

#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    pub token: Token,
    pub then_block: Box<Stmt>,
    pub else_block: Option<Box<Stmt>>
}

impl IfStmt {
    pub fn new(then_block: Box<Stmt>, else_block: Option<Box<Stmt>>, token: Token) -> Self {
        Self {
            token,
            then_block,
            else_block
        }
    }
}

impl StmtNode for IfStmt {
    fn accept(&mut self, visitor: &mut dyn StmtVisitor) -> BoxResult<Compiled> {
        return visitor.visit_if(self);
    }

    fn token(&self) -> Token {
        self.token.clone()
    }

}

#[derive(Debug, Clone, PartialEq)]
pub struct LoopStmt {
    pub token: Token,
    pub block: Box<Stmt>,
}

impl LoopStmt {
    pub fn new(block: Box<Stmt>, token: Token) -> Self {
        Self {
            token,
            block,
        }
    }
}

impl StmtNode for LoopStmt {
    fn accept(&mut self, visitor: &mut dyn StmtVisitor) -> BoxResult<Compiled> {
        return visitor.visit_loop(self);
    }

    fn token(&self) -> Token {
        self.token.clone()
    }

}
