use crate::parsing::{
    Span,
    ast::{KEYWORDS, SYMBOLS_DOUBLE, SYMBOLS_SINGLE},
};

use super::Location;

#[derive(Debug, Clone)]
pub enum LexerErrorType {
    InvalidNumber(String),
    InvalidChar(String),
    InvalidString(String),
    UnexpectedCharacter(char),
}

impl std::fmt::Display for LexerErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexerErrorType::InvalidNumber(s) => write!(f, "Invalid number literal: {}", s),
            LexerErrorType::InvalidChar(s) => write!(f, "Invalid character literal: {}", s),
            LexerErrorType::InvalidString(s) => write!(f, "Invalid string literal: {}", s),
            LexerErrorType::UnexpectedCharacter(s) => write!(f, "Unexpected character: {}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LexerError {
    pub error_type: LexerErrorType,
    pub location: Location,
}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Lexer error at {}: {}", self.location, self.error_type)
    }
}

impl std::error::Error for LexerError {}

type LexerResult<T> = Result<T, LexerError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    IntLiteral(i32),
    CharLiteral(char),
    StringLiteral(String),
    Identifier(String),
    Keyword(String),
    Symbol(String),
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::IntLiteral(_) => write!(f, "Integer Literal"),
            TokenKind::CharLiteral(_) => write!(f, "Character Literal"),
            TokenKind::StringLiteral(_) => write!(f, "String Literal"),
            TokenKind::Identifier(_) => write!(f, "Identifier"),
            TokenKind::Keyword(s) => write!(f, "{}", s),
            TokenKind::Symbol(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub struct Lexer<'src> {
    input_iter: std::iter::Peekable<std::str::Chars<'src>>,
    position: usize,
    location: Location,
}

