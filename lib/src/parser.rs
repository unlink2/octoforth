/*
 * Scans and parser an input file in one
 * step.
 * Generates a tree-structure
 */

use super::scanner::Scanner;
use super::token::{Token, TokenType};
use super::error::{ExecError, ErrorType, ErrorList, BoxResult};
use super::expr::*;
use super::stmt::*;

#[derive(Debug)]
pub struct Parser {
    current: usize,
    tokens: Vec<Token>,
}

impl Parser {
    pub fn new(source: &str, path: &str) -> Result<Parser, ErrorList> {
        let mut scanner = Scanner::new(source, path);
        let tokens = scanner.scan()?;

        Ok(Self {
            current: 0,
            tokens
        })
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ErrorList> {
        let mut exprs = vec![];
        let mut errors = vec![];

        while !self.is_at_end() {
            match self.stmt() {
                Ok(expr) => exprs.push(expr),
                Err(err) => {
                    errors.push(err);
                    self.sync();
                }
            }
        }

        if errors.len() > 0 {
            return Err(ErrorList::new(errors));
        }

        return Ok(exprs);
    }

    fn stmt(&mut self) -> BoxResult<Stmt> {
        if self.is_match(vec![TokenType::StartDefine]) {
            return self.define_stmt();
        } else if self.is_match(vec![TokenType::StartInlineDefine]) {
            return self.define_inline_stmt();
        } else if self.is_match(vec![TokenType::StartConstDefine]) {
            return self.define_const_stmt();
        } else if self.is_match(vec![TokenType::If]) {
            return self.if_stmt();
        } else if self.is_match(vec![TokenType::Loop]) {
            return self.loop_stmt();
        } else if self.is_match(vec![TokenType::Asm]) {
            return self.asm_stmt();
        } else if self.is_match(vec![TokenType::Mod]) {
            return self.mod_stmt();
        } else if self.is_match(vec![TokenType::Use]) {
            return self.use_stmt();
        } else if self.is_match(vec![TokenType::Tick]) {
            return self.tick_stmt();
        } else {
            // default case
            let expr = match self.expr() {
                Ok(expr) => expr,
                Err(err) => return Err(err)
            };

            return Ok(Stmt::Expr(ExprStmt::new(expr)));
        }
    }

    fn block_stmt(&mut self, delim: TokenType) -> BoxResult<Stmt> {
        let mut block = vec![];
        while !self.check(delim)
            && !self.is_at_end() {
            block.push(self.stmt()?);
        }
        self.consume(delim, ErrorType::UnterminatedBlock)?;
        return Ok(Stmt::Block(BlockStmt::new(block, self.previous().clone())));
    }

    fn define_stmt(&mut self) -> BoxResult<Stmt> {
        // eat the first expr which should be a word!
        let name = self.advance().clone();
        if name.token_type != TokenType::Word {
            return Err(Box::new(ExecError::new(ErrorType::ExpectedName, name)));
        }

        let block = Box::new(self.block_stmt(TokenType::EndDefine)?);
        return Ok(Stmt::Define(DefineStmt::new(name, block, DefineMode::Regular)));
    }

    fn define_inline_stmt(&mut self) -> BoxResult<Stmt> {
        // eat the first expr which should be a word!
        let name = self.advance().clone();
        if name.token_type != TokenType::Word {
            return Err(Box::new(ExecError::new(ErrorType::ExpectedName, name)));
        }

        let block = Box::new(self.block_stmt(TokenType::EndDefine)?);
        return Ok(Stmt::Define(DefineStmt::new(name, block, DefineMode::Inline)));
    }

    fn define_const_stmt(&mut self) -> BoxResult<Stmt> {
        // eat the first expr which should be a word!
        let name = self.advance().clone();
        if name.token_type != TokenType::Word {
            return Err(Box::new(ExecError::new(ErrorType::ExpectedName, name)));
        }

        let block = Box::new(self.block_stmt(TokenType::EndDefine)?);
        return Ok(Stmt::Define(DefineStmt::new(name, block, DefineMode::Constant)));
    }

    fn mod_stmt(&mut self) -> BoxResult<Stmt> {
        let mod_name = self.consume(TokenType::Word, ErrorType::UnexpectedToken)?;
        return Ok(Stmt::Mod(ModStmt::new(mod_name)));
    }

    fn use_stmt(&mut self) -> BoxResult<Stmt> {
        let mod_name = self.consume(TokenType::Str, ErrorType::UnexpectedToken)?;
        return Ok(Stmt::Use(UseStmt::new(mod_name.literal.clone(), mod_name)));
    }

    fn asm_stmt(&mut self) -> BoxResult<Stmt> {
        let code = self.consume(TokenType::Str, ErrorType::UnexpectedToken)?;
        return Ok(Stmt::Asm(AsmStmt::new(code.literal.clone(), code)));
    }

    fn if_stmt(&mut self) -> BoxResult<Stmt> {
        // if body ends at then or else
        let mut block = vec![];
        while !self.check(TokenType::Then)
            && !self.check(TokenType::Else)
            && !self.is_at_end() {
            block.push(self.stmt()?);
        }
        let if_block = Box::new(Stmt::Block(BlockStmt::new(block, self.previous().clone())));
        let mut else_block = None;
        if self.check(TokenType::Else) {
            self.advance();
            else_block = Some(Box::new(self.block_stmt(TokenType::Then)?));
        } else {
            self.consume(TokenType::Then, ErrorType::UnterminatedBlock)?;
        }

        Ok(Stmt::If(IfStmt::new(if_block, else_block, self.previous().clone())))
    }

    fn loop_stmt(&mut self) -> BoxResult<Stmt> {
        let loop_body = Box::new(self.block_stmt(TokenType::Until)?);
        return Ok(Stmt::Loop(LoopStmt::new(loop_body, self.previous().clone())));
    }

    fn tick_stmt(&mut self) -> BoxResult<Stmt> {
        let token = self.previous().clone();
        let word = self.expr()?;
        return Ok(Stmt::Tick(TickStmt::new(word, token)));
    }

    fn expr(&mut self) -> BoxResult<Expr> {
        if self.is_match(vec![TokenType::Word]) {
            Ok(Expr::Word(WordExpr::new(self.previous().clone())))
        } else if self.is_match(vec![
            TokenType::Str,
            TokenType::Number,
            TokenType::Real]) {
            Ok(Expr::Literal(LiteralExpr::new(self.previous().clone())))
        } else {
            Err(Box::new(
                    ExecError::new(ErrorType::UnexpectedToken, self.peek().clone())))
        }
    }

    fn consume(&mut self, token_type: TokenType, error: ErrorType) -> BoxResult<Token> {
        if self.check(token_type) {
            return Ok(self.advance().clone());
        }

        return Err(Box::new(ExecError::new(error, self.previous().clone())));
    }

    fn is_match(&mut self, types: Vec<TokenType>) -> bool {
        for t in types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }

        return false;
    }

