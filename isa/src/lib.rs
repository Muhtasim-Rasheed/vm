#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    RA,
    RB,
    RC,
    RD,
    SP,
    BP,
}

impl TryFrom<u8> for Register {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Register::R0),
            0x01 => Ok(Register::R1),
            0x02 => Ok(Register::R2),
            0x03 => Ok(Register::R3),
            0x04 => Ok(Register::R4),
            0x05 => Ok(Register::R5),
            0x06 => Ok(Register::R6),
            0x07 => Ok(Register::R7),
            0x08 => Ok(Register::R8),
            0x09 => Ok(Register::R9),
            0x0A => Ok(Register::RA),
            0x0B => Ok(Register::RB),
            0x0C => Ok(Register::RC),
            0x0D => Ok(Register::RD),
            0x0E => Ok(Register::SP),
            0x0F => Ok(Register::BP),
            _ => Err(()),
        }
    }
}

impl From<Register> for u8 {
    fn from(reg: Register) -> Self {
        match reg {
            Register::R0 => 0x00,
            Register::R1 => 0x01,
            Register::R2 => 0x02,
            Register::R3 => 0x03,
            Register::R4 => 0x04,
            Register::R5 => 0x05,
            Register::R6 => 0x06,
            Register::R7 => 0x07,
            Register::R8 => 0x08,
            Register::R9 => 0x09,
            Register::RA => 0x0A,
            Register::RB => 0x0B,
            Register::RC => 0x0C,
            Register::RD => 0x0D,
            Register::SP => 0x0E,
            Register::BP => 0x0F,
        }
    }
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let reg_str = match self {
            Register::R0 => "r0",
            Register::R1 => "r1",
            Register::R2 => "r2",
            Register::R3 => "r3",
            Register::R4 => "r4",
            Register::R5 => "r5",
            Register::R6 => "r6",
            Register::R7 => "r7",
            Register::R8 => "r8",
            Register::R9 => "r9",
            Register::RA => "ra",
            Register::RB => "rb",
            Register::RC => "rc",
            Register::RD => "rd",
            Register::SP => "sp",
            Register::BP => "bp",
        };
        write!(f, "{}", reg_str)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryMode {
    Direct,
    Indirect,
    Indexed,
}

impl TryFrom<u8> for MemoryMode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(MemoryMode::Direct),
            0x01 => Ok(MemoryMode::Indirect),
            0x02 => Ok(MemoryMode::Indexed),
            _ => Err(()),
        }
    }
}

impl From<MemoryMode> for u8 {
    fn from(mode: MemoryMode) -> Self {
        match mode {
            MemoryMode::Direct => 0x00,
            MemoryMode::Indirect => 0x01,
            MemoryMode::Indexed => 0x02,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionFormat {
    E,
    R,
    RR,
    RI,
    RRR,
    RRI,
    M,
    J,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    MOV,
    MOVI,
    LOAD,
    STORE,

    ADD,
    SUB,
    MUL,
    DIV,
    MOD,
    NOT,
    AND,
    OR,
    XOR,
    ADDI,
    SUBI,
    MULI,
    DIVI,
    MODI,
    NOTI,
    ANDI,
    ORI,
    XORI,

    CMP,
    CMPI,
    JMP,
    JE,
    JNE,
    JG,
    JL,
    CALL,
    RET,

    PUSH,
    POP,

    NOP,
    HALT,
}

impl Opcode {
    pub fn format(&self) -> InstructionFormat {
        use Opcode::*;

        match self {
            RET | NOP | HALT => InstructionFormat::E,
            NOT | PUSH | POP => InstructionFormat::R,
            MOV | CMP => InstructionFormat::RR,
            MOVI | NOTI | CMPI => InstructionFormat::RI,
            ADD | SUB | MUL | DIV | MOD | AND | OR | XOR => InstructionFormat::RRR,
            ADDI | SUBI | MULI | DIVI | MODI | ANDI | ORI | XORI => InstructionFormat::RRI,
            LOAD | STORE => InstructionFormat::M,
            JMP | JE | JNE | JG | JL | CALL => InstructionFormat::J,
        }
    }
}

impl TryFrom<u8> for Opcode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use Opcode::*;

        match value {
            0x00 => Ok(MOV),
            0x01 => Ok(MOVI),
            0x02 => Ok(LOAD),
            0x03 => Ok(STORE),

            0x10 => Ok(ADD),
            0x11 => Ok(SUB),
            0x12 => Ok(MUL),
            0x13 => Ok(DIV),
            0x14 => Ok(MOD),
            0x15 => Ok(NOT),
            0x16 => Ok(AND),
            0x17 => Ok(OR),
            0x18 => Ok(XOR),

            0x20 => Ok(ADDI),
            0x21 => Ok(SUBI),
            0x22 => Ok(MULI),
            0x23 => Ok(DIVI),
            0x24 => Ok(MODI),
            0x25 => Ok(NOTI),
            0x26 => Ok(ANDI),
            0x27 => Ok(ORI),
            0x28 => Ok(XORI),

            0x30 => Ok(CMP),
            0x31 => Ok(CMPI),

            0x40 => Ok(JMP),
            0x41 => Ok(JE),
            0x42 => Ok(JNE),
            0x43 => Ok(JG),
            0x44 => Ok(JL),
            0x45 => Ok(CALL),
            0x46 => Ok(RET),

            0x50 => Ok(PUSH),
            0x51 => Ok(POP),

            0xFE => Ok(NOP),
            0xFF => Ok(HALT),
            _ => Err(()),
        }
    }
}

impl From<Opcode> for u8 {
    fn from(opcode: Opcode) -> Self {
        use Opcode::*;

        match opcode {
            MOV => 0x00,
            MOVI => 0x01,
            LOAD => 0x02,
            STORE => 0x03,

            ADD => 0x10,
            SUB => 0x11,
            MUL => 0x12,
            DIV => 0x13,
            MOD => 0x14,
            NOT => 0x15,
            AND => 0x16,
            OR => 0x17,
            XOR => 0x18,

            ADDI => 0x20,
            SUBI => 0x21,
            MULI => 0x22,
            DIVI => 0x23,
            MODI => 0x24,
            NOTI => 0x25,
            ANDI => 0x26,
            ORI => 0x27,
            XORI => 0x28,

            CMP => 0x30,
            CMPI => 0x31,

            JMP => 0x40,
            JE => 0x41,
            JNE => 0x42,
            JG => 0x43,
            JL => 0x44,
            CALL => 0x45,
            RET => 0x46,

            PUSH => 0x50,
            POP => 0x51,

            NOP => 0xFE,
            HALT => 0xFF,
        }
    }
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
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
        imm: u32,
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
        imm: u32,
    },
    M {
        opcode: Opcode,
        mode: MemoryMode,
        reg1: Register,
        reg2: Register,
        imm: u32,
    },
    J {
        opcode: Opcode,
        imm: u32,
    },
}

