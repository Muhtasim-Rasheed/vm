use crate::ir::{IrBinOp, IrFunc, IrGlobalInit, IrInst, IrModule, IrUnaryOp, LabelId, LocalId, TempId};
use std::collections::{HashMap, HashSet};

pub struct StackFrame {
    temp_offsets: HashMap<TempId, i32>,
    next_offset: i32,
}

impl StackFrame {
    pub fn new() -> Self {
        Self {
            temp_offsets: HashMap::new(),
            next_offset: 0,
        }
    }

    pub fn reset(&mut self) {
        self.temp_offsets.clear();
        self.next_offset = 0;
    }

    pub fn alloc_temp(&mut self, temp: TempId) -> i32 {
        if let Some(&off) = self.temp_offsets.get(&temp) {
            return off;
        }

        self.next_offset -= 4;
        let off = self.next_offset;
        self.temp_offsets.insert(temp, off);
        off
    }

    pub fn get(&self, temp: TempId) -> i32 {
        *self.temp_offsets.get(&temp).expect("temp not allocated")
    }

    pub fn total_size(&self) -> i32 {
        self.next_offset
    }
}

pub struct CodeGenerator {
    ir: IrModule,
    out: Box<dyn std::io::Write>,

    stack_frame: StackFrame,
}

impl CodeGenerator {
    pub fn new(ir: IrModule, out: Box<dyn std::io::Write>) -> Self {
        Self {
            ir,
            out,
            stack_frame: StackFrame::new(),
        }
    }

    pub fn lower(&mut self) -> std::io::Result<()> {
        self.lower_funcs()?;
        self.lower_strings()?;
        self.lower_globals()?;

        Ok(())
    }

    fn emit_label(&mut self, label: &str) -> std::io::Result<()> {
        writeln!(self.out, ".label {}", label)
    }

    fn emit_label_id(&mut self, func_label: &str, label: LabelId) -> std::io::Result<()> {
        writeln!(self.out, ".label {}L{}", func_label, label)
    }

    fn emit_inst(&mut self, inst: &str) -> std::io::Result<()> {
        writeln!(self.out, "\t{}", inst)
    }

    fn compute_frame_layout(&self, func: &IrFunc) -> HashMap<LocalId, i32> {
        let mut map = HashMap::new();

        // start AFTER temps
        let mut offset = self.stack_frame.total_size();

        // locals (negative)
        for local in &func.locals {
            offset -= 4;
            map.insert(local.id, -offset);
        }

        // args (positive)
        for (i, arg) in func.params.iter().enumerate() {
            let offset = 12 + (i as i32 * 4);
            map.insert(*arg, offset);
        }

        map
    }

    fn collect_temps(func: &IrFunc) -> HashSet<TempId> {
        let mut temps = HashSet::new();

        for inst in &func.body {
            match inst {
                IrInst::ConstInt { dst, .. } => { temps.insert(*dst); }
                IrInst::ConstChar { dst, .. } => { temps.insert(*dst); }
                IrInst::ConstString { dst, .. } => { temps.insert(*dst); }
                IrInst::Copy { dst, src } => {
                    temps.insert(*dst);
                    temps.insert(*src);
                }
                IrInst::BinOp { dst, left, right, .. } => {
                    temps.insert(*dst);
                    temps.insert(*left);
                    temps.insert(*right);
                }
                IrInst::UnaryOp { dst, src, .. } => {
                    temps.insert(*dst);
                    temps.insert(*src);
                }
                IrInst::Load { dst, addr } => {
                    temps.insert(*dst);
                    temps.insert(*addr);
                }
                IrInst::Store { addr, src } => {
                    temps.insert(*addr);
                    temps.insert(*src);
                }
                IrInst::LoadByte { dst, addr } => {
                    temps.insert(*dst);
                    temps.insert(*addr);
                }
                IrInst::StoreByte { addr, src } => {
                    temps.insert(*addr);
                    temps.insert(*src);
                }
                IrInst::AddrOfLocal { dst, .. } => { temps.insert(*dst); }
                IrInst::LoadLocal { dst, .. } => { temps.insert(*dst); }
                IrInst::StoreLocal { src, .. } => { temps.insert(*src); }
                IrInst::LoadGlobal { dst, .. } => { temps.insert(*dst); }
                IrInst::StoreGlobal { src, .. } => { temps.insert(*src); }
                IrInst::AddrOfGlobal { dst, .. } => { temps.insert(*dst); }
                IrInst::AddrOfFunc { dst, .. } => { temps.insert(*dst); }

                IrInst::Jump { .. }
                | IrInst::JumpIfZero { .. }
                | IrInst::Return { .. }
                | IrInst::Label(_) => {}

                IrInst::Call { dst, func, args } => {
                    temps.insert(*func);
                    for a in args {
                        temps.insert(*a);
                    }
                    if let Some(d) = dst {
                        temps.insert(*d);
                    }
                }
            }
        }

        temps
    }

