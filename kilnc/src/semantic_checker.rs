use std::collections::HashMap;

use crate::parsing::{
    Location, Span,
    ast::{Expr, ExprNode, Stmt, StmtNode, Ty},
};

fn auto_castable(from: &Ty, to: &Ty) -> bool {
    match (from, to) {
        (_, _) if from == to => true,
        (Ty::Ptr(_), Ty::Ptr(_)) => true,
        (Ty::Int, Ty::Ptr(_)) => true,
        (Ty::Ptr(_), Ty::Int) => true,
        _ => false,
    }
}

#[derive(Debug, Clone)]
pub enum SemanticErrorType<'src> {
    // bool indicates whether we are accessing a variable or a function, just for better error
    // messages
    UnknownVariable(String, bool),
    CannotCallNonFunction(String),
    CannotAssignToConst(String),
    DuplicateVariable {
        original_location: Location,
        original_decl: &'src str,
        duplicate_stmt: &'src str,
        is_const: bool,
    },
    DuplicateFunction {
        original_location: Location,
        original_decl: &'src str,
        duplicate_signature: &'src str,
    },
    FunctionCallArgumentCountMismatch {
        func_name: &'src str,
        expected: usize,
        got: usize,
    },
    InvalidDeref(Ty),
    InvalidBinaryOp {
        left: Ty,
        op: String,
        right: Ty,
    },
    InvalidUnaryOp {
        op: String,
        operand: Ty,
    },
    TypeMismatch {
        expected: Ty,
        got: Ty,
    },
    ReturnOutsideFunction,
}

impl<'src> std::fmt::Display for SemanticErrorType<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SemanticErrorType::UnknownVariable(var, is_fn) => {
                if *is_fn {
                    write!(f, "Usage of an unknown function: {}", var)
                } else {
                    write!(f, "Usage of an unknown variable: {}", var)
                }
            }
            SemanticErrorType::CannotCallNonFunction(name) => {
                write!(f, "Attempted to call a non-function value: {}", name)
            }
            SemanticErrorType::CannotAssignToConst(name) => {
                write!(f, "Attempted to assign to const variable: {}", name)
            }
            SemanticErrorType::DuplicateVariable {
                original_location,
                original_decl,
                duplicate_stmt,
                is_const,
            } => {
                let kind = if *is_const { "const" } else { "let" };
                writeln!(
                    f,
                    "\n\tDuplicate {} declaration: `{}`",
                    kind, duplicate_stmt
                )?;
                writeln!(
                    f,
                    "\tOriginal declaration at {}: `{}`",
                    original_location, original_decl
                )
            }
            SemanticErrorType::DuplicateFunction {
                original_location,
                original_decl,
                duplicate_signature,
            } => {
                writeln!(
                    f,
                    "\n\tDuplicate function declaration: `{}`",
                    duplicate_signature
                )?;
                writeln!(
                    f,
                    "\tOriginal declaration at {}: `{}`",
                    original_location, original_decl
                )
            }
            SemanticErrorType::FunctionCallArgumentCountMismatch {
                func_name,
                expected,
                got,
            } => {
                write!(
                    f,
                    "Function `{}` called with wrong number of arguments: expected {}, got {}",
                    func_name, expected, got
                )
            }
            SemanticErrorType::InvalidDeref(ty) => {
                write!(f, "Cannot dereference value of type `{}`", ty)
            }
            SemanticErrorType::InvalidBinaryOp { left, op, right } => {
                write!(
                    f,
                    "Invalid binary operation: cannot apply operator `{}` to types `{}` and `{}`",
                    op, left, right
                )
            }
            SemanticErrorType::InvalidUnaryOp { op, operand } => {
                write!(
                    f,
                    "Invalid unary operation: cannot apply operator `{}` to type `{}`",
                    op, operand
                )
            }
            SemanticErrorType::TypeMismatch { expected, got } => {
                write!(f, "Type mismatch: expected `{}`, got `{}`", expected, got)
            }
            SemanticErrorType::ReturnOutsideFunction => {
                write!(f, "Return was used outside a function.")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SemanticError<'src> {
    pub error_type: SemanticErrorType<'src>,
    pub location: Location,
}

