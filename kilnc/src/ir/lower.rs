use std::collections::HashMap;

use crate::{
    ir::{
        IrFunc, IrGlobalInit, IrInst, IrLocal, IrModule, LabelId, LocalId, TempId, ValueRef,
        registries::{Functions, Globals, StringTable},
    },
    parsing::ast::{Expr, ExprNode, Stmt, StmtNode, Ty},
};

fn get_ty(expr: &ExprNode) -> &Ty {
    expr.ty
        .as_ref()
        .expect("ICE: expression type should have been resolved in semantic analysis")
}

pub struct IrFuncBuilder<'a> {
    name: String,
    strings: &'a mut StringTable,
    globals: &'a Globals,
    other_functions: &'a mut Functions,

    next_temp: TempId,
    next_label: LabelId,
    next_local: LocalId,

    dummy_temp: TempId,
    entry_label: LabelId,

    local_scopes: Vec<HashMap<String, LocalId>>,
    locals: Vec<IrLocal>,
    params: Vec<LocalId>,

    body: Vec<IrInst>,
}

impl<'a> IrFuncBuilder<'a> {
    pub fn new(
        name: impl Into<String>,
        params: &[(String, Ty)],
        strings: &'a mut StringTable,
        globals: &'a Globals,
        other_functions: &'a mut Functions,
        entry_label: LabelId,
    ) -> Self {
        let mut this = Self {
            name: name.into(),
            strings,
            globals,
            other_functions,
            next_temp: 0,
            next_label: 0,
            next_local: 0,
            dummy_temp: 0,
            entry_label,
            local_scopes: vec![HashMap::new()],
            locals: Vec::new(),
            params: Vec::new(),
            body: Vec::new(),
        };

        this.dummy_temp = this.new_temp();

        for (param_name, _) in params {
            this.declare_local(param_name, true);
        }

        this
    }

    fn emit(&mut self, inst: IrInst) {
        self.body.push(inst);
    }

    fn new_temp(&mut self) -> TempId {
        let id = self.next_temp;
        self.next_temp += 1;
        id
    }

    pub fn new_label(&mut self) -> LabelId {
        let id = self.next_label;
        self.next_label += 1;
        id
    }

    pub fn emit_label(&mut self, label: LabelId) {
        self.emit(IrInst::Label(label));
    }

    pub fn enter_scope(&mut self) {
        self.local_scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.local_scopes.pop();
        if self.local_scopes.is_empty() {
            self.local_scopes.push(HashMap::new());
        }
    }

    pub fn declare_local(&mut self, name: impl Into<String>, is_param: bool) -> LocalId {
        let name = name.into();
        let id = self.next_local;
        self.next_local += 1;

        self.local_scopes
            .last_mut()
            .expect("ICE: no local scope present")
            .insert(name.clone(), id);

        self.locals.push(IrLocal { id, name, is_param });

        if is_param {
            self.params.push(id);
        }

        id
    }

    pub fn lookup_value(&self, name: &str) -> Option<ValueRef> {
        for scope in self.local_scopes.iter().rev() {
            if let Some(&id) = scope.get(name) {
                return Some(ValueRef::Local(id));
            }
        }
        if let Some(global) = self.globals.get(name) {
            return Some(ValueRef::Global(global.id));
        }
        if let Some(func) = self.other_functions.get(name) {
            return Some(ValueRef::Func(func.id));
        }
        None
    }