    fn lower_globals(&mut self) -> std::io::Result<()> {
        let mut out = String::new();
        for (global_id, global) in self.ir.globals.iter() {
            let global_label = self.ir.globals.label_for(global_id);
            match &global.init {
                IrGlobalInit::Int(value) => {
                    out.push_str(&format!(".label {} .word {}\n", global_label, value));
                }
                IrGlobalInit::Char(value) => {
                    out.push_str(&format!(".label {} .data {}\n", global_label, *value as u32));
                }
                IrGlobalInit::StringPtr(string_id) => {
                    let string_label = self.ir.strings.label_for(*string_id);
                    out.push_str(&format!(".label {} .word {}\n", global_label, string_label));
                }
                IrGlobalInit::Zero => {
                    out.push_str(&format!(".label {} .word 0\n", global_label));
                }
            }
        }
        write!(self.out, "{}", out)?;

        Ok(())
    }

    fn lower_strings(&mut self) -> std::io::Result<()> {
        let mut out = String::new();
        for (string_id, string) in self.ir.strings.iter() {
            let string_label = self.ir.strings.label_for(string_id);
            out.push_str(&format!(".label {} .data \"{}\", 0\n", string_label, string));
        }
        write!(self.out, "{}", out)?;

        Ok(())
    }

    fn lower_funcs(&mut self) -> std::io::Result<()> {
        let mut funcs = Vec::new();
        for (func_id, _) in self.ir.functions.iter().map(|v| (v.0, v.1.clone())) {
            funcs.push(self.ir.function_bodies[func_id as usize].clone());
        }
        let start_func = funcs.iter().find(|f| f.name == "_start").expect("no _start function");
        self.lower_start(start_func)?;
        for func in funcs {
            if func.name != "_start" {
                self.lower_func(&func)?;
            }
        }
        Ok(())
    }

    fn fn_prologue(&mut self, func: &IrFunc) -> std::io::Result<HashMap<LocalId, i32>> {
        let func_label = self.ir.functions.label_for(func.entry_label);
        self.emit_label(&func_label)?;

        self.stack_frame.reset();

        let mut temps = Self::collect_temps(func).into_iter().collect::<Vec<_>>();
        temps.sort_by_key(|t| *t);
        for t in temps.into_iter().rev() {
            self.stack_frame.alloc_temp(t);
        }
        let frame_layout = self.compute_frame_layout(func);
        let temp_size = self.stack_frame.total_size();
        let local_size = func.locals.len() as i32 * 4;
        let frame_size = temp_size + local_size;

        self.emit_inst(&format!("push bp"))?;
        self.emit_inst(&format!("mov bp, sp"))?;
        if frame_size > 0 {
            self.emit_inst(&format!("subi sp, sp, {}", frame_size))?;
        }

        Ok(frame_layout)
    }

    fn lower_start(&mut self, func: &IrFunc) -> std::io::Result<()> {
        let func_label = self.ir.functions.label_for(func.entry_label);
        let frame = self.fn_prologue(func)?;

        for inst in &func.body {
            self.lower_inst(inst, &frame, &func_label, None)?;
        }

        self.emit_inst(&format!("halt"))?;

        Ok(())
    }