impl<'src> std::fmt::Display for SemanticError<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Semantic error at {}: {}",
            self.location, self.error_type
        )
    }
}

impl<'src> std::error::Error for SemanticError<'src> {}

type SemanticResult<'src, T> = Result<T, SemanticError<'src>>;

struct VariableInfo {
    var_type: Ty,
    is_const: bool,
    source_span: Span,
}

struct FunctionInfo {
    return_type: Ty,
    param_types: Vec<Ty>,
    source_signature_span: Span,
}

enum Symbol {
    Variable(VariableInfo),
    Function(FunctionInfo),
}

struct Environment {
    scopes: Vec<HashMap<String, Symbol>>,
}

impl Environment {
    fn new() -> Self {
        Environment {
            scopes: vec![HashMap::new()],
        }
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.scopes.pop();
        if self.scopes.is_empty() {
            self.scopes.push(HashMap::new());
        }
    }

    fn declare_variable(
        &mut self,
        name: String,
        var_type: Ty,
        is_const: bool,
        span: Span,
    ) -> Result<(), Span> {
        let current_scope = self.scopes.last_mut().unwrap();
        if let Some(existing) = current_scope.get(&name) {
            let existing_span = match existing {
                Symbol::Variable(info) => info.source_span,
                Symbol::Function(info) => info.source_signature_span,
            };
            return Err(existing_span);
        }
        current_scope.insert(
            name,
            Symbol::Variable(VariableInfo {
                var_type,
                is_const,
                source_span: span,
            }),
        );
        Ok(())
    }

    fn declare_function(
        &mut self,
        name: String,
        return_type: Ty,
        param_types: Vec<Ty>,
        signature_span: Span,
    ) -> Result<(), Span> {
        let current_scope = self.scopes.last_mut().unwrap();
        if let Some(existing) = current_scope.get(&name) {
            let existing_span = match existing {
                Symbol::Variable(info) => info.source_span,
                Symbol::Function(info) => info.source_signature_span,
            };
            return Err(existing_span);
        }
        current_scope.insert(
            name,
            Symbol::Function(FunctionInfo {
                return_type,
                param_types,
                source_signature_span: signature_span,
            }),
        );
        Ok(())
    }

    fn lookup_variable(&self, name: &str) -> Option<&VariableInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(Symbol::Variable(info)) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }

    fn lookup_function(&self, name: &str) -> Option<&FunctionInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(Symbol::Function(info)) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }
}

pub struct SemanticChecker<'src, 'ast> {
    ast: &'ast mut [StmtNode],
    src: &'src str,
    env: Environment,
    return_ty: Vec<Ty>,
}

impl<'src, 'ast> SemanticChecker<'src, 'ast> {
    pub fn new(ast: &'ast mut [StmtNode], src: &'src str) -> Self {
        SemanticChecker {
            ast,
            src,
            env: Environment::new(),
            return_ty: Vec::new(),
        }
    }

    pub fn check(&mut self) -> SemanticResult<'src, ()> {
        for stmt in self.ast.iter() {
            if let Stmt::FunctionDecl {
                name,
                params,
                return_ty,
                signature_span,
                ..
            } = &stmt.stmt
            {
                let param_types = params.iter().map(|(_, ty)| ty.clone()).collect();
                if let Err(original_span) = self.env.declare_function(
                    name.clone(),
                    return_ty.clone(),
                    param_types,
                    *signature_span,
                ) {
                    let original_location = original_span.start;
                    let original_decl =
                        self.src[original_span.start_offset..original_span.end_offset].trim();
                    let duplicate_signature =
                        self.src[signature_span.start_offset..signature_span.end_offset].trim();

                    return Err(SemanticError {
                        error_type: SemanticErrorType::DuplicateFunction {
                            original_location,
                            original_decl,
                            duplicate_signature,
                        },
                        location: signature_span.start,
                    });
                }
            }
        }