    pub fn lower_stmt(&mut self, stmt: &StmtNode) {
        match &stmt.stmt {
            Stmt::Const { name, ty: _, value } => {
                let local = self.declare_local(name, true);
                let value = self.lower_expr(value);

                self.emit(IrInst::StoreLocal { local, src: value });
            }
            Stmt::Let { name, ty: _, value } => {
                let local = self.declare_local(name, false);
                let value = self.lower_expr(value);

                self.emit(IrInst::StoreLocal { local, src: value });
            }
            Stmt::Expr(expr) => {
                self.lower_expr(expr);
            }
            Stmt::Block(stmts) => {
                self.enter_scope();
                for stmt in stmts {
                    self.lower_stmt(stmt);
                }
                self.exit_scope();
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let else_label = self.new_label();
                let end_label = self.new_label();

                let cond_temp = self.lower_expr(condition);
                self.emit(IrInst::JumpIfZero {
                    cond: cond_temp,
                    target: else_label,
                });

                self.lower_stmt(then_branch);
                self.emit(IrInst::Jump { target: end_label });

                self.emit_label(else_label);
                if let Some(else_branch) = else_branch {
                    self.lower_stmt(else_branch);
                }
                self.emit_label(end_label);
            }
            Stmt::While { condition, body } => {
                let start_label = self.new_label();
                let end_label = self.new_label();

                self.emit_label(start_label);
                let cond_temp = self.lower_expr(condition);
                self.emit(IrInst::JumpIfZero {
                    cond: cond_temp,
                    target: end_label,
                });

                self.lower_stmt(body);
                self.emit(IrInst::Jump {
                    target: start_label,
                });

                self.emit_label(end_label);
            }
            Stmt::FunctionDecl {
                name,
                params,
                return_ty: _,
                signature_span: _,
                body,
            } => {
                let func_label = self.new_label();
                self.emit_label(func_label);

                self.enter_scope();

                for (param_name, _) in params {
                    self.declare_local(param_name, true);
                }

                self.lower_stmt(body);

                self.emit(IrInst::Return { value: None });

                self.exit_scope();

                self.other_functions.declare(name, params.len());
            }
            Stmt::Return(expr) => {
                let value = expr.as_ref().map(|e| self.lower_expr(e));
                self.emit(IrInst::Return { value });
            }
        }
    }

