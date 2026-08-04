use std::num::ParseIntError;

use thiserror::Error;

use crate::{lexer::Token};

#[derive(Debug, Error)]
pub enum LispError {
    #[error("{0}")]
    LexerError(#[from] LexerError),
    #[error("{0}")]
    ParserError(#[from] ParserError),
    #[error("{0}")]
    EvalError(#[from] EvalError),
}


#[derive(Debug, Error)]
pub enum LexerError {
    #[error("eof")]
    EOF,

    #[error("unsupported char: {0}")]
    UnsupportedChar(char),

    #[error("unterminated string")]
    UnterminatedString,

    #[error("invalid integer")]
    ParseIntError(#[from] ParseIntError),
}


#[derive(Debug, Error)]
pub enum ParserError {
    #[error("eof")]
    EOF,

    #[error("expect token: {0}")]
    ExpectToken(Token),

    #[error("uknown token: {0}")]
    UnknownToken(Token)
}

// EvalError
#[derive(Debug, Error)]
pub enum EvalError {
    #[error("eof")]
    EOF,

    #[error("undfined symbol")]
    UndefinedSymbol,

    #[error("empty application")]
    EmptyApplication,
    
    #[error("bad special form: {0}")]
    BadSpecialForm(&'static str),

    #[error("not callable: {0}")]
    NotCallable(String),

    #[error("bad arg: {0}")]
    BadArg(&'static str),

    #[error("lambda args not match")]
    ArgNotMatch,

    #[error("div by zero")]
    DivisionByZero,
}