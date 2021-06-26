use super::parser::*;
use super::stmt::*;
use super::error::*;
use super::expr::*;

pub struct Compiler {
    stmts: Vec<Stmt>
}

impl Compiler {
    pub fn new(source: &str, path: &str) -> Result<Self, ErrorList> {
        let mut parser = Parser::new(source, path)?;
        let stmts = parser.parse()?;
        Ok(Self {
            stmts
        })
    }

    pub fn compile() -> Result<Self, ErrorList> {
        panic!();
    }
}
