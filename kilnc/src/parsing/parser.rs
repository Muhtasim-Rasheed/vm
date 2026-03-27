use crate::parsing::{
    Location,
    ast::{Expr, Stmt, Ty},
    lexer::{Token, TokenKind},
};

#[derive(Debug, Clone)]
pub enum ParserErrorType<'src> {
    UnexpectedToken(&'src str, Option<TokenKind>),
    UnexpectedEOF(Option<TokenKind>),
}

impl<'src> std::fmt::Display for ParserErrorType<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserErrorType::UnexpectedToken(s, None) => write!(f, "Unexpected token: {}", s),
            ParserErrorType::UnexpectedToken(s, Some(expected)) => {
                write!(f, "Unexpected token: {}, expected: {}", s, expected)
            }
            ParserErrorType::UnexpectedEOF(None) => write!(f, "Unexpected end of file"),
            ParserErrorType::UnexpectedEOF(Some(expected)) => {
                write!(f, "Unexpected end of file, expected: {}", expected)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParserError<'src> {
    pub error_type: ParserErrorType<'src>,
    pub location: Location,
}

impl<'src> std::fmt::Display for ParserError<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parser error at {}: {}", self.location, self.error_type)
    }
}

impl<'src> std::error::Error for ParserError<'src> {}

type ParserResult<'src, T> = Result<T, ParserError<'src>>;

pub struct Parser<'src> {
    tokens: Vec<Token>,
    src: &'src str,
    position: usize,
}

impl<'src> Parser<'src> {
    pub fn new(tokens: Vec<Token>, src: &'src str) -> Self {
        Self {
            tokens,
            src,
            position: 0,
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn peek_source(&self) -> Option<&'src str> {
        self.peek()
            .map(|token| &self.src[token.span.start_offset..token.span.end_offset])
    }

    fn next(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.position);
        if token.is_some() {
            self.position += 1;
        }
        token
    }