    fn check(&mut self, token_type: TokenType) -> bool {
        return !self.is_at_end() && self.peek().token_type == token_type;
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        return self.previous();
    }

    fn previous(&self) -> &Token {
        return &self.tokens[self.current-1];
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EndOfFile
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    // attemtps to revocer from error state
    // to allow more than 1 error message per pass
    fn sync(&mut self) {
        self.advance();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::*;

    /// this pretty much tests most of the parser!
    #[test]
    pub fn it_should_scan_definition() {
        let mut parser = Parser::new(": word 1 + ;", "").unwrap();
        let stmts = parser.parse().unwrap();



        assert_eq!(stmts, vec![
            Stmt::Define(DefineStmt::new(
                    Token::new(
                        TokenType::Word,
                        Object::Word("word".into()),
                        "word",
                        1,
                        2,
                        ""),
                Box::new(Stmt::Block(BlockStmt::new(
                    vec![
                    Stmt::Expr(ExprStmt::new(Expr::Literal(LiteralExpr::new(Token::new(
                                        TokenType::Number,
                                        Object::Number(1),
                                        "1",
                                        1,
                                        7,
                                        ""
                                ))))),
                    Stmt::Expr(ExprStmt::new(Expr::Word(WordExpr::new(Token::new(
                                        TokenType::Word,
                                        Object::Word("+".into()),
                                        "+",
                                        1,
                                        9,
                                        ""
                                ))))),
                    ], Token::new(
                        TokenType::EndDefine,
                        Object::Word(";".into()),
                        ";",
                        1,
                        11,
                        ""
                    )))),
                DefineMode::Regular
            ))

        ]);
    }

    #[test]
    pub fn it_should_fail_when_name_is_missing() {
        let mut parser = Parser::new(": 1 +;", "").unwrap();
        let errors = parser.parse().unwrap_err().errors;

        let errors_id: Vec<String> = errors.iter().map(|x| format!("{:?}", x)).collect();
        assert_eq!(errors_id, vec!["type: ExpectedName; lexeme: 1".to_string()]);
    }

    #[test]
    pub fn it_should_fail_when_unterminated() {
        let mut parser = Parser::new(": word 1 +", "").unwrap();
        let errors = parser.parse().unwrap_err().errors;

        let errors_id: Vec<String> = errors.iter().map(|x| format!("{:?}", x)).collect();
        assert_eq!(errors_id, vec!["type: UnterminatedBlock; lexeme: +".to_string()]);
    }

    #[test]
    pub fn it_should_fail_when_unterminated_if() {
        let mut parser = Parser::new("= 1 if 1 1 +", "").unwrap();
        let errors = parser.parse().unwrap_err().errors;

        let errors_id: Vec<String> = errors.iter().map(|x| format!("{:?}", x)).collect();
        assert_eq!(errors_id, vec!["type: UnterminatedBlock; lexeme: +".to_string()]);
    }

    #[test]
    pub fn it_should_fail_when_unterminated_else_if() {
        let mut parser = Parser::new("= 1 if 1 1 + else 2 2 +", "").unwrap();
        let errors = parser.parse().unwrap_err().errors;

        let errors_id: Vec<String> = errors.iter().map(|x| format!("{:?}", x)).collect();
        assert_eq!(errors_id, vec!["type: UnterminatedBlock; lexeme: +".to_string()]);
    }

    #[test]
    pub fn it_should_fail_when_unterminated_loop() {
        let mut parser = Parser::new("loop 1 ", "").unwrap();
        let errors = parser.parse().unwrap_err().errors;

        let errors_id: Vec<String> = errors.iter().map(|x| format!("{:?}", x)).collect();
        assert_eq!(errors_id, vec!["type: UnterminatedBlock; lexeme: 1".to_string()]);
    }

}