    pub fn lower_expr(&mut self, expr: &ExprNode) -> TempId {
        match &expr.expr {
            Expr::IntLiteral(int) => {
                let dst = self.new_temp();
                self.emit(IrInst::ConstInt { dst, value: *int });
                dst
            }
            Expr::CharLiteral(ch) => {
                let dst = self.new_temp();
                self.emit(IrInst::ConstChar {
                    dst,
                    value: *ch as u8,
                });
                dst
            }
            Expr::StringLiteral(string) => {
                let dst = self.new_temp();
                let id = self.strings.intern(string);
                self.emit(IrInst::ConstString { dst, id });
                dst
            }
            Expr::Var(var) => {
                let dst = self.new_temp();
                let local = self
                    .lookup_value(var)
                    .expect("ICE: unknown variable should have been caught in semantic analysis");

                match local {
                    ValueRef::Local(local_id) => self.emit(IrInst::LoadLocal {
                        dst,
                        local: local_id,
                    }),
                    ValueRef::Global(global_id) => self.emit(IrInst::LoadGlobal {
                        dst,
                        global: global_id,
                    }),
                    ValueRef::Func(func_id) => self.emit(IrInst::AddrOfFunc { dst, func: func_id }),
                }
                dst
            }
            Expr::BinaryOp { op, left, right } if op == "=" => {
                let addr = self.lower_lvalue(left);
                let value = self.lower_expr(right);

                match get_ty(left) {
                    Ty::Char => {
                        self.emit(IrInst::StoreByte { addr, src: value });
                    }
                    _ => {
                        self.emit(IrInst::Store { addr, src: value });
                    }
                }

                value
            }
            Expr::BinaryOp { op, left, right }
                if matches!(
                    (get_ty(left), get_ty(right), op.as_str()),
                    (Ty::Ptr(_), Ty::Int, "+" | "-")
                ) =>
            {
                let dst = self.new_temp();
                let left_temp = self.lower_expr(left);
                let right_temp = self.lower_expr(right);
                let size = match get_ty(left) {
                    Ty::Ptr(inner) => inner.size(),
                    _ => unreachable!(),
                };
                let size_temp = self.new_temp();
                self.emit(IrInst::ConstInt {
                    dst: size_temp,
                    value: size as i32,
                });
                let scaled_right = self.new_temp();
                self.emit(IrInst::BinOp {
                    dst: scaled_right,
                    op: crate::ir::IrBinOp::Mul,
                    left: right_temp,
                    right: size_temp,
                });
                self.emit(IrInst::BinOp {
                    dst,
                    op: if op == "+" {
                        crate::ir::IrBinOp::Add
                    } else {
                        crate::ir::IrBinOp::Sub
                    },
                    left: left_temp,
                    right: scaled_right,
                });
                dst
            }
            Expr::BinaryOp { op, left, right }
                if matches!(
                    (get_ty(left), get_ty(right), op.as_str()),
                    (Ty::Int, Ty::Ptr(_), "+" | "-")
                ) =>
            {
                let dst = self.new_temp();
                let left_temp = self.lower_expr(left);
                let right_temp = self.lower_expr(right);
                let size = match get_ty(right) {
                    Ty::Ptr(inner) => inner.size(),
                    _ => unreachable!(),
                };
                let size_temp = self.new_temp();
                self.emit(IrInst::ConstInt {
                    dst: size_temp,
                    value: size as i32,
                });
                let scaled_left = self.new_temp();
                self.emit(IrInst::BinOp {
                    dst: scaled_left,
                    op: crate::ir::IrBinOp::Mul,
                    left: left_temp,
                    right: size_temp,
                });
                self.emit(IrInst::BinOp {
                    dst,
                    op: if op == "+" {
                        crate::ir::IrBinOp::Add
                    } else {
                        crate::ir::IrBinOp::Sub
                    },
                    left: scaled_left,
                    right: right_temp,
                });
                dst
            }
            Expr::BinaryOp { op, left, right }
                if get_ty(left) == get_ty(right)
                    && matches!(get_ty(left), Ty::Ptr(_))
                    && op == "-" =>
            {
                let dst = self.new_temp();
                let left_temp = self.lower_expr(left);
                let right_temp = self.lower_expr(right);
                let size = match get_ty(left) {
                    Ty::Ptr(inner) => inner.size(),
                    _ => unreachable!(),
                };
                let size_temp = self.new_temp();
                self.emit(IrInst::ConstInt {
                    dst: size_temp,
                    value: size as i32,
                });
                let byte_diff = self.new_temp();
                self.emit(IrInst::BinOp {
                    dst: byte_diff,
                    op: crate::ir::IrBinOp::Sub,
                    left: left_temp,
                    right: right_temp,
                });
                self.emit(IrInst::BinOp {
                    dst,
                    op: crate::ir::IrBinOp::Div,
                    left: byte_diff,
                    right: size_temp,
                });
                dst
            }
            Expr::BinaryOp { op, left, right } => {
                let dst = self.new_temp();
                let left = self.lower_expr(left);
                let right = self.lower_expr(right);
                let ir_op = match op.as_str() {
                    "+" => crate::ir::IrBinOp::Add,
                    "-" => crate::ir::IrBinOp::Sub,
                    "*" => crate::ir::IrBinOp::Mul,
                    "/" => crate::ir::IrBinOp::Div,
                    "%" => crate::ir::IrBinOp::Mod,
                    "==" => crate::ir::IrBinOp::Eq,
                    "!=" => crate::ir::IrBinOp::Ne,
                    "<" => crate::ir::IrBinOp::Lt,
                    "<=" => crate::ir::IrBinOp::Le,
                    ">" => crate::ir::IrBinOp::Gt,
                    ">=" => crate::ir::IrBinOp::Ge,
                    "&" => crate::ir::IrBinOp::BitAnd,
                    "|" => crate::ir::IrBinOp::BitOr,
                    _ => panic!("ICE: unknown binary operator: {}", op),
                };
                self.emit(IrInst::BinOp {
                    dst,
                    op: ir_op,
                    left,
                    right,
                });
                dst
            }
            Expr::UnaryOp { op, expr } => {
                let dst = self.new_temp();
                let src = self.lower_expr(expr);
                match op.as_str() {
                    "+" => self.emit(IrInst::Copy { dst, src }),
                    "-" => self.emit(IrInst::UnaryOp {
                        dst,
                        src,
                        op: crate::ir::IrUnaryOp::Neg,
                    }),
                    "!" => self.emit(IrInst::UnaryOp {
                        dst,
                        src,
                        op: crate::ir::IrUnaryOp::Not,
                    }),
                    _ => panic!("ICE: unknown unary operator: {}", op),
                }
                dst
            }
            Expr::Call { func, args } => {
                let func_ptr = self.lower_expr(func);
                let arg_temps = args.iter().map(|arg| self.lower_expr(arg)).collect();
                let dst = self.new_temp();
                match get_ty(expr) {
                    Ty::Void => {
                        self.emit(IrInst::Call {
                            dst: None,
                            func: func_ptr,
                            args: arg_temps,
                        });
                        self.dummy_temp
                    }
                    _ => {
                        self.emit(IrInst::Call {
                            dst: Some(dst),
                            func: func_ptr,
                            args: arg_temps,
                        });
                        dst
                    }
                }
            }
            Expr::Deref(addr) => {
                let dst = self.new_temp();
                let addr = self.lower_expr(addr);
                match get_ty(expr) {
                    Ty::Char => self.emit(IrInst::LoadByte { dst, addr }),
                    _ => self.emit(IrInst::Load { dst, addr }),
                }
                dst
            }
            Expr::Addr(inner) => self.lower_lvalue(inner),
            Expr::Cast { expr, target_ty: _ } => {
                let dst = self.new_temp();
                let src = self.lower_expr(expr);
                self.emit(IrInst::Copy { dst, src });
                dst
            }
        }
    }

