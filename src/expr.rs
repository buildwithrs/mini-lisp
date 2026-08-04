use crate::{errors::ParserError, lexer::Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Integer(i64),
    Boolean(bool),
    Symbol(String),
    Str(String),
    List(Vec<Expr>),
    Nil,
}

pub type LispProgram = Vec<Expr>;

pub struct Parser {
    pub tokens: Vec<Token>,
    pub current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<LispProgram, ParserError> {
        let mut lisp_program = LispProgram::new();
        
        loop {
            if self.is_end() {
                break;
            }

            let expr = self.parse_expr()?;
            lisp_program.push(expr);
        }

        Ok(lisp_program)
    }

    fn parse_expr(&mut self) -> Result<Expr, ParserError> {
        let cur = self.peek().ok_or_else(|| ParserError::EOF)?;
        match cur {
            Token::Integer(v) => {
                self.advance();
                Ok(Expr::Integer(v))
            }
            Token::Boolean(v) => {
                self.advance();
                Ok(Expr::Boolean(v))
            }

            Token::Quote => {
                self.advance();
                let quote_expr = self.parse_expr()?;
                Ok(Expr::List(vec![
                    Expr::Symbol("quote".into()),
                    quote_expr,
                ]))
            }

            Token::String(v) => {
                self.advance();
                Ok(Expr::Str(v))
            }
            Token::Symbol(sym) => {
                self.advance();
                Ok(Expr::Symbol(sym))
            }

            Token::Nil => {
                self.advance();
                Ok(Expr::Nil)
            }
            Token::LeftParen => {
                Ok(self.parse_list()?)
            }

            Token::Plus => self.op_symbol("+"),
            Token::Minus => self.op_symbol("-"),
            Token::Mul => self.op_symbol("*"),
            Token::Div => self.op_symbol("/"),
            Token::Eq => self.op_symbol("="),
            Token::Lt => self.op_symbol("<"),
            Token::Lte => self.op_symbol("<="),
            Token::Gt => self.op_symbol(">"),
            Token::Gte => self.op_symbol(">="),

            _ => Err(ParserError::UnknownToken(cur))
        }
    }

    /// Advance past an operator token and produce the matching Symbol.
    fn op_symbol(&mut self, name: &str) -> Result<Expr, ParserError> {
        self.advance();
        Ok(Expr::Symbol(name.into()))
    }

    fn parse_list(&mut self) -> Result<Expr, ParserError> {
        self.advance();
        let mut exprs = Vec::new();
        loop {
            if self.is_end() {
                break;
            }

            if self.peek().is_some_and(|tk| tk == Token::RightParen) {
                break;
            }

            exprs.push(self.parse_expr()?);
        }

        self.consume(Token::RightParen)?;
        Ok(Expr::List(exprs))
    }

    fn is_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn consume(&mut self, tk: Token) -> Result<Token, ParserError> {
        if self.peek().is_some_and(|t| t == tk) {
            let t = self.advance().unwrap();
            Ok(t)
        } else {
            Err(ParserError::ExpectToken(tk))
        }
    }

    fn peek(&mut self) -> Option<Token> {
        if self.is_end() {
            None
        } else {
            Some(self.tokens[self.current].clone())
        }
    }