impl<'src> Lexer<'src> {
    pub fn new(input: &'src str) -> Self {
        Lexer {
            input_iter: input.chars().peekable(),
            position: 0,
            location: Location {
                line: 1,
                column: 1,
                offset: 0,
            },
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.input_iter.peek().copied()
    }

    fn advance(&mut self) {
        if let Some(c) = self.peek() {
            self.position += c.len_utf8();
            self.location.offset = self.position;
            if c == '\n' {
                self.location.line += 1;
                self.location.column = 1;
            } else {
                self.location.column += 1;
            }
            self.input_iter.next();
        }
    }

    fn ws(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        if self.peek() == Some('/') {
            self.advance();
            if self.peek() == Some('/') {
                while let Some(c) = self.peek() {
                    self.advance();
                    if c == '\n' {
                        break;
                    }
                }
            } else {
                // Not a comment, just a single '/'
                // We will handle this in next_token
            }
        }
    }

    fn parse_digits<F>(
        &mut self,
        base: u32,
        max: i64,
        error_msg: &str,
        is_valid: F,
    ) -> LexerResult<i32>
    where
        F: Fn(char) -> bool,
    {
        let start_location = self.location;
        let mut value: i64 = 0;
        let mut found = false;

        while let Some(c) = self.peek() {
            if is_valid(c) {
                value = value
                    .checked_mul(base as i64)
                    .and_then(|v| v.checked_add(c.to_digit(base).unwrap() as i64))
                    .ok_or(LexerError {
                        error_type: LexerErrorType::InvalidNumber(format!(
                            "{}: overflow",
                            error_msg
                        )),
                        location: start_location,
                    })?;
                self.advance();
                found = true;
            } else {
                break;
            }
        }

        if !found {
            return Err(LexerError {
                error_type: LexerErrorType::InvalidNumber(format!("{}: no digits", error_msg)),
                location: start_location,
            });
        }

        if value > max {
            return Err(LexerError {
                error_type: LexerErrorType::InvalidNumber(format!("{}: too large for i32", value)),
                location: start_location,
            });
        }

        Ok(value as i32)
    }

    fn number_lit(&mut self) -> LexerResult<Token> {
        // 1234
        // 0001234
        // -1234
        // 0x1234
        // 0b1010
        // 0o1234

        let start_location = self.location;
        let peeked = self.peek();
        if peeked == Some('0') {
            self.advance();
            if let Some(c) = self.peek() {
                match c {
                    'x' => {
                        self.advance();
                        self.parse_digits(16, i32::MAX as i64, "Hexadecimal literal", |c| {
                            c.is_digit(16)
                        })
                        .map(|v| Token {
                            kind: TokenKind::IntLiteral(v),
                            span: Span::new(start_location, self.location),
                        })
                    }
                    'b' => {
                        self.advance();
                        self.parse_digits(2, i32::MAX as i64, "Binary literal", |c| c.is_digit(2))
                            .map(|v| Token {
                                kind: TokenKind::IntLiteral(v),
                                span: Span::new(start_location, self.location),
                            })
                    }
                    'o' => {
                        self.advance();
                        self.parse_digits(8, i32::MAX as i64, "Octal literal", |c| c.is_digit(8))
                            .map(|v| Token {
                                kind: TokenKind::IntLiteral(v),
                                span: Span::new(start_location, self.location),
                            })
                    }
                    w if w.is_whitespace() || w.is_ascii_punctuation() => {
                        // just 0
                        return Ok(Token {
                            kind: TokenKind::IntLiteral(0),
                            span: Span::new(start_location, self.location),
                        });
                    }
                    d if d.is_digit(10) => self
                        .parse_digits(10, i32::MAX as i64, "Decimal literal", |c| c.is_digit(10))
                        .map(|v| Token {
                            kind: TokenKind::IntLiteral(v),
                            span: Span::new(start_location, self.location),
                        }),
                    _ => {
                        return Err(LexerError {
                            error_type: LexerErrorType::InvalidNumber(format!(
                                "Unexpected character '{}' after leading 0",
                                c
                            )),
                            location: self.location,
                        });
                    }
                }
            } else {
                // just 0
                return Ok(Token {
                    kind: TokenKind::IntLiteral(0),
                    span: Span::new(start_location, self.location),
                });
            }
        } else if peeked.is_some_and(|c| c.is_digit(10)) {
            self.parse_digits(10, i32::MAX as i64, "Decimal literal", |c| c.is_digit(10))
                .map(|v| Token {
                    kind: TokenKind::IntLiteral(v),
                    span: Span::new(start_location, self.location),
                })
        } else {
            return Err(LexerError {
                error_type: LexerErrorType::InvalidNumber("Expected a number literal".to_string()),
                location: self.location,
            });
        }
    }

    fn char_lit(&mut self) -> LexerResult<Token> {
        // 'b'
        // '\n'
        // '\''

        let start_location = self.location;

        if self.peek() != Some('\'') {
            return Err(LexerError {
                error_type: LexerErrorType::InvalidChar("Expected a character literal".to_string()),
                location: self.location,
            });
        }
        self.advance(); // consume opening '
        let c = match self.peek() {
            Some('\\') => {
                self.advance();
                match self.peek() {
                    Some('n') => {
                        self.advance();
                        '\n'
                    }
                    Some('t') => {
                        self.advance();
                        '\t'
                    }
                    Some('r') => {
                        self.advance();
                        '\r'
                    }
                    Some('\'') => {
                        self.advance();
                        '\''
                    }
                    Some('\\') => {
                        self.advance();
                        '\\'
                    }
                    Some(other) => {
                        return Err(LexerError {
                            error_type: LexerErrorType::InvalidChar(format!(
                                "Unknown escape sequence: \\{}",
                                other
                            )),
                            location: self.location,
                        });
                    }
                    None => {
                        return Err(LexerError {
                            error_type: LexerErrorType::InvalidChar(
                                "Unterminated escape sequence".to_string(),
                            ),
                            location: self.location,
                        });
                    }
                }
            }
            Some(c) if c != '\'' && c != '\\' => {
                self.advance();
                c
            }
            Some(_) => {
                return Err(LexerError {
                    error_type: LexerErrorType::InvalidChar(
                        "Invalid character literal".to_string(),
                    ),
                    location: self.location,
                });
            }
            None => {
                return Err(LexerError {
                    error_type: LexerErrorType::InvalidChar(
                        "Unterminated character literal".to_string(),
                    ),
                    location: self.location,
                });
            }
        };

        if self.peek() != Some('\'') {
            return Err(LexerError {
                error_type: LexerErrorType::InvalidChar(
                    "Character literal must contain exactly one character".to_string(),
                ),
                location: self.location,
            });
        }

        self.advance();

        Ok(Token {
            kind: TokenKind::CharLiteral(c),
            span: Span::new(start_location, self.location),
        })
    }

    fn string(&mut self) -> LexerResult<Token> {
        // "hello"
        // "line1\nline2"
        // "quote: \""

        let mut result = String::new();
        let start_location = self.location;

        if self.peek() != Some('"') {
            return Err(LexerError {
                error_type: LexerErrorType::InvalidString("Expected a string literal".to_string()),
                location: self.location,
            });
        }
        self.advance(); // consume opening "

        while let Some(c) = self.peek() {
            if c == '"' {
                self.advance(); // consume closing "
                return Ok(Token {
                    kind: TokenKind::StringLiteral(result),
                    span: Span::new(start_location, self.location),
                });
            } else if c == '\\' {
                self.advance();
                match self.peek() {
                    Some('n') => {
                        self.advance();
                        result.push('\n');
                    }
                    Some('t') => {
                        self.advance();
                        result.push('\t');
                    }
                    Some('r') => {
                        self.advance();
                        result.push('\r');
                    }
                    Some('"') => {
                        self.advance();
                        result.push('"');
                    }
                    Some('\\') => {
                        self.advance();
                        result.push('\\');
                    }
                    Some(other) => {
                        return Err(LexerError {
                            error_type: LexerErrorType::InvalidString(format!(
                                "Unknown escape sequence: \\{}",
                                other
                            )),
                            location: self.location,
                        });
                    }
                    None => {
                        return Err(LexerError {
                            error_type: LexerErrorType::InvalidString(
                                "Unterminated escape sequence".to_string(),
                            ),
                            location: self.location,
                        });
                    }
                }
            } else {
                result.push(c);
                self.advance();
            }
        }

        Err(LexerError {
            error_type: LexerErrorType::InvalidChar("Unterminated string literal".to_string()),
            location: start_location,
        })
    }

    fn parse_ident_or_keyword(&mut self) -> LexerResult<Token> {
        let mut ident = String::new();
        let start_location = self.location;

        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if ident.is_empty() {
            return Err(LexerError {
                error_type: LexerErrorType::InvalidChar("Expected an identifier".to_string()),
                location: self.location,
            });
        }

        let kind = if KEYWORDS.contains(&ident.as_str()) {
            TokenKind::Keyword(ident)
        } else {
            TokenKind::Identifier(ident)
        };

        Ok(Token {
            kind,
            span: Span::new(start_location, self.location),
        })
    }

    fn next_token(&mut self) -> LexerResult<Option<Token>> {
        self.ws();
        self.skip_comment();
        self.ws();

        if self.peek().is_none() {
            return Ok(None);
        }

        let start_location = self.location;
        let c = self.peek().unwrap();
        // is_digit also nicely covers 0x, 0b, 0o prefixes since they start with '0'
        let token = if c.is_digit(10) {
            self.number_lit()?
        } else if c == '\'' {
            self.char_lit()?
        } else if c == '"' {
            self.string()?
        } else if c.is_alphabetic() || c == '_' {
            self.parse_ident_or_keyword()?
        } else {
            self.advance();
            let another = self.peek();
            let both = format!("{}{}", c, another.unwrap_or('\0'));
            if SYMBOLS_DOUBLE.contains(&both.as_str()) {
                self.advance();
                Token {
                    kind: TokenKind::Symbol(both),
                    span: Span::new(start_location, self.location),
                }
            } else if SYMBOLS_SINGLE.contains(&c.to_string().as_str()) {
                Token {
                    kind: TokenKind::Symbol(c.to_string()),
                    span: Span::new(start_location, self.location),
                }
            } else {
                return Err(LexerError {
                    error_type: LexerErrorType::UnexpectedCharacter(c),
                    location: self.location,
                });
            }
        };

        Ok(Some(token))
    }

    pub fn tokenize(&mut self) -> LexerResult<Vec<Token>> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token()? {
            tokens.push(token);
        }
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let input = r#"
        const x: int = 42;
        // This is a comment
        "#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        dbg!(&tokens);
        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[0].kind, TokenKind::Keyword("const".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::Identifier("x".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Symbol(":".to_string()));
        assert_eq!(tokens[3].kind, TokenKind::Keyword("int".to_string()));
        assert_eq!(tokens[4].kind, TokenKind::Symbol("=".to_string()));
        assert_eq!(tokens[5].kind, TokenKind::IntLiteral(42));
        assert_eq!(tokens[6].kind, TokenKind::Symbol(";".to_string()));
    }
}