    fn lower_lvalue(&mut self, expr: &ExprNode) -> TempId {
        match &expr.expr {
            Expr::Var(var) => {
                let dst = self.new_temp();
                let local = self
                    .lookup_value(var)
                    .expect("ICE: Unknown variable handled in semantic checker.");
                match local {
                    ValueRef::Local(local_id) => self.emit(IrInst::AddrOfLocal {
                        dst,
                        local: local_id,
                    }),
                    ValueRef::Global(global_id) => self.emit(IrInst::AddrOfGlobal {
                        dst,
                        global: global_id,
                    }),
                    ValueRef::Func(func_id) => self.emit(IrInst::AddrOfFunc { dst, func: func_id }),
                }
                dst
            }
            Expr::Deref(inner) => self.lower_expr(inner),
            _ => panic!("ICE: expression is not an lvalue: {:?}", expr.expr),
        }
    }

    pub fn finish(self) -> IrFunc {
        IrFunc {
            name: self.name,
            params: self.params,
            locals: self.locals,
            body: self.body,
            entry_label: self.entry_label,
            temp_count: self.next_temp,
            label_count: self.next_label,
        }
    }
}

pub struct IrModuleBuilder<'a> {
    strings: StringTable,
    globals: Globals,
    functions: Functions,

    func_bodies: Vec<IrFunc>,
    ast: &'a [StmtNode],
}

impl<'a> IrModuleBuilder<'a> {
    pub fn new(ast: &'a [StmtNode]) -> Self {
        Self {
            strings: StringTable::default(),
            globals: Globals::default(),
            functions: Functions::default(),
            func_bodies: Vec::new(),
            ast,
        }
    }

    pub fn lower(mut self) -> IrModule {
        self.collect_globals();
        self.collect_function_decls();

        for stmt in self.ast {
            if matches!(stmt.stmt, Stmt::FunctionDecl { .. }) {
                self.lower_function(stmt);
            }
        }

        IrModule {
            strings: self.strings,
            globals: self.globals,
            functions: self.functions,
            function_bodies: self.func_bodies,
        }
    }