    fn advance(&mut self) -> Option<Token> {
        if self.is_end() {
            None
        } else {
            self.current += 1;
            Some(self.tokens[self.current - 1].clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Token;

    fn parse_tokens(tokens: Vec<Token>) -> LispProgram {
        let mut p = Parser::new(tokens);
        p.parse().expect("parse should succeed")
    }

    fn parse_tokens_err(tokens: Vec<Token>) -> ParserError {
        let mut p = Parser::new(tokens);
        p.parse().expect_err("parse should fail")
    }

    // ===== atoms =====

    #[test]
    fn empty_token_stream_yields_empty_program() {
        assert_eq!(parse_tokens(vec![]), vec![]);
    }

    #[test]
    fn parses_integer() {
        assert_eq!(
            parse_tokens(vec![Token::Integer(42)]),
            vec![Expr::Integer(42)]
        );
    }

    #[test]
    fn parses_boolean_true() {
        assert_eq!(
            parse_tokens(vec![Token::Boolean(true)]),
            vec![Expr::Boolean(true)]
        );
    }

    #[test]
    fn parses_boolean_false() {
        assert_eq!(
            parse_tokens(vec![Token::Boolean(false)]),
            vec![Expr::Boolean(false)]
        );
    }

    // ===== operator tokens (design §3.2: operators are symbols) =====

    #[test]
    fn parses_plus_as_symbol() {
        assert_eq!(
            parse_tokens(vec![Token::Plus]),
            vec![Expr::Symbol("+".into())]
        );
    }

    #[test]
    fn parses_minus_as_symbol() {
        assert_eq!(
            parse_tokens(vec![Token::Minus]),
            vec![Expr::Symbol("-".into())]
        );
    }

    #[test]
    fn parses_mul_as_symbol() {
        assert_eq!(
            parse_tokens(vec![Token::Mul]),
            vec![Expr::Symbol("*".into())]
        );
    }

    #[test]
    fn parses_div_as_symbol() {
        assert_eq!(
            parse_tokens(vec![Token::Div]),
            vec![Expr::Symbol("/".into())]
        );
    }

    #[test]
    fn parses_eq_as_symbol() {
        assert_eq!(
            parse_tokens(vec![Token::Eq]),
            vec![Expr::Symbol("=".into())]
        );
    }

    #[test]
    fn parses_lt_as_symbol() {
        assert_eq!(
            parse_tokens(vec![Token::Lt]),
            vec![Expr::Symbol("<".into())]
        );
    }

    #[test]
    fn parses_lte_as_symbol() {
        assert_eq!(
            parse_tokens(vec![Token::Lte]),
            vec![Expr::Symbol("<=".into())]
        );
    }

    #[test]
    fn parses_gt_as_symbol() {
        assert_eq!(
            parse_tokens(vec![Token::Gt]),
            vec![Expr::Symbol(">".into())]
        );
    }

    #[test]
    fn parses_gte_as_symbol() {
        assert_eq!(
            parse_tokens(vec![Token::Gte]),
            vec![Expr::Symbol(">=".into())]
        );
    }

    #[test]
    fn parses_arithmetic_call() {
        // (+ 1 2)
        assert_eq!(
            parse_tokens(vec![
                Token::LeftParen,
                Token::Plus,
                Token::Integer(1),
                Token::Integer(2),
                Token::RightParen,
            ]),
            vec![Expr::List(vec![
                Expr::Symbol("+".into()),
                Expr::Integer(1),
                Expr::Integer(2),
            ])]
        );
    }

    #[test]
    fn parses_string_literal() {
        assert_eq!(
            parse_tokens(vec![Token::String("hi".into())]),
            vec![Expr::Str("hi".into())]
        );
    }

    #[test]
    fn parses_symbol() {
        assert_eq!(
            parse_tokens(vec![Token::Symbol("foo".into())]),
            vec![Expr::Symbol("foo".into())]
        );
    }

    // ===== lists =====

    #[test]
    fn parses_empty_list() {
        assert_eq!(
            parse_tokens(vec![Token::LeftParen, Token::RightParen]),
            vec![Expr::List(vec![])]
        );
    }

    #[test]
    fn parses_flat_list() {
        assert_eq!(
            parse_tokens(vec![
                Token::LeftParen,
                Token::Integer(1),
                Token::Integer(2),
                Token::Integer(3),
                Token::RightParen,
            ]),
            vec![Expr::List(vec![
                Expr::Integer(1),
                Expr::Integer(2),
                Expr::Integer(3),
            ])]
        );
    }

    #[test]
    fn parses_nested_list() {
        // (+ 1 (* 2 3))
        assert_eq!(
            parse_tokens(vec![
                Token::LeftParen,
                Token::Symbol("+".into()),
                Token::Integer(1),
                Token::LeftParen,
                Token::Symbol("*".into()),
                Token::Integer(2),
                Token::Integer(3),
                Token::RightParen,
                Token::RightParen,
            ]),
            vec![Expr::List(vec![
                Expr::Symbol("+".into()),
                Expr::Integer(1),
                Expr::List(vec![
                    Expr::Symbol("*".into()),
                    Expr::Integer(2),
                    Expr::Integer(3),
                ]),
            ])]
        );
    }

    #[test]
    fn parses_multiple_top_level_expressions() {
        assert_eq!(
            parse_tokens(vec![Token::Integer(1), Token::Integer(2)]),
            vec![Expr::Integer(1), Expr::Integer(2)]
        );
    }

    // ===== errors =====

    #[test]
    fn unterminated_list_is_error() {
        // ( 1 2   (missing closing paren)
        assert!(matches!(
            parse_tokens_err(vec![
                Token::LeftParen,
                Token::Integer(1),
                Token::Integer(2),
            ]),
            ParserError::ExpectToken(Token::RightParen)
        ));
    }

    #[test]
    fn unmatched_close_paren_is_error() {
        // )
        assert!(matches!(
            parse_tokens_err(vec![Token::RightParen]),
            ParserError::UnknownToken(Token::RightParen)
        ));
    }

    // ===== design intent (currently failing — fix parser to make these pass) =====

    #[test]
    fn nil_token_becomes_nil_expr() {
        // `nil` is a runtime literal (empty list / false-like), not a symbol.
        assert_eq!(parse_tokens(vec![Token::Nil]), vec![Expr::Nil]);
    }

    #[test]
    fn quote_with_symbol_desugars() {
        // 'hello  =>  (quote hello)  — design §4.3
        assert_eq!(
            parse_tokens(vec![Token::Quote, Token::Symbol("hello".into())]),
            vec![Expr::List(vec![
                Expr::Symbol("quote".into()),
                Expr::Symbol("hello".into()),
            ])]
        );
    }

    #[test]
    fn quote_with_list_desugars() {
        // '(1 2)  =>  (quote (1 2))  — design §4.3
        assert_eq!(
            parse_tokens(vec![
                Token::Quote,
                Token::LeftParen,
                Token::Integer(1),
                Token::Integer(2),
                Token::RightParen,
            ]),
            vec![Expr::List(vec![
                Expr::Symbol("quote".into()),
                Expr::List(vec![Expr::Integer(1), Expr::Integer(2)]),
            ])]
        );
    }

    #[test]
    fn quote_without_following_expression_is_error() {
        // ' at EOF — design §4.5 "quote without a following expression"
        assert!(matches!(
            parse_tokens_err(vec![Token::Quote]),
            ParserError::EOF
        ));
    }
}
