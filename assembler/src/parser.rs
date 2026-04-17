use isa::{MemoryMode, Opcode, Register};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::{
        complete::{escaped_transform, tag, tag_no_case, take_while},
        is_not, take_until,
    },
    character::complete::{multispace1, satisfy},
    combinator::{map, not, peek, recognize, value},
    sequence::{delimited, pair, preceded, terminated},
};

fn ws<'a, F, O>(inner: F) -> impl Parser<&'a str, Output = O, Error = nom::error::Error<&'a str>>
where
    F: Parser<&'a str, Output = O, Error = nom::error::Error<&'a str>>,
{
    delimited(
        nom::multi::many0(alt((multispace1, parse_comment))),
        inner,
        nom::multi::many0(alt((multispace1, parse_comment))),
    )
}

fn parse_comment(input: &str) -> IResult<&str, &str> {
    preceded(tag(";"), take_until("\n")).parse(input)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ASTInstruction<'a> {
    E {
        opcode: Opcode,
    },
    R {
        opcode: Opcode,
        reg1: Register,
    },
    RR {
        opcode: Opcode,
        reg1: Register,
        reg2: Register,
    },
    RI {
        opcode: Opcode,
        reg1: Register,
        imm: Expr<'a>,
    },
    RRR {
        opcode: Opcode,
        reg1: Register,
        reg2: Register,
        reg3: Register,
    },
    RRI {
        opcode: Opcode,
        reg1: Register,
        reg2: Register,
        imm: Expr<'a>,
    },
    M {
        opcode: Opcode,
        mode: MemoryMode,
        reg1: Register,
        reg2: Register,
        imm: Expr<'a>,
    },
    J {
        opcode: Opcode,
        imm: Expr<'a>,
    },
    Directive(Directive<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr<'a> {
    Immediate(u32),
    Label(&'a str),
    LabelOffset(&'a str, i32),
    Negate(Box<Expr<'a>>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataItem {
    Byte(u8),
    Bytes(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Directive<'a> {
    // Unlike in other assembly languages, defining a label also uses a directive.
    Label(&'a str),
    Data(Vec<DataItem>),
    Word(Vec<Expr<'a>>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MemOperand<'a> {
    Direct(Expr<'a>),
    Indirect(Register),
    Indexed(Register, Expr<'a>),
}

fn parse_register(input: &str) -> IResult<&str, Register> {
    terminated(
        alt((
            map(tag_no_case("r0"), |_| Register::R0),
            map(tag_no_case("r1"), |_| Register::R1),
            map(tag_no_case("r2"), |_| Register::R2),
            map(tag_no_case("r3"), |_| Register::R3),
            map(tag_no_case("r4"), |_| Register::R4),
            map(tag_no_case("r5"), |_| Register::R5),
            map(tag_no_case("r6"), |_| Register::R6),
            map(tag_no_case("r7"), |_| Register::R7),
            map(tag_no_case("r8"), |_| Register::R8),
            map(tag_no_case("r9"), |_| Register::R9),
            map(tag_no_case("ra"), |_| Register::RA),
            map(tag_no_case("rb"), |_| Register::RB),
            map(tag_no_case("rc"), |_| Register::RC),
            map(tag_no_case("rd"), |_| Register::RD),
            map(tag_no_case("sp"), |_| Register::SP),
            map(tag_no_case("bp"), |_| Register::BP),
        )),
        not(peek(satisfy(|c: char| {
            c.is_ascii_alphanumeric() || c == '_'
        }))),
    )
    .parse(input)
}

fn parse_opcode(input: &str) -> IResult<&str, Opcode> {
    terminated(
        alt((
            alt((
                map(tag_no_case("storeb"), |_| Opcode::STOREB),
                map(tag_no_case("store"), |_| Opcode::STORE),
                map(tag_no_case("loadb"), |_| Opcode::LOADB),
                map(tag_no_case("callr"), |_| Opcode::CALLR),
                map(tag_no_case("setne"), |_| Opcode::SETNE),
                map(tag_no_case("setge"), |_| Opcode::SETGE),
                map(tag_no_case("setle"), |_| Opcode::SETLE),
                map(tag_no_case("halt"), |_| Opcode::HALT),
                map(tag_no_case("push"), |_| Opcode::PUSH),
                map(tag_no_case("call"), |_| Opcode::CALL),
                map(tag_no_case("cmpi"), |_| Opcode::CMPI),
                map(tag_no_case("xori"), |_| Opcode::XORI),
                map(tag_no_case("andi"), |_| Opcode::ANDI),
                map(tag_no_case("noti"), |_| Opcode::NOTI),
                map(tag_no_case("modi"), |_| Opcode::MODI),
                map(tag_no_case("divi"), |_| Opcode::DIVI),
                map(tag_no_case("muli"), |_| Opcode::MULI),
                map(tag_no_case("subi"), |_| Opcode::SUBI),
            )),
            alt((
                map(tag_no_case("addi"), |_| Opcode::ADDI),
                map(tag_no_case("load"), |_| Opcode::LOAD),
                map(tag_no_case("movi"), |_| Opcode::MOVI),
                map(tag_no_case("sete"), |_| Opcode::SETE),
                map(tag_no_case("setl"), |_| Opcode::SETL),
                map(tag_no_case("setg"), |_| Opcode::SETG),
                map(tag_no_case("nop"), |_| Opcode::NOP),
                map(tag_no_case("pop"), |_| Opcode::POP),
                map(tag_no_case("ret"), |_| Opcode::RET),
                map(tag_no_case("jne"), |_| Opcode::JNE),
                map(tag_no_case("jmp"), |_| Opcode::JMP),
                map(tag_no_case("cmp"), |_| Opcode::CMP),
                map(tag_no_case("ori"), |_| Opcode::ORI),
                map(tag_no_case("xor"), |_| Opcode::XOR),
            )),
            alt((
                map(tag_no_case("and"), |_| Opcode::AND),
                map(tag_no_case("not"), |_| Opcode::NOT),
                map(tag_no_case("mod"), |_| Opcode::MOD),
                map(tag_no_case("div"), |_| Opcode::DIV),
                map(tag_no_case("mul"), |_| Opcode::MUL),
                map(tag_no_case("sub"), |_| Opcode::SUB),
                map(tag_no_case("add"), |_| Opcode::ADD),
                map(tag_no_case("mov"), |_| Opcode::MOV),
                map(tag_no_case("lea"), |_| Opcode::LEA),
                map(tag_no_case("jl"), |_| Opcode::JL),
                map(tag_no_case("jg"), |_| Opcode::JG),
                map(tag_no_case("je"), |_| Opcode::JE),
                map(tag_no_case("or"), |_| Opcode::OR),
            )),
        )),
        not(peek(satisfy(|c: char| {
            c.is_ascii_alphanumeric() || c == '_'
        }))),
    )
    .parse(input)
}

fn parse_label_directive<'a>(input: &'a str) -> IResult<&'a str, Directive<'a>> {
    preceded(
        ws(tag(".")),
        preceded(tag("label"), ws(parse_label)).map(Directive::Label),
    )
    .parse(input)
}

fn parse_data_directive<'a>(input: &'a str) -> IResult<&'a str, Directive<'a>> {
    fn parse_item(input: &str) -> IResult<&str, DataItem> {
        alt((
            parse_string.map(|s| DataItem::Bytes(s.as_bytes().to_vec())),
            parse_number.map(|n| DataItem::Byte(n as u8)),
        ))
        .parse(input)
    }

    preceded(
        ws(tag(".")),
        preceded(
            tag("data"),
            ws(nom::multi::separated_list0(ws(tag(",")), parse_item)).map(Directive::Data),
        ),
    )
    .parse(input)
}

fn parse_word_directive<'a>(input: &'a str) -> IResult<&'a str, Directive<'a>> {
    preceded(
        ws(tag(".")),
        preceded(
            tag("word"),
            ws(nom::multi::separated_list0(ws(tag(",")), parse_expr)).map(Directive::Word),
        ),
    )
    .parse(input)
}

fn parse_directive<'a>(input: &'a str) -> IResult<&'a str, Directive<'a>> {
    alt((
        parse_label_directive,
        parse_data_directive,
        parse_word_directive,
    ))
    .parse(input)
}

fn parse_number(input: &str) -> IResult<&str, u32> {
    alt((
        map(
            recognize(pair(
                tag_no_case("0x"),
                take_while(|c: char| c.is_ascii_hexdigit()),
            )),
            |s: &str| u32::from_str_radix(&s[2..], 16).unwrap(),
        ),
        map(
            recognize(pair(
                tag_no_case("0b"),
                take_while(|c: char| c == '0' || c == '1'),
            )),
            |s: &str| u32::from_str_radix(&s[2..], 2).unwrap(),
        ),
        nom::character::complete::u32,
    ))
    .parse(input)
}

fn parse_string(input: &str) -> IResult<&str, String> {
    delimited(
        nom::character::complete::char('"'),
        escaped_transform(
            is_not("\\\""),
            '\\',
            alt((
                value("\\", tag("\\")),
                value("\"", tag("\"")),
                value("\n", tag("n")),
                value("\r", tag("r")),
                value("\t", tag("t")),
                value("\0", tag("0")),
            )),
        ),
        nom::character::complete::char('"'),
    )
    .parse(input)
}

fn parse_label(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        satisfy(|c: char| c.is_ascii_alphabetic() || c == '_'),
        take_while(|c: char| c.is_ascii_alphanumeric() || c == '_'),
    ))
    .parse(input)
}

fn parse_expr<'a>(input: &'a str) -> IResult<&'a str, Expr<'a>> {
    alt((
        map(
            (ws(parse_label), alt((tag("+"), tag("-"))), ws(parse_number)),
            |(label, op, offset)| {
                let offset = if op == "+" {
                    offset as i32
                } else {
                    -(offset as i32)
                };
                Expr::LabelOffset(label, offset)
            },
        ),
        map(parse_number, Expr::Immediate),
        map(parse_label, Expr::Label),
    ))
    .parse(input)
}

fn parse_memory_operand<'a>(input: &'a str) -> IResult<&'a str, MemOperand<'a>> {
    delimited(
        ws(tag("[")),
        ws(alt((
            map(
                (parse_register, ws(tag("+")), parse_expr),
                |(base, _, offset)| MemOperand::Indexed(base, offset),
            ),
            map(
                (parse_register, ws(tag("-")), parse_expr),
                |(base, _, offset)| MemOperand::Indexed(base, Expr::Negate(Box::new(offset)))
            ),
            map(parse_register, MemOperand::Indirect),
            map(parse_expr, MemOperand::Direct),
        ))),
        ws(tag("]")),
    )
    .parse(input)
}

fn parse_instruction<'a>(input: &'a str) -> IResult<&'a str, ASTInstruction<'a>> {
    #[derive(Debug)]
    enum Operand<'a> {
        Register(Register),
        Immediate(Expr<'a>),
        Memory(MemOperand<'a>),
    }

    impl<'a> From<Register> for Operand<'a> {
        fn from(value: Register) -> Self {
            Operand::Register(value)
        }
    }

    impl<'a> From<Expr<'a>> for Operand<'a> {
        fn from(value: Expr<'a>) -> Self {
            Operand::Immediate(value)
        }
    }

    impl<'a> From<MemOperand<'a>> for Operand<'a> {
        fn from(value: MemOperand<'a>) -> Self {
            Operand::Memory(value)
        }
    }

    if let Ok((rest, directive)) = parse_directive(input) {
        return Ok((rest, ASTInstruction::Directive(directive)));
    }

    let (input, opcode) = ws(parse_opcode).parse(input)?;
    let (input, operands) = ws(nom::multi::separated_list0(
        ws(tag(",")),
        alt((
            map(parse_memory_operand, Operand::from),
            map(parse_register, Operand::from),
            map(parse_expr, Operand::from),
        )),
    ))
    .parse(input)?;

    let instruction = match opcode.format() {
        isa::InstructionFormat::E => {
            // opcode
            ASTInstruction::E { opcode }
        }
        isa::InstructionFormat::R => {
            // opcode reg1
            let reg1 = match operands.get(0) {
                Some(Operand::Register(r)) => *r,
                _ => {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Digit,
                    )));
                }
            };
            ASTInstruction::R { opcode, reg1 }
        }
        isa::InstructionFormat::RR => {
            // opcode reg1, reg2
            let reg1 = match operands.get(0) {
                Some(Operand::Register(r)) => *r,
                _ => {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Digit,
                    )));
                }
            };
            let reg2 = match operands.get(1) {
                Some(Operand::Register(r)) => *r,
                _ => {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Digit,
                    )));
                }
            };
            ASTInstruction::RR { opcode, reg1, reg2 }
        }
        isa::InstructionFormat::RI => {
            // opcode reg1, imm
            let reg1 = match operands.get(0) {
                Some(Operand::Register(r)) => *r,
                _ => {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Digit,
                    )));
                }
            };
            let imm = match operands.get(1) {
                Some(Operand::Immediate(i)) => i.clone(),
                _ => {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Digit,
                    )));
                }
            };
            ASTInstruction::RI { opcode, reg1, imm }
        }
        isa::InstructionFormat::RRR => {
            // opcode reg1, reg2, reg3
            let reg1 = match operands.get(0) {
                Some(Operand::Register(r)) => *r,
                _ => {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Digit,
                    )));
                }
            };
            let reg2 = match operands.get(1) {
                Some(Operand::Register(r)) => *r,
                _ => {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Digit,
                    )));
                }
            };
            let reg3 = match operands.get(2) {
                Some(Operand::Register(r)) => *r,
                _ => {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Digit,
                    )));
                }
            };
            ASTInstruction::RRR {
                opcode,
                reg1,
                reg2,
                reg3,
            }
        }
        isa::InstructionFormat::RRI => {
            // opcode reg1, reg2, imm
            let reg1 = match operands.get(0) {
                Some(Operand::Register(r)) => *r,
                _ => {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Digit,
                    )));
                }
            };
            let reg2 = match operands.get(1) {
                Some(Operand::Register(r)) => *r,
                _ => {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Digit,
                    )));
                }
            };
            let imm = match operands.get(2) {
                Some(Operand::Immediate(i)) => i.clone(),
                _ => {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Digit,
                    )));
                }
            };
            ASTInstruction::RRI {
                opcode,
                reg1,
                reg2,
                imm,
            }
        }
        isa::InstructionFormat::M => {
            match opcode {
                Opcode::LOAD | Opcode::LOADB => {
                    let reg1 = match operands.get(0) {
                        Some(Operand::Register(r)) => *r,
                        _ => {
                            return Err(nom::Err::Error(nom::error::Error::new(
                                input,
                                nom::error::ErrorKind::Digit,
                            )));
                        }
                    };

                    let mem = match operands.get(1) {
                        Some(Operand::Memory(m)) => m.clone(),
                        _ => {
                            return Err(nom::Err::Error(nom::error::Error::new(
                                input,
                                nom::error::ErrorKind::Digit,
                            )));
                        }
                    };

                    match mem {
                        MemOperand::Direct(expr) => ASTInstruction::M {
                            opcode,
                            mode: MemoryMode::Direct,
                            reg1,
                            reg2: Register::R0, // unused
                            imm: expr,
                        },
                        MemOperand::Indirect(base) => ASTInstruction::M {
                            opcode,
                            mode: MemoryMode::Indirect,
                            reg1,
                            reg2: base,
                            imm: Expr::Immediate(0),
                        },
                        MemOperand::Indexed(base, offset) => ASTInstruction::M {
                            opcode,
                            mode: MemoryMode::Indexed,
                            reg1,
                            reg2: base,
                            imm: offset,
                        },
                    }
                }

                Opcode::STORE | Opcode::STOREB => {
                    let mem = match operands.get(0) {
                        Some(Operand::Memory(m)) => m.clone(),
                        _ => {
                            return Err(nom::Err::Error(nom::error::Error::new(
                                input,
                                nom::error::ErrorKind::Digit,
                            )));
                        }
                    };

                    let reg1 = match operands.get(1) {
                        Some(Operand::Register(r)) => *r,
                        _ => {
                            return Err(nom::Err::Error(nom::error::Error::new(
                                input,
                                nom::error::ErrorKind::Digit,
                            )));
                        }
                    };

                    match mem {
                        MemOperand::Direct(expr) => ASTInstruction::M {
                            opcode,
                            mode: MemoryMode::Direct,
                            reg1,
                            reg2: Register::R0, // unused
                            imm: expr,
                        },
                        MemOperand::Indirect(base) => ASTInstruction::M {
                            opcode,
                            mode: MemoryMode::Indirect,
                            reg1,
                            reg2: base,
                            imm: Expr::Immediate(0),
                        },
                        MemOperand::Indexed(base, offset) => ASTInstruction::M {
                            opcode,
                            mode: MemoryMode::Indexed,
                            reg1,
                            reg2: base,
                            imm: offset,
                        },
                    }
                }

                Opcode::LEA => {
                    let reg1 = match operands.get(0) {
                        Some(Operand::Register(r)) => *r,
                        _ => {
                            return Err(nom::Err::Error(nom::error::Error::new(
                                input,
                                nom::error::ErrorKind::Digit,
                            )));
                        }
                    };

                    let mem = match operands.get(1) {
                        Some(Operand::Memory(m)) => m.clone(),
                        _ => {
                            return Err(nom::Err::Error(nom::error::Error::new(
                                input,
                                nom::error::ErrorKind::Digit,
                            )));
                        }
                    };

                    match mem {
                        MemOperand::Direct(expr) => ASTInstruction::M {
                            opcode,
                            mode: MemoryMode::Direct,
                            reg1,
                            reg2: Register::R0, // unused
                            imm: expr,
                        },
                        MemOperand::Indirect(base) => ASTInstruction::M {
                            opcode,
                            mode: MemoryMode::Indirect,
                            reg1,
                            reg2: base,
                            imm: Expr::Immediate(0),
                        },
                        MemOperand::Indexed(base, offset) => ASTInstruction::M {
                            opcode,
                            mode: MemoryMode::Indexed,
                            reg1,
                            reg2: base,
                            imm: offset,
                        },
                    }
                }

                _ => unreachable!(),
            }
        }
        isa::InstructionFormat::J => {
            let imm = match operands.get(0) {
                Some(Operand::Immediate(i)) => i.clone(),
                _ => {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Digit,
                    )));
                }
            };

            ASTInstruction::J { opcode, imm }
        }
    };

    Ok((input, instruction))
}

