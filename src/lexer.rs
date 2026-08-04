use std::fmt::Display;

use crate::errors::LexerError;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Integer(i64),
    Boolean(bool),
    String(String),
    Symbol(String),
    LeftParen,
    RightParen,
    Plus,
    Minus,
    Mul,
    Div,
    Eq,  // =
    Lt,  // <
    Lte, // <=
    Gt,  // >
    Gte, // >=
    Quote,
    Nil,
    Eof,
    Whitespace,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Lexer {
    pub chars: Vec<char>,
    pub start: usize,
    pub current: usize,
}

impl Lexer {
    pub fn new(src: &str) -> Self {
        Self {
            chars: src.chars().collect(),
            start: 0,
            current: 0,
        }
    }

    /// parse char into tokens
    /// 1. check if the source is end
    /// 2. get next token from next_token method
    /// 3. append the token into the result
    pub fn lex(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();
        loop {
            if self.is_end() {
                break;
            }

            self.start = self.current;
            let tk = self.next_token()?;
            if tk == Token::Whitespace {
                continue;
            }
            tokens.push(tk);
        }

        Ok(tokens)
    }

    fn is_end(&self) -> bool {
        self.current >= self.chars.len()
    }

    /// 1. get the current char and move current
    /// 2. dispatch parse token on current char
    /// 3. return the token
    fn next_token(&mut self) -> Result<Token, LexerError> {
        let cur = self.advance().ok_or_else(|| LexerError::EOF)?;
        match cur {
            '\n' | '\t' | ' ' => Ok(Token::Whitespace),
            '(' => Ok(Token::LeftParen),
            ')' => Ok(Token::RightParen),
            '+' => Ok(Token::Plus),
            '-' => Ok(Token::Minus),
            '*' => Ok(Token::Mul),
            '/' => Ok(Token::Div),
            '\'' => Ok(Token::Quote),
            '#' => {
                if self.is_match('t') {
                    self.advance();
                    Ok(Token::Boolean(true))
                } else if self.is_match('f') {
                    self.advance();
                    Ok(Token::Boolean(false))
                } else {
                    Err(LexerError::UnsupportedChar('#'))
                }
            }
            '=' => Ok(Token::Eq),
            '>' => {
                if self.is_match('=') {
                    self.advance();
                    Ok(Token::Gte)
                } else {
                    Ok(Token::Gt)
                }
            }

            '<' => {
                if self.is_match('=') {
                    self.advance();
                    Ok(Token::Lte)
                } else {
                    Ok(Token::Lt)
                }
            }

            '"' => Ok(self.parse_string()?),

            ch => {
                if ch.is_alphabetic() {
                    return self.parse_symbol_kw();
                } else if ch.is_ascii_digit() {
                    return self.parse_num();
                } else {
                    Err(LexerError::UnsupportedChar(ch))
                }
            }
        }
    }

    fn parse_string(&mut self) -> Result<Token, LexerError> {
        let mut is_str_end = false;
        while let Some(ch) = self.peek() {
            if ch == '"' {
                is_str_end = true;
                break;
            }

            self.advance();
        }

        if self.is_end() && !is_str_end {
            return Err(LexerError::UnterminatedString);
        }

        let s = String::from_iter(&self.chars[self.start + 1..self.current]);
        self.advance();

        Ok(Token::String(s))
    }

    fn parse_symbol_kw(&mut self) -> Result<Token, LexerError> {
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() {
                self.advance();
            } else {
                break;
            }
        }

