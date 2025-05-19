use std::fmt::Display;

use miette::{ByteOffset, Diagnostic, SourceSpan};
use strum::{EnumMessage, IntoStaticStr};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
#[diagnostic(code("65"))]
pub enum TokenError {
    #[error("[line {}] Error: Unexpected character: {}", .line, 
    .src[.span.offset()..(.span.offset() + .span.len())].to_string())]
    InvalidToken {
        #[source_code]
        src: String,
        #[label("here")]
        span: SourceSpan,
        line: usize,
    },

    #[error("[line {}] Error: Unterminated string: {}", .line, 
    .src[.span.offset()..(.span.offset() + .span.len())].to_string())]
    UnterminatedString {
        #[source_code]
        src: String,
        #[label("here")]
        span: SourceSpan,
        line: usize,
    },
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Token {
    pub(crate) lexeme: TokenType,
    source_span: Option<SourceSpan>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, EnumMessage, IntoStaticStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum TokenType {
    // keywords
    #[strum(message = "true")]
    True,
    #[strum(message = "false")]
    False,
    #[strum(message = "nil")]
    Nil,
    #[strum(message = "and")]
    And,
    #[strum(message = "or")]
    Or,
    #[strum(message = "class")]
    Class,
    #[strum(message = "for")]
    For,
    #[strum(message = "fun")]
    Fun,
    #[strum(message = "if")]
    If,
    #[strum(message = "else")]
    Else,
    #[strum(message = "return")]
    Return,
    #[strum(message = "super")]
    Super,
    #[strum(message = "this")]
    This,
    #[strum(message = "var")]
    Var,
    #[strum(message = "while")]
    While,
    #[strum(message = "print")]
    Print,

    // literals
    #[strum(message = "(")]
    LeftParen,
    #[strum(message = ")")]
    RightParen,
    #[strum(message = "{")]
    LeftBrace,
    #[strum(message = "}")]
    RightBrace,
    #[strum(message = ",")]
    Comma,
    #[strum(message = ".")]
    Dot,
    #[strum(message = "-")]
    Minus,
    #[strum(message = "+")]
    Plus,
    #[strum(message = ";")]
    SemiColon,
    #[strum(message = "*")]
    Star,
    #[strum(message = "=")]
    Eq,
    #[strum(message = "==")]
    EqEq,
    #[strum(message = "!")]
    Bang,
    #[strum(message = "!=")]
    BangEq,
    #[strum(message = "<")]
    Less,
    #[strum(message = "<=")]
    LessEq,
    #[strum(message = ">")]
    Greater,
    #[strum(message = ">=")]
    GreaterEq,
    #[strum(message = "/")]
    Slash,
    #[strum(detailed_message = "{.val}")]
    Number { val: f64 },
    #[strum(detailed_message = "{.val}")]
    Identifier { val: String },
    #[strum(detailed_message = "{.val}")]
    String { val: String },
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Number { val } => write!(f, "{val}"),
            TokenType::Identifier { val } | TokenType::String { val } => write!(f, "{val}"),
            _ => write!(f, "{}", self.get_message().unwrap()),
        }
    }
}

impl Token {
    pub fn new(ty: TokenType, start: ByteOffset, len: usize) -> Token {
        Token {
            lexeme: ty,
            source_span: Some((start, len).into()),
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ", self.lexeme)?;
        match &self.lexeme {
            TokenType::Number { val } => {
                write!(f, "{} {}", val, val)
            }
            TokenType::Identifier { val } => {
                write!(f, "{} ", val)
            }
            TokenType::String { val } => {
                write!(f, "\"{}\" {}", val, val)
            }
            _ => {
                write!(f, "{} null", self.lexeme.get_message().unwrap())
            }
        }
    }
}

macro_rules! is_keyword_token {
    ($k:expr) => {
        match ($k) {
            "and" => Some(TokenType::And),
            "class" => Some(TokenType::Class),
            "else" => Some(TokenType::Else),
            "false" => Some(TokenType::False),
            "for" => Some(TokenType::For),
            "fun" => Some(TokenType::Fun),
            "if" => Some(TokenType::If),
            "nil" => Some(TokenType::Nil),
            "or" => Some(TokenType::Or),
            "print" => Some(TokenType::Print),
            "return" => Some(TokenType::Return),
            "super" => Some(TokenType::Super),
            "this" => Some(TokenType::This),
            "true" => Some(TokenType::True),
            "var" => Some(TokenType::Var),
            "while" => Some(TokenType::While),
            _ => None,
        }
    };
}

pub struct Scanner<'a> {
    source: &'a str,
}