pub fn parse_instructions<'a>(input: &'a str) -> IResult<&'a str, Vec<ASTInstruction<'a>>> {
    nom::multi::many0(ws(parse_instruction)).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_register() {
        assert_eq!(parse_register("r0"), Ok(("", Register::R0)));
        assert_eq!(parse_register("r5"), Ok(("", Register::R5)));
        assert_eq!(parse_register("sp"), Ok(("", Register::SP)));
        assert_eq!(parse_register("bp"), Ok(("", Register::BP)));
        assert!(parse_register("r10").is_err());
    }

    #[test]
    fn test_parse_opcode() {
        assert_eq!(parse_opcode("add"), Ok(("", Opcode::ADD)));
        assert_eq!(parse_opcode("SUB"), Ok(("", Opcode::SUB)));
        assert_eq!(parse_opcode("Load"), Ok(("", Opcode::LOAD)));
        assert_eq!(parse_opcode("movi"), Ok(("", Opcode::MOVI)));
        assert_eq!(parse_opcode("mov"), Ok(("", Opcode::MOV)));
        assert!(parse_opcode("invalid").is_err());
        assert!(parse_opcode("movx").is_err());
    }

    #[test]
    fn test_parse_expr() {
        assert_eq!(parse_expr("123"), Ok(("", Expr::Immediate(123))));
        assert_eq!(parse_expr("label"), Ok(("", Expr::Label("label"))));
        assert_eq!(
            parse_expr("label+  4"),
            Ok(("", Expr::LabelOffset("label", 4)))
        );
        assert_eq!(
            parse_expr("label   -8"),
            Ok(("", Expr::LabelOffset("label", -8)))
        );
    }

    #[test]
    fn test_parse_memory_operand() {
        assert_eq!(
            parse_memory_operand("[0x001F8004]"),
            Ok(("", MemOperand::Direct(Expr::Immediate(0x001F8004))))
        );
        assert_eq!(
            parse_memory_operand("[r1    ]"),
            Ok(("", MemOperand::Indirect(Register::R1)))
        );
        assert_eq!(
            parse_memory_operand("[   r2 +   4]"),
            Ok(("", MemOperand::Indexed(Register::R2, Expr::Immediate(4))))
        );
        assert_eq!(
            parse_memory_operand("[ label    - 8]"),
            Ok(("", MemOperand::Direct(Expr::LabelOffset("label", -8))))
        );
        assert_eq!(
            parse_memory_operand("[ label]"),
            Ok(("", MemOperand::Direct(Expr::Label("label"))))
        );
    }

    #[test]
    fn test_parse_instruction_e() {
        assert_eq!(
            parse_instruction("halt"),
            Ok((
                "",
                ASTInstruction::E {
                    opcode: Opcode::HALT
                }
            ))
        );
    }

    #[test]
    fn test_parse_instruction_rr() {
        assert_eq!(
            parse_instruction("mov r1, r2"),
            Ok((
                "",
                ASTInstruction::RR {
                    opcode: Opcode::MOV,
                    reg1: Register::R1,
                    reg2: Register::R2,
                }
            ))
        );
    }

    #[test]
    fn test_parse_instruction_ri() {
        assert_eq!(
            parse_instruction("movi r1, 123"),
            Ok((
                "",
                ASTInstruction::RI {
                    opcode: Opcode::MOVI,
                    reg1: Register::R1,
                    imm: Expr::Immediate(123),
                }
            ))
        );

        assert_eq!(
            parse_instruction("movi r2, label + 2"),
            Ok((
                "",
                ASTInstruction::RI {
                    opcode: Opcode::MOVI,
                    reg1: Register::R2,
                    imm: Expr::LabelOffset("label", 2),
                }
            ))
        );
    }

    #[test]
    fn test_parse_instruction_m() {
        assert_eq!(
            parse_instruction("load r1, [r2 + 4]"),
            Ok((
                "",
                ASTInstruction::M {
                    opcode: Opcode::LOAD,
                    mode: MemoryMode::Indexed,
                    reg1: Register::R1,
                    reg2: Register::R2,
                    imm: Expr::Immediate(4),
                }
            ))
        );

        assert_eq!(
            parse_instruction("store [label - 8], r3"),
            Ok((
                "",
                ASTInstruction::M {
                    opcode: Opcode::STORE,
                    mode: MemoryMode::Direct,
                    reg1: Register::R3,
                    reg2: Register::R0, // unused
                    imm: Expr::LabelOffset("label", -8),
                }
            ))
        );
    }

    #[test]
    fn test_parse_instruction_j() {
        assert_eq!(
            parse_instruction("jmp label"),
            Ok((
                "",
                ASTInstruction::J {
                    opcode: Opcode::JMP,
                    imm: Expr::Label("label"),
                }
            ))
        );
    }

    #[test]
    fn test_parse_directive() {
        assert_eq!(
            parse_instruction(".label start"),
            Ok(("", ASTInstruction::Directive(Directive::Label("start"))))
        );

        assert_eq!(
            parse_instruction(".data \"Hello\", 0"),
            Ok((
                "",
                ASTInstruction::Directive(Directive::Data(vec![
                    DataItem::Bytes(b"Hello".to_vec()),
                    DataItem::Byte(0),
                ]))
            ))
        );
    }
}
