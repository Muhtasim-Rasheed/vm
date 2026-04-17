use std::collections::HashMap;

use isa::Instruction;

use crate::parser::{ASTInstruction, Directive, Expr};

type SymbolTable<'a> = HashMap<&'a str, u32>;

#[derive(Debug)]
pub enum LoweringError<'a> {
    UndefinedLabel(&'a str),
    ImmediateOutOfRange {
        label: &'a str,
        base: u32,
        offset: i32,
    },
}

pub fn resolve_expr<'a>(
    expr: Expr<'a>,
    symbols: &SymbolTable<'a>,
) -> Result<u32, LoweringError<'a>> {
    match expr {
        Expr::Immediate(value) => Ok(value),
        Expr::Label(name) => symbols
            .get(name)
            .copied()
            .ok_or(LoweringError::UndefinedLabel(name)),
        Expr::LabelOffset(name, offset) => {
            let base = symbols
                .get(name)
                .copied()
                .ok_or(LoweringError::UndefinedLabel(name))?;

            let value = (base as i64) + (offset as i64);

            if !(0..=u32::MAX as i64).contains(&value) {
                return Err(LoweringError::ImmediateOutOfRange {
                    label: name,
                    base,
                    offset,
                });
            }

            Ok(value as u32)
        }
        Expr::Negate(inner) => {
            let value = resolve_expr(*inner, symbols)?;
            Ok(value.wrapping_neg())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoweredInstruction<'a> {
    Instruction(Instruction),
    Directive(Directive<'a>),
}

pub fn build_symbol_table<'a>(ast: &[ASTInstruction<'a>]) -> SymbolTable<'a> {
    let mut symbols = HashMap::new();
    let mut pc = 0u32;

    for instr in ast {
        match instr {
            ASTInstruction::Directive(Directive::Label(name)) => {
                symbols.insert(*name, pc);
            }
            ASTInstruction::Directive(Directive::Data(bytes)) => {
                // Each byte in the data directive takes up 1 byte in memory
                pc += bytes
                    .iter()
                    .map(|item| match item {
                        crate::parser::DataItem::Byte(_) => 1,
                        crate::parser::DataItem::Bytes(bytes) => bytes.len() as u32,
                    })
                    .sum::<u32>();
            }
            ASTInstruction::Directive(Directive::Word(exprs)) => {
                // Each word takes up 4 bytes in memory
                pc += exprs.len() as u32 * 4;
            }
            ASTInstruction::E { .. }
            | ASTInstruction::R { .. }
            | ASTInstruction::RR { .. }
            | ASTInstruction::RI { .. }
            | ASTInstruction::RRR { .. }
            | ASTInstruction::RRI { .. }
            | ASTInstruction::M { .. }
            | ASTInstruction::J { .. } => {
                // Count instructions for PC increment
                // Since all instructions are 8 bytes, we can simply increment by 8 for each
                // instruction
                pc += 8;
            }
        }
    }

    symbols
}

fn lower_instruction<'a>(
    ast_instruction: ASTInstruction<'a>,
    symbols: &SymbolTable<'a>,
) -> Result<Option<LoweredInstruction<'a>>, LoweringError<'a>> {
    if let ASTInstruction::Directive(directive) = ast_instruction {
        if !matches!(directive, Directive::Label(_)) {
            return Ok(Some(LoweredInstruction::Directive(directive)));
        } else {
            return Ok(None);
        }
    }

    Ok(Some(LoweredInstruction::Instruction(
        match ast_instruction {
            ASTInstruction::E { opcode } => Instruction::E { opcode },
            ASTInstruction::R { opcode, reg1 } => Instruction::R { opcode, reg1 },
            ASTInstruction::RR { opcode, reg1, reg2 } => Instruction::RR { opcode, reg1, reg2 },
            ASTInstruction::RI { opcode, reg1, imm } => {
                let imm = resolve_expr(imm, symbols)?;
                Instruction::RI { opcode, reg1, imm }
            }
            ASTInstruction::RRR {
                opcode,
                reg1,
                reg2,
                reg3,
            } => Instruction::RRR {
                opcode,
                reg1,
                reg2,
                reg3,
            },
            ASTInstruction::RRI {
                opcode,
                reg1,
                reg2,
                imm,
            } => {
                let imm = resolve_expr(imm, symbols)?;
                Instruction::RRI {
                    opcode,
                    reg1,
                    reg2,
                    imm,
                }
            }
            ASTInstruction::M {
                opcode,
                mode,
                reg1,
                reg2,
                imm,
            } => {
                let imm = resolve_expr(imm, symbols)?;
                Instruction::M {
                    opcode,
                    mode,
                    reg1,
                    reg2,
                    imm,
                }
            }
            ASTInstruction::J { opcode, imm } => {
                let imm = resolve_expr(imm, symbols)?;
                Instruction::J { opcode, imm }
            }
            _ => unreachable!(),
        },
    )))
}

pub fn lower_instructions<'a>(
    ast: &[ASTInstruction<'a>],
    symbols: &SymbolTable<'a>,
) -> Result<Vec<LoweredInstruction<'a>>, LoweringError<'a>> {
    ast.iter()
        .filter_map(|instr| lower_instruction(instr.clone(), symbols).transpose())
        .collect()
}
