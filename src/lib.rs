use crate::{
    env::EnvRef,
    errors::LispError,
    expr::Parser,
    lexer::Lexer,
    value::{Value, eval},
};

pub mod builtin;
pub mod env;
pub mod errors;
pub mod expr;
pub mod lexer;
pub mod value;

pub struct LispInterpreter {
    pub env: EnvRef,
}

impl LispInterpreter {
    pub fn new(root: EnvRef) -> Self {
        Self { env: root }
    }

    pub fn eval(&self, src: &str) -> Result<Value, LispError> {
        let tokens = Lexer::new(src).lex()?;
        let mut parser = Parser::new(tokens);
        let exprs = parser.parse()?;

        let mut last = Value::Nil;
        for exp in exprs {
            last = eval(&exp, self.env.clone())?;
        }
        Ok(last)
    }
}