impl Scanner<'_> {
    pub fn new(source: &str) -> Scanner {
        Scanner { source }
    }

    pub fn scan(&mut self) -> miette::Result<Vec<Token>> {
        let source = self.source;
        let mut line: usize = 1;
        let mut tokens = vec![];
        let mut peekable_iter = source.char_indices().peekable();

        #[allow(clippy::while_let_loop)]
        loop {
            match peekable_iter.next() {
                Some((i, c)) => match c {
                    c if one_of(c, "\n\r") => {
                        line += 1;
                        continue;
                    }
                    c if c.is_whitespace() => continue,
                    c if one_of(c, "(){}+-;,*.") => match c {
                        '(' => tokens.push(Token::new(TokenType::LeftParen, i, 1)),
                        ')' => tokens.push(Token::new(TokenType::RightParen, i, 1)),
                        '{' => tokens.push(Token::new(TokenType::LeftBrace, i, 1)),
                        '}' => tokens.push(Token::new(TokenType::RightBrace, i, 1)),
                        ',' => tokens.push(Token::new(TokenType::Comma, i, 1)),
                        '.' => tokens.push(Token::new(TokenType::Dot, i, 1)),
                        '+' => tokens.push(Token::new(TokenType::Plus, i, 1)),
                        '-' => tokens.push(Token::new(TokenType::Minus, i, 1)),
                        ';' => tokens.push(Token::new(TokenType::SemiColon, i, 1)),
                        '*' => tokens.push(Token::new(TokenType::Star, i, 1)),
                        _ => unreachable!(),
                    },
                    c if one_of(c, "<>=!") => match c {
                        '=' => {
                            if peekable_iter.peek().is_some_and(|(_, l)| *l == '=') {
                                peekable_iter.next();
                                tokens.push(Token::new(TokenType::EqEq, i, 2))
                            } else {
                                tokens.push(Token::new(TokenType::Eq, i, 1))
                            }
                        }
                        '<' => {
                            if peekable_iter.peek().is_some_and(|(_, l)| *l == '=') {
                                peekable_iter.next();
                                tokens.push(Token::new(TokenType::LessEq, i, 2))
                            } else {
                                tokens.push(Token::new(TokenType::Less, i, 1))
                            }
                        }
                        '>' => {
                            if peekable_iter.peek().is_some_and(|(_, l)| *l == '=') {
                                peekable_iter.next();
                                tokens.push(Token::new(TokenType::GreaterEq, i, 2))
                            } else {
                                tokens.push(Token::new(TokenType::Greater, i, 1))
                            }
                        }
                        '!' => {
                            if peekable_iter.peek().is_some_and(|(_, l)| *l == '=') {
                                peekable_iter.next();
                                tokens.push(Token::new(TokenType::BangEq, i, 2))
                            } else {
                                tokens.push(Token::new(TokenType::Bang, i, 1))
                            }
                        }
                        _ => unreachable!(),
                    },
                    '\"' => {
                        let mut str = String::new();

                        while peekable_iter
                            .peek()
                            .is_some_and(|(_, l)| !matches!(*l, '"'))
                        {
                            let (_, ch) = peekable_iter.next().unwrap();
                            str.push(ch);
                        }

                        if peekable_iter.peek().is_none() {
                            return Err(TokenError::UnterminatedString {
                                src: self.source.to_string(),
                                span: (i, str.len()).into(),
                                line,
                            }
                            .into());
                        } else {
                            // consume terminating "
                            let (l, _) = peekable_iter.next().unwrap();
                            tokens.push(Token::new(
                                TokenType::String { val: str },
                                i + 1,     // i points to the first '"' so add 1
                                l - 1 - i, // l points to the last '"' so sub 1
                            ))
                        }
                    }
                    c if c.is_ascii_digit() => {
                        let mut number = String::from(c);
                        while peekable_iter
                            .peek()
                            .is_some_and(|(_, l)| l.is_ascii_digit())
                        {
                            let (_, ch) = peekable_iter.next().unwrap();
                            number.push(ch);
                        }

                        if peekable_iter.peek().is_some_and(|(_, l)| matches!(*l, '.')) {
                            let (_, ch) = peekable_iter.next().unwrap();
                            number.push(ch);
                        }

                        while peekable_iter
                            .peek()
                            .is_some_and(|(_, l)| l.is_ascii_digit())
                        {
                            let (_, ch) = peekable_iter.next().unwrap();
                            number.push(ch);
                        }

                        let mut num_literal = number.as_str();

                        // There are 3 possibilities now. The number can be one of
                        // 1. just digits e.g. 123
                        // 2. digits plus a trailing . e.g. 123.
                        // 3. digits before and after a . e.g. 123.45
                        // So doing the splitn(3, '.') will result in
                        // 1. Some, None, None => the _ case
                        // 2. Some, Some, None => the 123. case
                        // 3. Some, Some, Some => the 123.45 case
                        let mut split = num_literal.splitn(3, '.');
                        match (split.next(), split.next(), split.next()) {
                            (Some(first), Some(second), Some(_)) => {
                                num_literal = &num_literal[..first.len() + 1 + second.len()]; // +1 for the dot sep
                            }
                            (Some(first), Some(second), None) => {
                                if second.is_empty() {
                                    num_literal = &num_literal[..first.len()];
                                }
                            }
                            _ => {}
                        }

                        let val = match num_literal.parse() {
                            Ok(value) => value,
                            Err(_e) => {
                                return Err(TokenError::InvalidToken {
                                    src: self.source.to_string(),
                                    span: (i, num_literal.len()).into(),
                                    line,
                                }
                                .into());
                            }
                        };

                        tokens.push(Token::new(TokenType::Number { val }, i, num_literal.len()))
                    }
                    c if c.is_alphabetic() | (c == '_') => {
                        let mut str = String::from(c);
                        while peekable_iter
                            .peek()
                            .is_some_and(|(_, l)| matches!(*l, '_' | 'a'..='z'|'A'..='Z'))
                        {
                            let (_, ch) = peekable_iter.next().unwrap();
                            str.push(ch);
                        }

                        if let Some(keyword) = is_keyword_token!(str.as_str()) {
                            tokens.push(Token::new(keyword, i, str.len()))
                        } else {
                            let len = str.len();
                            tokens.push(Token::new(TokenType::Identifier { val: str }, i, len))
                        }
                    }
                    _ => {
                        return Err(TokenError::InvalidToken {
                            src: self.source.to_string(),
                            span: (i, 1).into(),
                            line,
                        }
                        .into());
                    }
                },
                None => {
                    break;
                }
            }
        }

        Ok(tokens)
    }
}

