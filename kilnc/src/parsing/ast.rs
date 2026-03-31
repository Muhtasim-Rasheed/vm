use crate::parsing::Span;

pub const SYMBOLS_DOUBLE: &[&str] = &["==", "!=", "<=", ">=", "&&", "||"];
pub const SYMBOLS_SINGLE: &[&str] = &[
    "+", "-", "*", "/", "<", ">", "=", "!", "&", "|", "(", ")", "{", "}", ";", ",", ":",
];
pub const KEYWORDS: &[&str] = &[
    "const", "let", "if", "else", "while", "fn", "return", "int", "char", "void", "cast",
];

#[derive(Debug, PartialEq)]
pub struct StmtNode {
    pub stmt: Stmt,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Const {
        name: String,
        ty: Ty,
        value: ExprNode,
    },
    Let {
        name: String,
        ty: Ty,
        value: ExprNode,
    },
    Expr(ExprNode),
    Block(Vec<StmtNode>),
    If {
        condition: ExprNode,
        then_branch: Box<StmtNode>,
        else_branch: Option<Box<StmtNode>>,
    },
    While {
        condition: ExprNode,
        body: Box<StmtNode>,
    },
    FunctionDecl {
        name: String,
        params: Vec<(String, Ty)>,
        return_ty: Ty,
        signature_span: Span,
        body: Box<StmtNode>,
    },
    Return(Option<ExprNode>),
}

#[derive(Debug, PartialEq)]
pub struct ExprNode {
    pub expr: Expr,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    IntLiteral(i32),
    CharLiteral(char),
    StringLiteral(String),
    Var(String),
    BinaryOp {
        op: String,
        left: Box<ExprNode>,
        right: Box<ExprNode>,
    },
    UnaryOp {
        op: String,
        expr: Box<ExprNode>,
    },
    Call {
        func: Box<ExprNode>,
        args: Vec<ExprNode>,
    },
    Deref(Box<ExprNode>),
    Addr(Box<ExprNode>),
    Cast {
        expr: Box<ExprNode>,
        target_ty: Ty,
    },
}

impl StmtNode {
    pub fn new(stmt: Stmt, span: Span) -> Self {
        Self { stmt, span }
    }
}

impl ExprNode {
    pub fn new(expr: Expr, span: Span) -> Self {
        Self { expr, span }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Int,
    Char,
    Void,
    Ptr(Box<Ty>),
    FnPtr { params: Vec<Ty>, return_ty: Box<Ty> },
}

impl std::fmt::Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ty::Int => write!(f, "int"),
            Ty::Char => write!(f, "char"),
            Ty::Void => write!(f, "void"),
            Ty::Ptr(inner) => write!(f, "*{}", inner),
            Ty::FnPtr { params, return_ty } => {
                let params_str = params
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "fn({}) {}", params_str, return_ty)
            }
        }
    }
}