impl Instruction {
    pub fn opcode(&self) -> Opcode {
        match self {
            Instruction::E { opcode } => *opcode,
            Instruction::R { opcode, .. } => *opcode,
            Instruction::RR { opcode, .. } => *opcode,
            Instruction::RI { opcode, .. } => *opcode,
            Instruction::RRR { opcode, .. } => *opcode,
            Instruction::RRI { opcode, .. } => *opcode,
            Instruction::M { opcode, .. } => *opcode,
            Instruction::J { opcode, .. } => *opcode,
        }
    }

    pub fn format(&self) -> InstructionFormat {
        self.opcode().format()
    }
}

impl TryFrom<u64> for Instruction {
    type Error = ();

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        let opcode = Opcode::try_from((value & 0xFF) as u8)?;
        let format = opcode.format();

        match format {
            InstructionFormat::E => Ok(Instruction::E { opcode }),
            InstructionFormat::R => {
                let reg1 = Register::try_from(((value >> 8) & 0xFF) as u8)?;
                Ok(Instruction::R { opcode, reg1 })
            }
            InstructionFormat::RR => {
                let reg1 = Register::try_from(((value >> 8) & 0xFF) as u8)?;
                let reg2 = Register::try_from(((value >> 16) & 0xFF) as u8)?;
                Ok(Instruction::RR { opcode, reg1, reg2 })
            }
            InstructionFormat::RI => {
                let reg1 = Register::try_from(((value >> 8) & 0xFF) as u8)?;
                let imm = (value >> 16) as u32;
                Ok(Instruction::RI { opcode, reg1, imm })
            }
            InstructionFormat::RRR => {
                let reg1 = Register::try_from(((value >> 8) & 0xFF) as u8)?;
                let reg2 = Register::try_from(((value >> 16) & 0xFF) as u8)?;
                let reg3 = Register::try_from(((value >> 24) & 0xFF) as u8)?;
                Ok(Instruction::RRR {
                    opcode,
                    reg1,
                    reg2,
                    reg3,
                })
            }
            InstructionFormat::RRI => {
                let reg1 = Register::try_from(((value >> 8) & 0xFF) as u8)?;
                let reg2 = Register::try_from(((value >> 16) & 0xFF) as u8)?;
                let imm = (value >> 24) as u32;
                Ok(Instruction::RRI {
                    opcode,
                    reg1,
                    reg2,
                    imm,
                })
            }
            InstructionFormat::M => {
                let mode = MemoryMode::try_from(((value >> 8) & 0xFF) as u8)?;
                let reg1 = Register::try_from(((value >> 16) & 0xFF) as u8)?;
                let reg2 = Register::try_from(((value >> 24) & 0xFF) as u8)?;
                let imm = (value >> 32) as u32;
                Ok(Instruction::M {
                    opcode,
                    mode,
                    reg1,
                    reg2,
                    imm,
                })
            }
            InstructionFormat::J => {
                let imm = (value >> 8) as u32;
                Ok(Instruction::J { opcode, imm })
            }
        }
    }
}

