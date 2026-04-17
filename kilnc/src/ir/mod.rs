use crate::ir::registries::{Functions, Globals, StringTable};

pub mod lower;
pub mod registries;

pub type TempId = u32;
pub type LabelId = u32;
pub type LocalId = u32;
pub type GlobalId = u32;
pub type FuncId = u32;

#[derive(Debug, Clone, Copy)]
pub enum IrBinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    BitAnd,
    BitOr,
}

#[derive(Debug, Clone, Copy)]
pub enum IrUnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
pub enum IrInst {
    ConstInt {
        dst: TempId,
        value: i32,
    },
    ConstChar {
        dst: TempId,
        value: u8,
    },
    ConstString {
        dst: TempId,
        id: usize,
    },
    Copy {
        dst: TempId,
        src: TempId,
    },
    BinOp {
        dst: TempId,
        op: IrBinOp,
        left: TempId,
        right: TempId,
    },
    UnaryOp {
        dst: TempId,
        op: IrUnaryOp,
        src: TempId,
    },
    Load {
        dst: TempId,
        addr: TempId,
    },
    Store {
        addr: TempId,
        src: TempId,
    },
    LoadByte {
        dst: TempId,
        addr: TempId,
    },
    StoreByte {
        addr: TempId,
        src: TempId,
    },
    LoadLocal {
        dst: TempId,
        local: LocalId,
    },
    StoreLocal {
        local: LocalId,
        src: TempId,
    },
    AddrOfLocal {
        dst: TempId,
        local: LocalId,
    },
    LoadGlobal {
        dst: TempId,
        global: GlobalId,
    },
    StoreGlobal {
        global: GlobalId,
        src: TempId,
    },
    AddrOfGlobal {
        dst: TempId,
        global: GlobalId,
    },
    AddrOfFunc {
        dst: TempId,
        func: FuncId,
    },
    Jump {
        target: LabelId,
    },
    JumpIfZero {
        cond: TempId,
        target: LabelId,
    },
    Call {
        dst: Option<TempId>,
        func: TempId,
        args: Vec<TempId>,
    },
    Return {
        value: Option<TempId>,
    },
    Label(LabelId),
}

#[derive(Debug, Clone)]
pub struct IrFunc {
    pub name: String,
    pub params: Vec<LocalId>,
    pub locals: Vec<IrLocal>,
    pub body: Vec<IrInst>,
    pub entry_label: LabelId,
    pub temp_count: TempId,
    pub label_count: LabelId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IrFuncDecl {
    pub id: FuncId,
    pub name: String,
    pub param_count: usize,
    pub entry_label: LabelId,
}

#[derive(Debug, Clone)]
pub struct IrLocal {
    pub id: LocalId,
    pub name: String,
    pub is_param: bool,
}

#[derive(Debug, Clone)]
pub struct IrGlobal {
    pub name: String,
    pub id: GlobalId,
    pub init: IrGlobalInit,
    pub is_const: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum ValueRef {
    Local(LocalId),
    Global(GlobalId),
    Func(FuncId),
}

#[derive(Debug, Clone)]
pub enum IrGlobalInit {
    Int(i32),
    Char(u8),
    StringPtr(usize),
    Zero,
}

#[derive(Debug, Clone)]
pub struct IrModule {
    pub strings: StringTable,
    pub globals: Globals,
    pub functions: Functions,
    pub function_bodies: Vec<IrFunc>,
}
