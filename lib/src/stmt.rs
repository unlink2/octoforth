use super::object::*;
use super::token::*;
use super::error::*;
use super::expr::*;
use std::str;

/// a statement instruction the compiler to
/// perform an action and returns the resulting code
#[derive(Clone)]
pub struct Compiled {
    pub data: Vec<u8>
}

impl Compiled {
    pub fn flatten(data: Vec<Compiled>) -> BoxResult<String> {
        let mut result = "".to_string();
        for d in data {
            if d.data.len() > 0 {
                result = format!("{}{}\n", result, str::from_utf8(&d.data)?.to_string());
            }

        }

        return Ok(result);
    }

    pub fn flatten_bytes(data: &mut Vec<Compiled>) -> Compiled {
        let mut result = Compiled::new(vec![]);
        for d in data {
            result.data.append(&mut d.data);
        }
        return result;
    }

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
    Loop(LoopStmt),
    Import(ImportStmt),
    Use(UseStmt),
    Asm(AsmStmt),
    Mod(ModStmt),
    Tick(TickStmt)
}

impl StmtNode for Stmt {
    fn accept(&mut self, visitor: &mut dyn StmtVisitor) -> BoxResult<Compiled> {
        match self {
            Self::Expr(expr) => expr.accept(visitor),
            Self::Block(block) => block.accept(visitor),
            Self::Define(define) => define.accept(visitor),
            Self::If(ifstmt) => ifstmt.accept(visitor),
            Self::Loop(loopstmt) => loopstmt.accept(visitor),
            Self::Import(stmt) => stmt.accept(visitor),
            Self::Mod(modstmt) => modstmt.accept(visitor),
            Self::Asm(asmstmt) => asmstmt.accept(visitor),
            Self::Tick(tickstmt) => tickstmt.accept(visitor),
            Self::Use(stmt) => stmt.accept(visitor)
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
    fn visit_expr(&mut self, stmt: &mut ExprStmt) -> BoxResult<Compiled>;
    fn visit_block(&mut self, stmt: &mut BlockStmt) -> BoxResult<Compiled>;
    fn visit_define(&mut self, stmt: &mut DefineStmt) -> BoxResult<Compiled>;
    fn visit_if(&mut self, stmt: &mut IfStmt) -> BoxResult<Compiled>;
    fn visit_loop(&mut self, stmt: &mut LoopStmt) -> BoxResult<Compiled>;
    fn visit_impoprt(&mut self, stmt: &mut ImportStmt) -> BoxResult<Compiled>;
    fn visit_mod(&mut self, stmt: &mut ModStmt) -> BoxResult<Compiled>;
    fn visit_asm(&mut self, stmt: &mut AsmStmt) -> BoxResult<Compiled>;
    fn visit_tick(&mut self, stmt: &mut TickStmt) -> BoxResult<Compiled>;
    fn visit_use(&mut self, stmt: &mut UseStmt) -> BoxResult<Compiled>;
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

    fn token(&self) -> Token {
        self.expr.token()
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

    fn token(&self) -> Token {
        self.token.clone()
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

    fn token(&self) -> Token {
        self.name.clone()
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

#[derive(Debug, Clone, PartialEq)]
pub struct AsmStmt {
    pub token: Token,
    pub code: Object,
}

impl AsmStmt {
    pub fn new(code: Object, token: Token) -> Self {
        Self {
            token,
            code
        }
    }
}

impl StmtNode for AsmStmt {
    fn accept(&mut self, visitor: &mut dyn StmtVisitor) -> BoxResult<Compiled> {
        return visitor.visit_asm(self);
    }

    fn token(&self) -> Token {
        self.token.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportStmt {
    pub token: Token,
    pub path: Object,
}

impl ImportStmt {
    pub fn new(path: Object, token: Token) -> Self {
        Self {
            token,
            path
        }
    }
}

impl StmtNode for ImportStmt {
    fn accept(&mut self, visitor: &mut dyn StmtVisitor) -> BoxResult<Compiled> {
        return visitor.visit_impoprt(self);
    }

    fn token(&self) -> Token {
        self.token.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModStmt {
    pub name: Token,
}

impl ModStmt {
    pub fn new(name: Token) -> Self {
        Self {
            name
        }
    }
}

impl StmtNode for ModStmt {
    fn accept(&mut self, visitor: &mut dyn StmtVisitor) -> BoxResult<Compiled> {
        return visitor.visit_mod(self);
    }

    fn token(&self) -> Token {
        self.name.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TickStmt {
    pub token: Token,
    pub word: Expr
}

impl TickStmt {
    pub fn new(word: Expr, token: Token) -> Self {
        Self {
            word,
            token
        }
    }
}

impl StmtNode for TickStmt {
    fn accept(&mut self, visitor: &mut dyn StmtVisitor) -> BoxResult<Compiled> {
        return visitor.visit_tick(self);
    }

    fn token(&self) -> Token {
        self.token.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UseStmt {
    pub module: Token,
    pub words: Vec<Token>
}

impl UseStmt {
    pub fn new(module: Token, words: Vec<Token>) -> Self {
        Self {
            module,
            words
        }
    }
}

impl StmtNode for UseStmt {
    fn accept(&mut self, visitor: &mut dyn StmtVisitor) -> BoxResult<Compiled> {
        return visitor.visit_use(self);
    }

    fn token(&self) -> Token {
        self.module.clone()
    }
}