        let sym = String::from_iter(&self.chars[self.start..self.current]);
        match sym.as_str() {
            "nil" => Ok(Token::Nil),
            _ => Ok(Token::Symbol(sym)),
        }
    }

    fn parse_num(&mut self) -> Result<Token, LexerError> {
        while let Some(d) = self.peek() {
            if d.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }

        let num = String::from_iter(&self.chars[self.start..self.current]);
        Ok(Token::Integer(num.parse::<i64>()?))
    }

    fn is_match(&self, ch: char) -> bool {
        if let Some(cur) = self.peek() {
            ch == cur
        } else {
            false
        }
    }

    fn peek(&self) -> Option<char> {
        if self.is_end() {
            None
        } else {
            Some(self.chars[self.current])
        }
    }

    /// get and current char
    /// move current to next pos
    fn advance(&mut self) -> Option<char> {
        if self.is_end() {
            None
        } else {
            self.current += 1;
            Some(self.chars[self.current - 1])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex_ok(src: &str) -> Vec<Token> {
        let mut l = Lexer::new(src);
        l.lex().expect("lex should succeed")
    }

    fn lex_err(src: &str) -> LexerError {
        let mut l = Lexer::new(src);
        l.lex().expect_err("lex should fail")
    }

    // ===== punctuation =====

    #[test]
    fn empty_parens() {
        assert_eq!(lex_ok("()"), vec![Token::LeftParen, Token::RightParen]);
    }

    #[test]
    fn quote_token() {
        assert_eq!(lex_ok("'"), vec![Token::Quote]);
    }

    // ===== arithmetic =====

    #[test]
    fn four_arithmetic_operators() {
        assert_eq!(
            lex_ok("+ - * /"),
            vec![Token::Plus, Token::Minus, Token::Mul, Token::Div]
        );
    }

    // ===== comparison =====

    #[test]
    fn single_char_comparisons() {
        assert_eq!(lex_ok("="), vec![Token::Eq]);
        assert_eq!(lex_ok("<"), vec![Token::Lt]);
        assert_eq!(lex_ok(">"), vec![Token::Gt]);
    }

    #[test]
    fn two_char_comparisons() {
        assert_eq!(lex_ok("<="), vec![Token::Lte]);
        assert_eq!(lex_ok(">="), vec![Token::Gte]);
    }

    // ===== booleans =====

    #[test]
    fn boolean_true() {
        assert_eq!(lex_ok("#t"), vec![Token::Boolean(true)]);
    }

    #[test]
    fn boolean_false() {
        assert_eq!(lex_ok("#f"), vec![Token::Boolean(false)]);
    }

    // ===== strings =====

    #[test]
    fn empty_string_literal() {
        assert_eq!(lex_ok("\"\""), vec![Token::String(String::new())]);
    }

    #[test]
    fn plain_string_literal() {
        assert_eq!(lex_ok("\"hello\""), vec![Token::String("hello".into())]);
    }

    #[test]
    fn unterminated_string_is_error() {
        assert!(matches!(lex_err("\"hello"), LexerError::UnterminatedString));
    }

    // ===== symbols =====

    #[test]
    fn multi_char_symbol() {
        assert_eq!(lex_ok("foo"), vec![Token::Symbol("foo".into())]);
    }

    #[test]
    fn symbol_followed_by_paren() {
        assert_eq!(
            lex_ok("foo()"),
            vec![
                Token::Symbol("foo".into()),
                Token::LeftParen,
                Token::RightParen,
            ]
        );
    }

    // ===== keywords =====

    #[test]
    fn keyword_define() {
        assert_eq!(lex_ok("define"), vec![Token::Symbol("define".to_string())]);
    }

    #[test]
    fn keyword_lambda() {
        assert_eq!(lex_ok("lambda"), vec![Token::Symbol("lambda".to_string())]);
    }

    #[test]
    fn keyword_if() {
        assert_eq!(lex_ok("if"), vec![Token::Symbol("if".to_string())]);
    }

    #[test]
    fn keyword_begin() {
        assert_eq!(lex_ok("begin"), vec![Token::Symbol("begin".to_string())]);
    }

    #[test]
    fn keyword_nil() {
        assert_eq!(lex_ok("nil"), vec![Token::Nil]);
    }

    // ===== numbers =====

    #[test]
    fn integer_42() {
        assert_eq!(lex_ok("42"), vec![Token::Integer(42)]);
    }

    #[test]
    fn negative_integer_is_minus_then_integer() {
        // Per design, "-" is an operator token; "(-7)" lexes as Minus Integer 7.
        assert_eq!(lex_ok("-7"), vec![Token::Minus, Token::Integer(7)]);
    }

    // ===== combined (design §3.5 simplified) =====

    #[test]
    fn simple_addition_call() {
        assert_eq!(
            lex_ok("(+ 40 2)"),
            vec![
                Token::LeftParen,
                Token::Plus,
                Token::Integer(40),
                Token::Integer(2),
                Token::RightParen,
            ]
        );
    }

    // ===== errors =====

    #[test]
    fn invalid_hash_char_is_error() {
        assert!(matches!(lex_err("#x"), LexerError::UnsupportedChar('#')));
    }
}