    fn const_expr(&mut self, expr: &ExprNode) -> IrGlobalInit {
        match &expr.expr {
            Expr::IntLiteral(i) => IrGlobalInit::Int(*i),
            Expr::CharLiteral(c) => IrGlobalInit::Char(*c as u8),
            Expr::StringLiteral(s) => {
                let id = self.strings.intern(s);
                IrGlobalInit::StringPtr(id)
            }
            Expr::BinaryOp { op, left, right } if op == "+" => {
                let left = self.const_expr(left);
                let right = self.const_expr(right);
                match (left, right) {
                    (IrGlobalInit::Int(l), IrGlobalInit::Int(r)) => IrGlobalInit::Int(l + r),
                    (IrGlobalInit::Char(l), IrGlobalInit::Char(r)) => {
                        IrGlobalInit::Char(l.wrapping_add(r))
                    }
                    _ => panic!("invalid operands for + in global initializer"),
                }
            }
            Expr::BinaryOp { op, left, right } if op == "-" => {
                let left = self.const_expr(left);
                let right = self.const_expr(right);
                match (left, right) {
                    (IrGlobalInit::Int(l), IrGlobalInit::Int(r)) => IrGlobalInit::Int(l - r),
                    (IrGlobalInit::Char(l), IrGlobalInit::Char(r)) => {
                        IrGlobalInit::Char(l.wrapping_sub(r))
                    }
                    _ => panic!("invalid operands for - in global initializer"),
                }
            }
            Expr::BinaryOp { op, left, right } if op == "*" => {
                let left = self.const_expr(left);
                let right = self.const_expr(right);
                match (left, right) {
                    (IrGlobalInit::Int(l), IrGlobalInit::Int(r)) => IrGlobalInit::Int(l * r),
                    (IrGlobalInit::Char(l), IrGlobalInit::Char(r)) => {
                        IrGlobalInit::Char(l.wrapping_mul(r))
                    }
                    _ => panic!("invalid operands for * in global initializer"),
                }
            }
            Expr::BinaryOp { op, left, right } if op == "/" => {
                let left = self.const_expr(left);
                let right = self.const_expr(right);
                match (left, right) {
                    (IrGlobalInit::Int(l), IrGlobalInit::Int(r)) => IrGlobalInit::Int(l / r),
                    (IrGlobalInit::Char(l), IrGlobalInit::Char(r)) => {
                        IrGlobalInit::Char(l.wrapping_div(r))
                    }
                    _ => panic!("invalid operands for / in global initializer"),
                }
            }
            Expr::BinaryOp { op, left, right } if op == "%" => {
                let left = self.const_expr(left);
                let right = self.const_expr(right);
                match (left, right) {
                    (IrGlobalInit::Int(l), IrGlobalInit::Int(r)) => IrGlobalInit::Int(l % r),
                    (IrGlobalInit::Char(l), IrGlobalInit::Char(r)) => {
                        IrGlobalInit::Char(l.wrapping_rem(r))
                    }
                    _ => panic!("invalid operands for % in global initializer"),
                }
            }
            Expr::BinaryOp { op, left, right } if op == "&" => {
                let left = self.const_expr(left);
                let right = self.const_expr(right);
                match (left, right) {
                    (IrGlobalInit::Int(l), IrGlobalInit::Int(r)) => IrGlobalInit::Int(l & r),
                    (IrGlobalInit::Char(l), IrGlobalInit::Char(r)) => IrGlobalInit::Char(l & r),
                    _ => panic!("invalid operands for & in global initializer"),
                }
            }
            Expr::BinaryOp { op, left, right } if op == "|" => {
                let left = self.const_expr(left);
                let right = self.const_expr(right);
                match (left, right) {
                    (IrGlobalInit::Int(l), IrGlobalInit::Int(r)) => IrGlobalInit::Int(l | r),
                    (IrGlobalInit::Char(l), IrGlobalInit::Char(r)) => IrGlobalInit::Char(l | r),
                    _ => panic!("invalid operands for | in global initializer"),
                }
            }
            Expr::Cast { expr, target_ty } if matches!(target_ty, Ty::Int | Ty::Char) => {
                let value = self.const_expr(expr);
                match (value, target_ty) {
                    (IrGlobalInit::Int(i), Ty::Int) => IrGlobalInit::Int(i),
                    (IrGlobalInit::Int(i), Ty::Char) => IrGlobalInit::Char(i as u8),
                    (IrGlobalInit::Char(c), Ty::Char) => IrGlobalInit::Char(c),
                    (IrGlobalInit::Char(c), Ty::Int) => IrGlobalInit::Int(c as i32),
                    _ => panic!("invalid cast in global initializer"),
                }
            }
            _ => panic!("global must be constant or constant expression"),
        }
    }

    fn collect_globals(&mut self) {
        for stmt in self.ast {
            match &stmt.stmt {
                Stmt::Const { name, value, .. } => {
                    let init = self.const_expr(value);
                    self.globals.declare(name, init, true);
                }
                Stmt::Let { name, value, .. } => {
                    let init = self.const_expr(value);
                    self.globals.declare(name, init, false);
                }
                _ => {}
            }
        }
    }

    fn collect_function_decls(&mut self) {
        for stmt in self.ast {
            if let Stmt::FunctionDecl { name, params, .. } = &stmt.stmt {
                self.functions.declare(name, params.len());
            }
        }
    }

    fn lower_function(&mut self, stmt: &StmtNode) {
        let (name, params, body) = match &stmt.stmt {
            Stmt::FunctionDecl {
                name, params, body, ..
            } => (name, params, body),
            _ => unreachable!(),
        };

        let func_id = self.functions.get(name).unwrap().id;

        let entry_label = self.func_bodies.len() as LabelId;

        let mut builder = IrFuncBuilder::new(
            name.clone(),
            params,
            &mut self.strings,
            &self.globals,
            &mut self.functions,
            entry_label,
        );

        builder.lower_stmt(body);
        let func = builder.finish();

        self.func_bodies.push(func);

        self.functions.set_entry_label(func_id, entry_label);
    }
}