fn one_of(c: char, chars: &str) -> bool {
    chars.chars().any(|possible| possible == c)
}

#[cfg(test)]
mod tests {
    use crate::token::{Token, TokenError, TokenType};

    use super::Scanner;

    #[test]
    fn test_eqeq_literal() {
        let source = "==";
        assert_token_eq(
            source,
            &[Token {
                lexeme: TokenType::EqEq,
                source_span: Some((0, source.len()).into()),
            }],
        );
    }

    #[test]
    fn test_bang_eq_literal() {
        let source = "!=";
        assert_token_eq(
            source,
            &[Token {
                lexeme: TokenType::BangEq,
                source_span: Some((0, source.len()).into()),
            }],
        );
    }

    #[test]
    fn test_greater_eq_literal() {
        let source = ">=";
        assert_token_eq(
            source,
            &[Token {
                lexeme: TokenType::GreaterEq,
                source_span: Some((0, source.len()).into()),
            }],
        );
    }

    #[test]
    fn test_less_eq_literal() {
        let source = "<=";
        assert_token_eq(
            source,
            &[Token {
                lexeme: TokenType::LessEq,
                source_span: Some((0, source.len()).into()),
            }],
        );
    }

    #[test]
    fn test_string_litteral() {
        let source = "\"hello\"";
        let expected_src = source.trim_matches('"').to_string();
        let expected_src_len = expected_src.len();
        assert_token_eq(
            source,
            &[Token {
                lexeme: TokenType::String { val: expected_src },
                source_span: Some((1, expected_src_len).into()),
            }],
        );
    }

    #[test]
    fn test_identifier() {
        let source = "_hello";
        assert_token_eq(
            source,
            &[Token {
                lexeme: TokenType::Identifier {
                    val: source.to_string(),
                },
                source_span: Some((0, source.len()).into()),
            }],
        );
    }

    #[test]
    fn test_keyword() {
        let source = "and";
        assert_token_eq(
            source,
            &[Token {
                lexeme: TokenType::And,
                source_span: Some((0, source.len()).into()),
            }],
        );
    }

    #[test]
    fn test_number() {
        let source = "123.45";
        assert_token_eq(
            source,
            &[Token {
                lexeme: TokenType::Number { val: 123.45 },
                source_span: Some((0, source.len()).into()),
            }],
        );
    }

    fn assert_token_eq(source: &str, expected_tokens: &[Token]) {
        let expected = Vec::from(expected_tokens);
        let tokens = Scanner::new(source).scan();
        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();
        assert_eq!(expected.len(), tokens.len());
        assert_eq!(expected, tokens);
    }

    #[test]
    fn test_invalid_token() {
        let tokens = Scanner::new("'").scan();
        match tokens {
            Ok(_) => panic!("expected Err(InvalidToken)"),
            Err(e) => {
                assert!(e.is::<TokenError>());

                let it = e
                    .downcast_ref::<TokenError>()
                    .expect("should be InvalidToken");

                match it {
                    TokenError::InvalidToken { src: _, span, line } => {
                        assert_eq!(1, *line);
                        assert_eq!(0, span.offset());
                        assert_eq!(1, span.len());
                    }
                    _ => panic!("expected InvalidToken"),
                }

                if let Some(code) = e.code() {
                    assert_eq!(format!("{code}"), "65".to_string());
                }
            }
        }
    }
}