impl From<Instruction> for u64 {
    fn from(instr: Instruction) -> Self {
        match instr {
            Instruction::E { opcode } => {
                let opcode_byte: u8 = opcode.into();
                opcode_byte as u64
            }
            Instruction::R { opcode, reg1 } => {
                let opcode_byte: u8 = opcode.into();
                let reg1_byte: u8 = reg1.into();
                (reg1_byte as u64) << 8 | (opcode_byte as u64)
            }
            Instruction::RR { opcode, reg1, reg2 } => {
                let opcode_byte: u8 = opcode.into();
                let reg1_byte: u8 = reg1.into();
                let reg2_byte: u8 = reg2.into();
                (reg2_byte as u64) << 16 | (reg1_byte as u64) << 8 | (opcode_byte as u64)
            }
            Instruction::RI { opcode, reg1, imm } => {
                let opcode_byte: u8 = opcode.into();
                let reg1_byte: u8 = reg1.into();
                (imm as u64) << 16 | (reg1_byte as u64) << 8 | (opcode_byte as u64)
            }
            Instruction::RRR {
                opcode,
                reg1,
                reg2,
                reg3,
            } => {
                let opcode_byte: u8 = opcode.into();
                let reg1_byte: u8 = reg1.into();
                let reg2_byte: u8 = reg2.into();
                let reg3_byte: u8 = reg3.into();
                (reg3_byte as u64) << 24
                    | (reg2_byte as u64) << 16
                    | (reg1_byte as u64) << 8
                    | (opcode_byte as u64)
            }
            Instruction::RRI {
                opcode,
                reg1,
                reg2,
                imm,
            } => {
                let opcode_byte: u8 = opcode.into();
                let reg1_byte: u8 = reg1.into();
                let reg2_byte: u8 = reg2.into();
                (imm as u64) << 24
                    | (reg2_byte as u64) << 16
                    | (reg1_byte as u64) << 8
                    | (opcode_byte as u64)
            }
            Instruction::M {
                opcode,
                mode,
                reg1,
                reg2,
                imm,
            } => {
                let opcode_byte: u8 = opcode.into();
                let mode_byte: u8 = mode.into();
                let reg1_byte: u8 = reg1.into();
                let reg2_byte: u8 = reg2.into();
                (imm as u64) << 32
                    | (reg2_byte as u64) << 24
                    | (reg1_byte as u64) << 16
                    | (mode_byte as u64) << 8
                    | (opcode_byte as u64)
            }
            Instruction::J { opcode, imm } => {
                let opcode_byte: u8 = opcode.into();
                (imm as u64) << 8 | (opcode_byte as u64)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_encoding() {
        // addi r1, r2, 0x12345678
        let instr = Instruction::RRI {
            opcode: Opcode::ADDI,
            reg1: Register::R1,
            reg2: Register::R2,
            imm: 0x12345678,
        };

        let encoded: u64 = instr.into();
        let decoded = Instruction::try_from(encoded).expect("Failed to decode instruction");

        assert_eq!(instr, decoded);
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::E { opcode } => write!(f, "{}", opcode),
            Instruction::R { opcode, reg1 } => write!(f, "{} {}", opcode, reg1),
            Instruction::RR { opcode, reg1, reg2 } => {
                write!(f, "{} {}, {}", opcode, reg1, reg2)
            }
            Instruction::RI { opcode, reg1, imm } => {
                write!(f, "{} {}, 0x{:X}", opcode, reg1, imm)
            }
            Instruction::RRR {
                opcode,
                reg1,
                reg2,
                reg3,
            } => write!(f, "{} {}, {}, {}", opcode, reg1, reg2, reg3),
            Instruction::RRI {
                opcode,
                reg1,
                reg2,
                imm,
            } => write!(f, "{} {}, {}, 0x{:X}", opcode, reg1, reg2, imm),
            Instruction::M {
                opcode,
                mode,
                reg1,
                reg2,
                imm,
            } => match opcode {
                Opcode::LOAD => match mode {
                    MemoryMode::Direct => write!(f, "{} {}, [0x{:X}]", opcode, reg1, imm),
                    MemoryMode::Indirect => write!(f, "{} {}, [{}]", opcode, reg1, reg2),
                    MemoryMode::Indexed => {
                        write!(f, "{} {}, [{} + 0x{:X}]", opcode, reg1, reg2, imm)
                    }
                },
                Opcode::STORE => match mode {
                    MemoryMode::Direct => write!(f, "{} [0x{:X}], {}", opcode, imm, reg1),
                    MemoryMode::Indirect => write!(f, "{} [{}], {}", opcode, reg2, reg1),
                    MemoryMode::Indexed => {
                        write!(f, "{} [{} + 0x{:X}], {}", opcode, reg2, imm, reg1)
                    }
                },
                _ => unreachable!(),
            },
            Instruction::J { opcode, imm } => write!(f, "{} 0x{:X}", opcode, imm),
        }
    }
}