        // Second pass: check everything else
        let src = self.src;
        let env = &mut self.env;
        let return_ty = &mut self.return_ty;
        for stmt in self.ast.iter_mut() {
            Self::check_stmt(src, env, return_ty, stmt, true)?;
        }

        Ok(())
    }

    fn try_declare_variable(
        src: &'src str,
        env: &mut Environment,
        name: String,
        ty: Ty,
        is_const: bool,
        span: Span,
    ) -> SemanticResult<'src, ()> {
        if let Err(original_span) = env.declare_variable(name, ty, is_const, span) {
            let original_location = original_span.start;
            let original_decl = &src[original_span.start_offset..original_span.end_offset].trim();
            let duplicate_stmt = &src[span.start_offset..span.end_offset].trim();
            return Err(SemanticError {
                error_type: SemanticErrorType::DuplicateVariable {
                    original_location,
                    original_decl,
                    duplicate_stmt,
                    is_const,
                },
                location: span.start,
            });
        }
        Ok(())
    }

    // fn check_stmt(&mut self, stmt: &mut StmtNode, global: bool) -> SemanticResult<'src, ()> {
    fn check_stmt(
        src: &'src str,
        env: &mut Environment,
        ret_ty: &mut Vec<Ty>,
        stmt: &mut StmtNode,
        global: bool,
    ) -> SemanticResult<'src, ()> {
        match &mut stmt.stmt {
            Stmt::Const { name, ty, value } => {
                let ty_found = Self::check_expr(src, env, value)?;
                if !auto_castable(&ty_found, &ty) {
                    return Err(SemanticError {
                        error_type: SemanticErrorType::TypeMismatch {
                            expected: ty.clone(),
                            got: ty_found,
                        },
                        location: value.span.start,
                    });
                }
                Self::try_declare_variable(src, env, name.clone(), ty.clone(), true, stmt.span)?;
            }
            Stmt::Let { name, ty, value } => {
                let ty_found = Self::check_expr(src, env, value)?;
                if !auto_castable(&ty_found, &ty) {
                    return Err(SemanticError {
                        error_type: SemanticErrorType::TypeMismatch {
                            expected: ty.clone(),
                            got: ty_found,
                        },
                        location: value.span.start,
                    });
                }
                Self::try_declare_variable(src, env, name.clone(), ty.clone(), false, stmt.span)?;
            }
            Stmt::Expr(expr) => {
                Self::check_expr(src, env, expr)?;
            }
            Stmt::Block(stmts) => {
                env.enter_scope();
                for stmt in stmts {
                    Self::check_stmt(src, env, ret_ty, stmt, false)?;
                }
                env.exit_scope();
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                Self::check_expr(src, env, condition)?;
                Self::check_stmt(src, env, ret_ty, then_branch, false)?;
                if let Some(else_branch) = else_branch {
                    Self::check_stmt(src, env, ret_ty, else_branch, false)?;
                }
            }
            Stmt::While { condition, body } => {
                Self::check_expr(src, env, condition)?;
                Self::check_stmt(src, env, ret_ty, body, false)?;
            }
            Stmt::FunctionDecl {
                name,
                params,
                return_ty,
                signature_span,
                body,
            } => {
                if global {
                    // We already declared all global functions in the first pass, so we just need
                    // to check the body here.
                    env.enter_scope();
                    for (param_name, param_ty) in params {
                        Self::try_declare_variable(
                            src,
                            env,
                            param_name.clone(),
                            param_ty.clone(),
                            false,
                            *signature_span,
                        )?;
                    }
                    ret_ty.push(return_ty.clone());
                    Self::check_stmt(src, env, ret_ty, body, false)?;
                    ret_ty.pop();
                    env.exit_scope();
                    return Ok(());
                }

                let param_types = params.iter().map(|(_, ty)| ty.clone()).collect();
                if let Err(original_span) = env.declare_function(
                    name.clone(),
                    return_ty.clone(),
                    param_types,
                    *signature_span,
                ) {
                    let original_location = original_span.start;
                    let original_decl =
                        &src[original_span.start_offset..original_span.end_offset].trim();
                    let duplicate_signature =
                        &src[signature_span.start_offset..signature_span.end_offset].trim();
                    return Err(SemanticError {
                        error_type: SemanticErrorType::DuplicateFunction {
                            original_location,
                            original_decl,
                            duplicate_signature,
                        },
                        location: signature_span.start,
                    });
                }
                env.enter_scope();
                for (param_name, param_ty) in params {
                    Self::try_declare_variable(
                        src,
                        env,
                        param_name.clone(),
                        param_ty.clone(),
                        false,
                        *signature_span,
                    )?;
                }
                Self::check_stmt(src, env, ret_ty, body, false)?;
                env.exit_scope();
            }
            Stmt::Return(expr_opt) => {
                if let Some(last_ret) = ret_ty.last().cloned() {
                    let ty = if let Some(expr) = expr_opt {
                        Self::check_expr(src, env, expr)?
                    } else {
                        Ty::Void
                    };
                    if last_ret != ty {
                        return Err(SemanticError {
                            error_type: SemanticErrorType::TypeMismatch {
                                expected: last_ret.clone(),
                                got: ty,
                            },
                            location: stmt.span.start,
                        });
                    }
                } else {
                    return Err(SemanticError {
                        error_type: SemanticErrorType::ReturnOutsideFunction,
                        location: stmt.span.start,
                    });
                }
            }
        }
        Ok(())
    }

    fn check_expr(
        src: &'src str,
        env: &Environment,
        expr: &mut ExprNode,
    ) -> SemanticResult<'src, Ty> {
        let ty = Self::check_expr_inner(src, env, expr)?;
        expr.ty = Some(ty.clone());
        Ok(ty)
    }

    fn check_expr_inner(
        src: &'src str,
        env: &Environment,
        expr: &mut ExprNode,
    ) -> SemanticResult<'src, Ty> {
        match &mut expr.expr {
            Expr::IntLiteral(_) => return Ok(Ty::Int),
            Expr::CharLiteral(_) => return Ok(Ty::Char),
            Expr::StringLiteral(_) => return Ok(Ty::Ptr(Box::new(Ty::Char))),
            Expr::Var(name) => {
                if let Some(var_info) = env.lookup_variable(name.as_str()) {
                    Ok(var_info.var_type.clone())
                } else if let Some(fn_info) = env.lookup_function(name.as_str()) {
                    Ok(Ty::FnPtr {
                        params: fn_info.param_types.clone(),
                        return_ty: Box::new(fn_info.return_type.clone()),
                    })
                } else {
                    Err(SemanticError {
                        error_type: SemanticErrorType::UnknownVariable(name.clone(), false),
                        location: expr.span.start,
                    })
                }
            }
            Expr::BinaryOp { left, op, right } => {
                let left_ty = Self::check_expr(src, env, &mut *left)?;
                let right_ty = Self::check_expr(src, env, &mut *right)?;
                let op = op.as_str();
                match (&left_ty, &right_ty) {
                    (Ty::Int, Ty::Int)
                        if [
                            "+", "-", "*", "/", "%", "==", "!=", "<", ">", "<=", ">=", "&", "|",
                        ]
                        .contains(&op) =>
                    {
                        return Ok(Ty::Int);
                    }
                    (Ty::Char, Ty::Char) if ["==", "!="].contains(&op) => {
                        return Ok(Ty::Int);
                    }
                    (Ty::Ptr(_), Ty::Int) if ["+", "-"].contains(&op) => {
                        return Ok(left_ty);
                    }
                    (Ty::Int, Ty::Ptr(_)) if ["+", "-"].contains(&op) => {
                        return Ok(right_ty);
                    }
                    (Ty::Ptr(_), Ty::Ptr(_))
                        if ["-", "==", "!=", "<", ">", "<=", ">="].contains(&op) =>
                    {
                        return Ok(Ty::Int);
                    }
                    (_, _) if op == "=" && auto_castable(&right_ty, &left_ty) => {
                        // make sure we arent assigning to a const!
                        if let Expr::Var(var_name) = &left.expr {
                            if let Some(var_info) = env.lookup_variable(var_name.as_str()) {
                                if var_info.is_const {
                                    return Err(SemanticError {
                                        error_type: SemanticErrorType::CannotAssignToConst(
                                            var_name.clone(),
                                        ),
                                        location: expr.span.start,
                                    });
                                }
                                return Ok(var_info.var_type.clone());
                            }

                            return Err(SemanticError {
                                error_type: SemanticErrorType::UnknownVariable(
                                    var_name.clone(),
                                    false,
                                ),
                                location: left.span.start,
                            });
                        }
                        Ok(left_ty)
                    }
                    _ => {
                        return Err(SemanticError {
                            error_type: SemanticErrorType::InvalidBinaryOp {
                                left: left_ty,
                                op: op.to_string(),
                                right: right_ty,
                            },
                            location: expr.span.start,
                        });
                    }
                }
            }
            Expr::UnaryOp { op, expr } => {
                let ty = Self::check_expr(src, env, &mut *expr)?;
                let op = op.as_str();
                match (op, &ty) {
                    // + does absolutely nothing but it exists because why not
                    ("+", Ty::Int) => return Ok(Ty::Int),
                    ("-", Ty::Int) => return Ok(Ty::Int),
                    ("!", Ty::Int) => return Ok(Ty::Int),
                    _ => {
                        return Err(SemanticError {
                            error_type: SemanticErrorType::InvalidUnaryOp {
                                op: op.to_string(),
                                operand: ty,
                            },
                            location: expr.span.start,
                        });
                    }
                }
            }
            Expr::Call { func, args } => {
                let func_span = func.span;
                let func_ty = Self::check_expr(src, env, &mut *func)?;
                let arg_tys: Vec<Ty> = args
                    .iter_mut()
                    .map(|arg| Self::check_expr(src, env, arg))
                    .collect::<SemanticResult<'src, Vec<Ty>>>()?;
                match func_ty {
                    Ty::FnPtr { params, return_ty } => {
                        if params.len() != arg_tys.len() {
                            return Err(SemanticError {
                                error_type: SemanticErrorType::FunctionCallArgumentCountMismatch {
                                    func_name: src[func_span.start_offset..func_span.end_offset]
                                        .trim(),
                                    expected: params.len(),
                                    got: arg_tys.len(),
                                },
                                location: func_span.start,
                            });
                        }
                        for (i, (expected, got)) in params.iter().zip(arg_tys.iter()).enumerate() {
                            if expected != got {
                                return Err(SemanticError {
                                    error_type: SemanticErrorType::TypeMismatch {
                                        expected: expected.clone(),
                                        got: got.clone(),
                                    },
                                    location: args[i].span.start,
                                });
                            }
                        }
                        return Ok(*return_ty);
                    }
                    _ => {
                        return Err(SemanticError {
                            error_type: SemanticErrorType::CannotCallNonFunction(
                                src[func_span.start_offset..func_span.end_offset]
                                    .trim()
                                    .to_string(),
                            ),
                            location: func_span.start,
                        });
                    }
                }
            }
            Expr::Deref(inner) => {
                let ty = Self::check_expr(src, env, &mut *inner)?;
                match ty {
                    Ty::Ptr(inner_ty) => return Ok(*inner_ty),
                    // dereferencing an int is basically reading from memory from an address, which
                    // is unsafe, but this entire language is unsafe anyways (beautifully of course)
                    Ty::Int => return Ok(Ty::Int),
                    _ => {
                        return Err(SemanticError {
                            error_type: SemanticErrorType::InvalidDeref(ty),
                            location: expr.span.start,
                        });
                    }
                }
            }
            Expr::Addr(inner) => {
                let ty = Self::check_expr(src, env, &mut *inner)?;
                return Ok(Ty::Ptr(Box::new(ty)));
            }
            Expr::Cast { expr, target_ty } => {
                let _ = Self::check_expr(src, env, &mut *expr)?;
                // TODO: check
                return Ok(target_ty.clone());
            }
        }
    }
}
