pub const SYMBOLS_DOUBLE: &[&str] = &["==", "!=", "<=", ">=", "&&", "||"];
pub const SYMBOLS_SINGLE: &[&str] = &[
    "+", "-", "*", "/", "<", ">", "=", "!", "&", "|", "(", ")", "{", "}", ";", ",", ":",
];
pub const KEYWORDS: &[&str] = &[
    "const", "let", "if", "else", "while", "fn", "return", "int", "char", "void",
];

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Const {
        name: String,
        ty: Ty,
        value: Expr,
    },
    Let {
        name: String,
        ty: Ty,
        value: Expr,
    },
    Expr(Expr),
    Block(Vec<Stmt>),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    FunctionDecl {
        name: String,
        params: Vec<(String, Ty)>,
        return_ty: Ty,
        body: Box<Stmt>,
    },
    Return(Option<Expr>),
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    IntLiteral(i32),
    CharLiteral(char),
    StringLiteral(String),
    Var(String),
    BinaryOp {
        op: String,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryOp {
        op: String,
        expr: Box<Expr>,
    },
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
    Deref(Box<Expr>),
    Addr(Box<Expr>),
}

#[derive(Debug, PartialEq)]
pub enum Ty {
    Int,
    Char,
    Void,
    Ptr(Box<Ty>),
}

impl std::fmt::Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ty::Int => write!(f, "int"),
            Ty::Char => write!(f, "char"),
            Ty::Void => write!(f, "void"),
            Ty::Ptr(inner) => write!(f, "*{}", inner),
        }
    }
}