    fn expect(&mut self, expected: TokenKind) -> ParserResult<'src, ()> {
        match self.next() {
            Some(token) if token.kind == expected => Ok(()),
            Some(token) => {
                let (start_off, end_off) = (token.span.start_offset, token.span.end_offset);
                let start_loc = token.span.start;
                Err(ParserError {
                    error_type: ParserErrorType::UnexpectedToken(
                        &self.src[start_off..end_off],
                        Some(expected),
                    ),
                    location: start_loc,
                })
            }
            None => Err(ParserError {
                error_type: ParserErrorType::UnexpectedEOF(Some(expected)),
                location: Location {
                    line: 0,
                    column: 0,
                    offset: self.src.len(),
                },
            }),
        }
    }

    fn expect_any(&mut self, expected: &[TokenKind]) -> ParserResult<'src, ()> {
        match self.next() {
            Some(token) if expected.contains(&token.kind) => Ok(()),
            Some(token) => {
                let (start_off, end_off) = (token.span.start_offset, token.span.end_offset);
                let start_loc = token.span.start;
                Err(ParserError {
                    error_type: ParserErrorType::UnexpectedToken(
                        &self.src[start_off..end_off],
                        None,
                    ),
                    location: start_loc,
                })
            }
            None => Err(ParserError {
                error_type: ParserErrorType::UnexpectedEOF(None),
                location: Location {
                    line: 0,
                    column: 0,
                    offset: self.src.len(),
                },
            }),
        }
    }

    fn expect_identifier(&mut self) -> ParserResult<'src, String> {
        match self.next() {
            Some(Token {
                kind: TokenKind::Identifier(name),
                ..
            }) => Ok(name.clone()),
            Some(token) => {
                let (start_off, end_off) = (token.span.start_offset, token.span.end_offset);
                let start_loc = token.span.start;
                Err(ParserError {
                    error_type: ParserErrorType::UnexpectedToken(
                        &self.src[start_off..end_off],
                        Some(TokenKind::Identifier("identifier".to_string())),
                    ),
                    location: start_loc,
                })
            }
            None => Err(ParserError {
                error_type: ParserErrorType::UnexpectedEOF(Some(TokenKind::Identifier(
                    "identifier".to_string(),
                ))),
                location: Location {
                    line: 0,
                    column: 0,
                    offset: self.src.len(),
                },
            }),
        }
    }

    fn parse_type(&mut self) -> ParserResult<'src, Ty> {
        // int
        // *char
        // **void

        let mut pointer_depth = 0;
        while self
            .peek()
            .map_or(false, |t| t.kind == TokenKind::Symbol("*".to_string()))
        {
            self.next();
            pointer_depth += 1;
        }

        let base_ty = match self.next() {
            Some(Token {
                kind: TokenKind::Keyword(kw),
                ..
            }) if kw == "int" => Ty::Int,
            Some(Token {
                kind: TokenKind::Keyword(kw),
                ..
            }) if kw == "char" => Ty::Char,
            Some(Token {
                kind: TokenKind::Keyword(kw),
                ..
            }) if kw == "void" => Ty::Void,
            Some(token) => {
                let (start_off, end_off) = (token.span.start_offset, token.span.end_offset);
                let start_loc = token.span.start;
                return Err(ParserError {
                    error_type: ParserErrorType::UnexpectedToken(
                        &self.src[start_off..end_off],
                        None,
                    ),
                    location: start_loc,
                });
            }
            None => {
                return Err(ParserError {
                    error_type: ParserErrorType::UnexpectedEOF(None),
                    location: Location {
                        line: 0,
                        column: 0,
                        offset: self.src.len(),
                    },
                });
            }
        };

        let mut ty = base_ty;
        for _ in 0..pointer_depth {
            ty = Ty::Ptr(Box::new(ty));
        }

        Ok(ty)
    }

    fn precedence(&self, op: &str) -> Option<u8> {
        match op {
            "=" => Some(1),
            "==" | "!=" => Some(2),
            "<" | ">" | "<=" | ">=" => Some(3),
            "+" | "-" => Some(4),
            "*" | "/" => Some(5),
            _ => None,
        }
    }

    fn right_associative(&self, op: &str) -> bool {
        matches!(op, "=")
    }

    fn parse_binary_op(&mut self, min_prec: u8) -> ParserResult<'src, Expr> {
        let mut left = self.parse_unary()?;

        while let Some(Token {
            kind: TokenKind::Symbol(op),
            ..
        }) = self.peek()
        {
            let op = op.clone();
            if let Some(prec) = self.precedence(op.as_str()) {
                if prec < min_prec {
                    break;
                }
                self.next(); // consume operator
                let next_min_prec = if self.right_associative(op.as_str()) {
                    prec
                } else {
                    prec + 1
                };
                let right = self.parse_binary_op(next_min_prec)?;
                left = Expr::BinaryOp {
                    op: op,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> ParserResult<'src, Expr> {
        if let Some(Token {
            kind: TokenKind::Symbol(op),
            ..
        }) = self.peek()
        {
            if op == "+" || op == "-" || op == "!" {
                let op = op.clone();
                self.next(); // consume operator
                let expr = self.parse_unary()?;
                return Ok(Expr::UnaryOp {
                    op,
                    expr: Box::new(expr),
                });
            } else if op == "*" {
                self.next(); // consume '*'
                let expr = self.parse_unary()?;
                return Ok(Expr::Deref(Box::new(expr)));
            } else if op == "&" {
                self.next(); // consume '&'
                let expr = self.parse_unary()?;
                return Ok(Expr::Addr(Box::new(expr)));
            }
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> ParserResult<'src, Expr> {
        let mut expr = self.parse_primary_atom()?;

        loop {
            match self.peek() {
                Some(Token {
                    kind: TokenKind::Symbol(sym),
                    ..
                }) if sym == "(" => {
                    self.next();
                    let mut args = Vec::new();
                    if !self
                        .peek()
                        .map_or(false, |t| t.kind == TokenKind::Symbol(")".to_string()))
                    {
                        loop {
                            args.push(self.parse_binary_op(1)?);
                            if self
                                .peek()
                                .map_or(false, |t| t.kind == TokenKind::Symbol(")".to_string()))
                            {
                                break;
                            }
                            self.expect(TokenKind::Symbol(",".to_string()))?;
                        }
                    }
                    self.expect(TokenKind::Symbol(")".to_string()))?;
                    expr = Expr::Call {
                        func: Box::new(expr),
                        args,
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary_atom(&mut self) -> ParserResult<'src, Expr> {
        match self.next() {
            Some(Token {
                kind: TokenKind::IntLiteral(value),
                ..
            }) => Ok(Expr::IntLiteral(*value)),
            Some(Token {
                kind: TokenKind::CharLiteral(value),
                ..
            }) => Ok(Expr::CharLiteral(*value)),
            Some(Token {
                kind: TokenKind::StringLiteral(value),
                ..
            }) => Ok(Expr::StringLiteral(value.clone())),
            Some(Token {
                kind: TokenKind::Identifier(name),
                ..
            }) => Ok(Expr::Var(name.clone())),
            Some(Token {
                kind: TokenKind::Symbol(sym),
                ..
            }) if sym == "(" => {
                let expr = self.parse_binary_op(1)?;
                self.expect(TokenKind::Symbol(")".to_string()))?;
                Ok(expr)
            }
            Some(token) => {
                let (start_off, end_off) = (token.span.start_offset, token.span.end_offset);
                let start_loc = token.span.start;
                Err(ParserError {
                    error_type: ParserErrorType::UnexpectedToken(
                        &self.src[start_off..end_off],
                        None,
                    ),
                    location: start_loc,
                })
            }
            None => Err(ParserError {
                error_type: ParserErrorType::UnexpectedEOF(None),
                location: Location {
                    line: 0,
                    column: 0,
                    offset: self.src.len(),
                },
            }),
        }
    }

    fn parse_const(&mut self) -> ParserResult<'src, Stmt> {
        self.expect(TokenKind::Keyword("const".to_string()))?;
        let name = self.expect_identifier()?;
        self.expect(TokenKind::Symbol(":".to_string()))?;
        let ty = self.parse_type()?;
        self.expect(TokenKind::Symbol("=".to_string()))?;
        let value = self.parse_binary_op(1)?;
        self.expect(TokenKind::Symbol(";".to_string()))?;
        Ok(Stmt::Const { name, ty, value })
    }

    fn parse_let(&mut self) -> ParserResult<'src, Stmt> {
        self.expect(TokenKind::Keyword("let".to_string()))?;
        let name = self.expect_identifier()?;
        self.expect(TokenKind::Symbol(":".to_string()))?;
        let ty = self.parse_type()?;
        self.expect(TokenKind::Symbol("=".to_string()))?;
        let value = self.parse_binary_op(1)?;
        self.expect(TokenKind::Symbol(";".to_string()))?;
        Ok(Stmt::Let { name, ty, value })
    }

    fn parse_expr_stmt(&mut self) -> ParserResult<'src, Stmt> {
        let expr = self.parse_binary_op(1)?;
        self.expect(TokenKind::Symbol(";".to_string()))?;
        Ok(Stmt::Expr(expr))
    }

    fn parse_block(&mut self) -> ParserResult<'src, Stmt> {
        self.expect(TokenKind::Symbol("{".to_string()))?;
        let mut stmts = Vec::new();
        while !self
            .peek()
            .map_or(false, |t| t.kind == TokenKind::Symbol("}".to_string()))
        {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(TokenKind::Symbol("}".to_string()))?;
        Ok(Stmt::Block(stmts))
    }

    fn parse_if(&mut self) -> ParserResult<'src, Stmt> {
        self.expect(TokenKind::Keyword("if".to_string()))?;
        let condition = self.parse_binary_op(1)?;
        let then_branch = Box::new(self.parse_stmt()?);
        let else_branch = if self
            .peek()
            .map_or(false, |t| t.kind == TokenKind::Keyword("else".to_string()))
        {
            self.next(); // consume 'else'
            Some(Box::new(self.parse_stmt()?))
        } else {
            None
        };
        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn parse_while(&mut self) -> ParserResult<'src, Stmt> {
        self.expect(TokenKind::Keyword("while".to_string()))?;
        let condition = self.parse_binary_op(1)?;
        let body = Box::new(self.parse_stmt()?);
        Ok(Stmt::While { condition, body })
    }

    fn parse_function_decl(&mut self) -> ParserResult<'src, Stmt> {
        self.expect(TokenKind::Keyword("fn".to_string()))?;
        let name = self.expect_identifier()?;

        self.expect(TokenKind::Symbol("(".to_string()))?;
        let mut params = Vec::new();
        if !self
            .peek()
            .map_or(false, |t| t.kind == TokenKind::Symbol(")".to_string()))
        {
            loop {
                let param_name = self.expect_identifier()?;
                self.expect(TokenKind::Symbol(":".to_string()))?;
                let param_ty = self.parse_type()?;
                params.push((param_name, param_ty));
                if self
                    .peek()
                    .map_or(false, |t| t.kind == TokenKind::Symbol(")".to_string()))
                {
                    break;
                }
                self.expect(TokenKind::Symbol(",".to_string()))?;
            }
        }
        self.expect(TokenKind::Symbol(")".to_string()))?;

        let return_ty = self.parse_type()?;
        let body = Box::new(self.parse_stmt()?);

        Ok(Stmt::FunctionDecl {
            name,
            params,
            return_ty,
            body,
        })
    }

    fn parse_return(&mut self) -> ParserResult<'src, Stmt> {
        self.expect(TokenKind::Keyword("return".to_string()))?;
        let value = if !self
            .peek()
            .map_or(false, |t| t.kind == TokenKind::Symbol(";".to_string()))
        {
            Some(self.parse_binary_op(1)?)
        } else {
            None
        };
        self.expect(TokenKind::Symbol(";".to_string()))?;
        Ok(Stmt::Return(value))
    }

    fn parse_stmt(&mut self) -> ParserResult<'src, Stmt> {
        match self.peek().map(|t| &t.kind) {
            Some(TokenKind::Keyword(kw)) if kw == "const" => self.parse_const(),
            Some(TokenKind::Keyword(kw)) if kw == "let" => self.parse_let(),
            Some(TokenKind::Symbol(sym)) if sym == "{" => self.parse_block(),
            Some(TokenKind::Keyword(kw)) if kw == "if" => self.parse_if(),
            Some(TokenKind::Keyword(kw)) if kw == "while" => self.parse_while(),
            Some(TokenKind::Keyword(kw)) if kw == "fn" => self.parse_function_decl(),
            Some(TokenKind::Keyword(kw)) if kw == "return" => self.parse_return(),
            _ => self.parse_expr_stmt(),
        }
    }

    pub fn parse(&mut self) -> ParserResult<'src, Vec<Stmt>> {
        let mut stmts = Vec::new();
        while self.peek().is_some() {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::lexer::Lexer;

    fn parse_source(src: &str) -> ParserResult<'_, Vec<Stmt>> {
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().unwrap_or_else(|e| {
            panic!("Lexer error at {}: {}", e.location, e.error_type);
        });
        let mut parser = Parser::new(tokens, src);
        parser.parse()
    }

    #[test]
    fn test_parse_const() {
        let src = "const x: int = 42;";
        let stmts = parse_source(src).unwrap();
        assert_eq!(stmts.len(), 1);
        assert_eq!(
            stmts[0],
            Stmt::Const {
                name: "x".to_string(),
                ty: Ty::Int,
                value: Expr::IntLiteral(42),
            }
        );
    }

    #[test]
    fn test_parse_let() {
        let src = r#"
            let y: char = 'a';
            let z: *char = &"hello";
        "#;
        let stmts = parse_source(src).unwrap();
        assert_eq!(stmts.len(), 2);
        assert_eq!(
            stmts[0],
            Stmt::Let {
                name: "y".to_string(),
                ty: Ty::Char,
                value: Expr::CharLiteral('a'),
            }
        );
        assert_eq!(
            stmts[1],
            Stmt::Let {
                name: "z".to_string(),
                ty: Ty::Ptr(Box::new(Ty::Char)),
                value: Expr::Addr(Box::new(Expr::StringLiteral("hello".to_string()))),
            }
        );
    }

    #[test]
    fn test_parse_if() {
        let src = r#"
            if x > 0 {
                y = 1;
            } else {
                y = -1;
            }
        "#;
        let stmts = parse_source(src).unwrap();
        assert_eq!(stmts.len(), 1);
        assert_eq!(
            stmts[0],
            Stmt::If {
                condition: Expr::BinaryOp {
                    op: ">".to_string(),
                    left: Box::new(Expr::Var("x".to_string())),
                    right: Box::new(Expr::IntLiteral(0)),
                },
                then_branch: Box::new(Stmt::Block(vec![Stmt::Expr(Expr::BinaryOp {
                    op: "=".to_string(),
                    left: Box::new(Expr::Var("y".to_string())),
                    right: Box::new(Expr::IntLiteral(1)),
                })])),
                else_branch: Some(Box::new(Stmt::Block(vec![Stmt::Expr(Expr::BinaryOp {
                    op: "=".to_string(),
                    left: Box::new(Expr::Var("y".to_string())),
                    right: Box::new(Expr::UnaryOp {
                        op: "-".to_string(),
                        expr: Box::new(Expr::IntLiteral(1)),
                    }),
                })]))),
            }
        );
    }

    #[test]
    fn test_parse_while() {
        let src = r#"
            while n > 0 {
                n = n - 1;
            }
        "#;
        let stmts = parse_source(src).unwrap();
        assert_eq!(stmts.len(), 1);
        assert_eq!(
            stmts[0],
            Stmt::While {
                condition: Expr::BinaryOp {
                    op: ">".to_string(),
                    left: Box::new(Expr::Var("n".to_string())),
                    right: Box::new(Expr::IntLiteral(0)),
                },
                body: Box::new(Stmt::Block(vec![Stmt::Expr(Expr::BinaryOp {
                    op: "=".to_string(),
                    left: Box::new(Expr::Var("n".to_string())),
                    right: Box::new(Expr::BinaryOp {
                        op: "-".to_string(),
                        left: Box::new(Expr::Var("n".to_string())),
                        right: Box::new(Expr::IntLiteral(1)),
                    }),
                })])),
            }
        );
    }

    #[test]
    fn test_parse_function_decl() {
        let src = r#"
            fn add(a: int, b: int) int {
                return a + b;
            }
            fn does_absolutely_nothing() void {
                return;
            }
        "#;
        let stmts = parse_source(src).unwrap();
        assert_eq!(stmts.len(), 2);
        assert_eq!(
            stmts[0],
            Stmt::FunctionDecl {
                name: "add".to_string(),
                params: vec![("a".to_string(), Ty::Int), ("b".to_string(), Ty::Int),],
                return_ty: Ty::Int,
                body: Box::new(Stmt::Block(vec![Stmt::Return(Some(Expr::BinaryOp {
                    op: "+".to_string(),
                    left: Box::new(Expr::Var("a".to_string())),
                    right: Box::new(Expr::Var("b".to_string())),
                }))]))
            },
        );
        assert_eq!(
            stmts[1],
            Stmt::FunctionDecl {
                name: "does_absolutely_nothing".to_string(),
                params: vec![],
                return_ty: Ty::Void,
                body: Box::new(Stmt::Block(vec![Stmt::Return(None)]))
            },
        );
    }

    #[test]
    fn test_parse_assign() {
        let src = "x = y = 42;";
        let stmts = parse_source(src).unwrap();
        assert_eq!(stmts.len(), 1);
        assert_eq!(
            stmts[0],
            Stmt::Expr(Expr::BinaryOp {
                op: "=".to_string(),
                left: Box::new(Expr::Var("x".to_string())),
                right: Box::new(Expr::BinaryOp {
                    op: "=".to_string(),
                    left: Box::new(Expr::Var("y".to_string())),
                    right: Box::new(Expr::IntLiteral(42)),
                }),
            })
        );
    }

    #[test]
    fn test_parse_function_call() {
        // Oh boy, currying!
        let src = r#"
            result = adder_factory()()(10, 20);
            result = (*function_pointer)(10, 20);
        "#;
        let stmts = parse_source(src).unwrap();
        assert_eq!(stmts.len(), 2);
        assert_eq!(
            stmts[0],
            Stmt::Expr(Expr::BinaryOp {
                op: "=".to_string(),
                left: Box::new(Expr::Var("result".to_string())),
                right: Box::new(Expr::Call {
                    func: Box::new(Expr::Call {
                        func: Box::new(Expr::Call {
                            func: Box::new(Expr::Var("adder_factory".to_string())),
                            args: vec![],
                        }),
                        args: vec![],
                    }),
                    args: vec![Expr::IntLiteral(10), Expr::IntLiteral(20)],
                }),
            })
        );
        assert_eq!(
            stmts[1],
            Stmt::Expr(Expr::BinaryOp {
                op: "=".to_string(),
                left: Box::new(Expr::Var("result".to_string())),
                right: Box::new(Expr::Call {
                    func: Box::new(Expr::Deref(Box::new(Expr::Var(
                        "function_pointer".to_string()
                    )))),
                    args: vec![Expr::IntLiteral(10), Expr::IntLiteral(20)],
                }),
            })
        );
    }

    #[test]
    fn test_parse_cursed_pointer_shenanigans() {
        let src = r#"
            *off = c;
            *(off + 1) = current_color;
        "#;
        let stmts = parse_source(src).unwrap();
        assert_eq!(stmts.len(), 2);
        assert_eq!(
            stmts[0],
            Stmt::Expr(Expr::BinaryOp {
                op: "=".to_string(),
                left: Box::new(Expr::Deref(Box::new(Expr::Var("off".to_string())))),
                right: Box::new(Expr::Var("c".to_string())),
            })
        );
        assert_eq!(
            stmts[1],
            Stmt::Expr(Expr::BinaryOp {
                op: "=".to_string(),
                left: Box::new(Expr::Deref(Box::new(Expr::BinaryOp {
                    op: "+".to_string(),
                    left: Box::new(Expr::Var("off".to_string())),
                    right: Box::new(Expr::IntLiteral(1)),
                }))),
                right: Box::new(Expr::Var("current_color".to_string())),
            })
        );
    }
}