    fn lower_func(&mut self, func: &IrFunc) -> std::io::Result<()> {
        let func_label = self.ir.functions.label_for(func.entry_label);
        let exit_label = format!("{}_exit", func_label);
        let frame = self.fn_prologue(func)?;

        for inst in &func.body {
            self.lower_inst(inst, &frame, &func_label, Some(&exit_label))?;
        }

        self.emit_label(&exit_label)?;
        self.emit_inst("mov sp, bp")?;
        self.emit_inst("pop bp")?;
        self.emit_inst("ret")?;

        Ok(())
    }

    fn addr(offset: i32) -> String {
        format!("[bp {:+}]", offset)
    }

    fn lower_inst(
        &mut self,
        inst: &IrInst,
        frame: &HashMap<LocalId, i32>,
        func_label: &str,
        exit_label: Option<&str>,
    ) -> std::io::Result<()> {
        match inst {
            IrInst::ConstInt { dst, value } => {
                // movi r0, value
                // store [bp - offset], r0
                let temp_offset = self.stack_frame.get(*dst);
                self.emit_inst(&format!("movi r0, {}", value))?;
                self.emit_inst(&format!("store {}, r0", Self::addr(temp_offset)))?;
            }
            IrInst::ConstChar { dst, value } => {
                // movi r0, value
                // storeb [bp - offset], r0
                let temp_offset = self.stack_frame.get(*dst);
                self.emit_inst(&format!("movi r0, {}", *value as u32))?;
                self.emit_inst(&format!("storeb {}, r0", Self::addr(temp_offset)))?;
            }
            IrInst::ConstString { dst, id } => {
                // movi r0, string_label
                // store [bp - offset], r0
                let temp_offset = self.stack_frame.get(*dst);
                let string_label = self.ir.strings.label_for(*id);
                self.emit_inst(&format!("movi r0, {}", string_label))?;
                self.emit_inst(&format!("store {}, r0", Self::addr(temp_offset)))?;
            }
            IrInst::Copy { dst, src } => {
                // load r0, [bp - src_offset]
                // store [bp - dst_offset], r0
                let src_offset = self.stack_frame.get(*src);
                let dst_offset = self.stack_frame.get(*dst);
                if src_offset == dst_offset {
                    // If src and dst are the same, we can skip the load/store
                    return Ok(());
                }
                self.emit_inst(&format!("load r0, {}", Self::addr(src_offset)))?;
                self.emit_inst(&format!("store {}, r0", Self::addr(dst_offset)))?;
            }
            IrInst::BinOp {
                dst,
                op,
                left,
                right,
            } => {
                // load r0, [bp - left_offset]
                // load r1, [bp - right_offset]
                // ; perform operation
                // store [bp - dst_offset], r0
                let left_offset = self.stack_frame.get(*left);
                let right_offset = self.stack_frame.get(*right);
                let dst_offset = self.stack_frame.get(*dst);
                self.emit_inst(&format!("load r0, {}", Self::addr(left_offset)))?;
                self.emit_inst(&format!("load r1, {}", Self::addr(right_offset)))?;
                match op {
                    IrBinOp::Add => self.emit_inst("add r0, r0, r1")?,
                    IrBinOp::Sub => self.emit_inst("sub r0, r0, r1")?,
                    IrBinOp::Mul => self.emit_inst("mul r0, r0, r1")?,
                    IrBinOp::Div => self.emit_inst("div r0, r0, r1")?,
                    IrBinOp::Mod => self.emit_inst("mod r0, r0, r1")?,
                    IrBinOp::Eq | IrBinOp::Ne | IrBinOp::Lt | IrBinOp::Gt | IrBinOp::Le | IrBinOp::Ge => {
                        self.emit_inst("cmp r0, r1")?;
                        let set_inst = match op {
                            IrBinOp::Eq => "sete",
                            IrBinOp::Ne => "setne",
                            IrBinOp::Lt => "setl",
                            IrBinOp::Gt => "setg",
                            IrBinOp::Le => "setle",
                            IrBinOp::Ge => "setge",
                            _ => unreachable!(),
                        };
                        self.emit_inst(&format!("{} r0", set_inst))?;
                    }
                    IrBinOp::BitAnd => self.emit_inst("and r0, r0, r1")?,
                    IrBinOp::BitOr => self.emit_inst("or r0, r0, r1")?,
                }
                self.emit_inst(&format!("store {}, r0", Self::addr(dst_offset)))?;
            }
            IrInst::UnaryOp { dst, op, src } => {
                // load r0, [bp - src_offset]
                // ; perform operation
                // store [bp - dst_offset], r0
                let src_offset = self.stack_frame.get(*src);
                let dst_offset = self.stack_frame.get(*dst);
                self.emit_inst(&format!("load r0, {}", Self::addr(src_offset)))?;
                match op {
                    IrUnaryOp::Neg => {
                        self.emit_inst("not r0")?;
                        self.emit_inst("addi r0, r0, 1")?;
                    }
                    IrUnaryOp::Not => {
                        self.emit_inst("not r0")?;
                    }
                }
                self.emit_inst(&format!("store {}, r0", Self::addr(dst_offset)))?;
            }
            IrInst::Load { dst, addr } => {
                // load r0, [bp - addr_offset]
                // load r0, [r0]
                // store [bp - dst_offset], r0
                let addr_offset = self.stack_frame.get(*addr);
                let dst_offset = self.stack_frame.get(*dst);
                self.emit_inst(&format!("load r0, {}", Self::addr(addr_offset)))?;
                self.emit_inst("load r0, [r0]")?;
                self.emit_inst(&format!("store {}, r0", Self::addr(dst_offset)))?;
            }
            IrInst::Store { addr, src } => {
                // load r0, [bp - src_offset]
                // load r1, [bp - addr_offset]
                // store [r1], r0
                let addr_offset = self.stack_frame.get(*addr);
                let src_offset = self.stack_frame.get(*src);
                self.emit_inst(&format!("load r0, {}", Self::addr(src_offset)))?;
                self.emit_inst(&format!("load r1, {}", Self::addr(addr_offset)))?;
                self.emit_inst("store [r1], r0")?;
            }
            IrInst::LoadByte { dst, addr } => {
                // load r0, [bp - addr_offset]
                // loadb r0, [r0]
                // store [bp - dst_offset], r0
                let addr_offset = self.stack_frame.get(*addr);
                let dst_offset = self.stack_frame.get(*dst);
                self.emit_inst(&format!("load r0, {}", Self::addr(addr_offset)))?;
                self.emit_inst("loadb r0, [r0]")?;
                self.emit_inst(&format!("store {}, r0", Self::addr(dst_offset)))?;
            }
            IrInst::StoreByte { addr, src } => {
                // load r0, [bp - src_offset]
                // load r1, [bp - addr_offset]
                // storeb [r1], r0
                let addr_offset = self.stack_frame.get(*addr);
                let src_offset = self.stack_frame.get(*src);
                self.emit_inst(&format!("load r0, {}", Self::addr(src_offset)))?;
                self.emit_inst(&format!("load r1, {}", Self::addr(addr_offset)))?;
                self.emit_inst("storeb [r1], r0")?;
            }
            IrInst::LoadLocal { dst, local } => {
                // load r0, [bp - local_offset]
                // store [bp - dst_offset], r0
                let dst_offset = self.stack_frame.get(*dst);
                let local_offset = frame.get(local).expect("local not found");
                if dst_offset == *local_offset {
                    // If dst and local are the same, we can skip the load/store
                    return Ok(());
                }
                self.emit_inst(&format!("load r0, {}", Self::addr(*local_offset)))?;
                self.emit_inst(&format!("store {}, r0", Self::addr(dst_offset)))?;
            }
            IrInst::StoreLocal { local, src } => {
                // load r0, [bp - src_offset]
                // store [bp - local_offset], r0
                let src_offset = self.stack_frame.get(*src);
                let local_offset = frame.get(local).expect("local not found");
                if src_offset == *local_offset {
                    // If src and local are the same, we can skip the load/store
                    return Ok(());
                }
                self.emit_inst(&format!("load r0, {}", Self::addr(src_offset)))?;
                self.emit_inst(&format!("store {}, r0", Self::addr(*local_offset)))?;
            }
            IrInst::AddrOfLocal { dst, local } => {
                // lea r0, [bp - local_offset]
                // store [bp - dst_offset], r0
                let dst_offset = self.stack_frame.get(*dst);
                let local_offset = frame.get(local).expect("local not found");
                self.emit_inst(&format!("lea r0, {}", Self::addr(*local_offset)))?;
                self.emit_inst(&format!("store {}, r0", Self::addr(dst_offset)))?;
            }
            IrInst::LoadGlobal { dst, global } => {
                // laod r0, [global_label]
                // store [bp - dst_offset], r0
                let dst_offset = self.stack_frame.get(*dst);
                let global_name = self.ir.globals.label_for(*global);
                self.emit_inst(&format!("load r0, [{}]", global_name))?;
                self.emit_inst(&format!("store {}, r0", Self::addr(dst_offset)))?;
            }
            IrInst::StoreGlobal { global, src } => {
                // load r0, [bp - src_offset]
                // store [global_label], r0
                let src_offset = self.stack_frame.get(*src);
                let global_name = self.ir.globals.label_for(*global);
                self.emit_inst(&format!("load r0, {}", Self::addr(src_offset)))?;
                self.emit_inst(&format!("store [{}], r0", global_name))?;
            }
            IrInst::AddrOfGlobal { dst, global } => {
                // movi r0, global_label
                // store [bp - dst_offset], r0
                let dst_offset = self.stack_frame.get(*dst);
                let global_name = self.ir.globals.label_for(*global);
                self.emit_inst(&format!("movi r0, {}", global_name))?;
                self.emit_inst(&format!("store {}, r0", Self::addr(dst_offset)))?;
            }
            IrInst::AddrOfFunc { dst, func } => {
                // movi r0, func_label
                // store [bp - dst_offset], r0
                let dst_offset = self.stack_frame.get(*dst);
                let func_name = self.ir.functions.label_for(*func);
                self.emit_inst(&format!("movi r0, {}", func_name))?;
                self.emit_inst(&format!("store {}, r0", Self::addr(dst_offset)))?;
            }
            IrInst::Jump { target } => {
                // jmp target_label
                self.emit_inst(&format!("jmp {}L{}", func_label, target))?;
            }
            IrInst::JumpIfZero { cond, target } => {
                // load r0, [bp - cond_offset]
                // cmpi r0, 0
                // je target_label
                let cond_offset = self.stack_frame.get(*cond);
                self.emit_inst(&format!("load r0, {}", Self::addr(cond_offset)))?;
                self.emit_inst("cmpi r0, 0")?;
                self.emit_inst(&format!("je {}L{}", func_label, target))?;
            }
            IrInst::Call { dst, func, args } => {
                // load r0, [bp - func_offset]
                // callr r0
                // if dst.is_some() { store return value }
                let func_offset = self.stack_frame.get(*func);
                for arg in args.iter().rev() {
                    let arg_offset = self.stack_frame.get(*arg);
                    self.emit_inst(&format!("load r0, {}", Self::addr(arg_offset)))?;
                    self.emit_inst(&format!("push r0"))?;
                }
                self.emit_inst(&format!("load r0, {}", Self::addr(func_offset)))?;
                self.emit_inst("callr r0")?;
                if let Some(dst) = dst {
                    let dst_offset = self.stack_frame.get(*dst);
                    self.emit_inst(&format!("store {}, r0", Self::addr(dst_offset)))?;
                }

                if !args.is_empty() {
                    self.emit_inst(&format!("addi sp, sp, {}", args.len() * 4))?;
                }
            }
            IrInst::Return { value } => {
                if let Some(value) = value {
                    let value_offset = self.stack_frame.get(*value);
                    self.emit_inst(&format!("load r0, {}", Self::addr(value_offset)))?;
                } else {
                    // return 0 by default
                    self.emit_inst("movi r0, 0")?;
                }
                if let Some(exit_label) = exit_label {
                    self.emit_inst(&format!("jmp {}", exit_label))?;
                } else {
                    // If no exit label, just emit halt
                    self.emit_inst("halt")?;
                }
            }
            IrInst::Label(label) => {
                self.emit_label_id(func_label, *label)?;
            }
        }

        Ok(())
    }
}
