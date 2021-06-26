/*
 * Scans and parser an input file in one
 * step.
 * Generates a tree-structure
 */

use super::scanner::Scanner;
use super::token::{Token, TokenType};
use super::error::{ExecError, ErrorType, MaybeErrors, BoxResult};
use super::expr::*;
use super::stmt::*;

pub struct Parser {
    current: usize,
    tokens: Vec<Token>,
}

impl Parser {
    pub fn new(source: &str, path: &str) -> MaybeErrors<Parser> {
        let mut scanner = Scanner::new(source, path);
        let tokens = match scanner.scan() {
            MaybeErrors::Results(t) => t,
            MaybeErrors::Errors(err) => return MaybeErrors::Errors(err)
        };

        MaybeErrors::Results(Self {
            current: 0,
            tokens
        })
    }

    pub fn parse(&mut self) -> MaybeErrors<Vec<Stmt>> {
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

        return MaybeErrors::Results(exprs);
    }

    fn stmt(&mut self) -> BoxResult<Stmt> {
        if self.is_match(vec![TokenType::StartDefine]) {
            return self.define_stmt();
        } else {
            // default case
            let expr = match self.expr() {
                Ok(expr) => expr,
                Err(err) => return Err(err)
            };

            return Ok(Stmt::ExprStmt(ExprStmt::new(expr)));
        }
    }

    fn define_stmt(&mut self) -> BoxResult<Stmt> {
        panic!();
    }

    fn expr(&mut self) -> BoxResult<Expr> {
        if self.is_match(vec![
            TokenType::Word,
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
