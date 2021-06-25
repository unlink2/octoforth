/*
 * Scans and parser an input file in one
 * step.
 * Generates a tree-structure
 */

use super::scanner::Scanner;
use super::token::{Token, TokenType};
use super::error::{ExecError, ErrorType, MaybeErrors};

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
}
