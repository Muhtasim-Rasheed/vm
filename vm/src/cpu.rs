use isa::{Instruction, MemoryMode, Opcode, Register};
use macroquad::prelude::*;

use crate::mem::{Memory, TEXT_GRID_SIZE, TEXT_GRID_WIDTH};

const FLAG_ZERO: u32 = 1 << 0;
const FLAG_NEG: u32 = 1 << 1;
const FLAG_CARRY: u32 = 1 << 2;
const CPU_HZ: u32 = 1_000_000;

pub struct Cpu {
    // Accessible by programmer
    pub registers: [u32; 14],
    pub sp: u32,
    pub bp: u32,

    // Not accessible by programmer
    pub flags: u32,
    pub pc: u32,

    leftover_cycles: f32,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            registers: [0; 14],
            sp: 0x00100000,
            bp: 0x00100000,
            flags: 0,
            pc: 0x00000000,
            leftover_cycles: 0.0,
        }
    }

    fn get(&self, reg: Register) -> u32 {
        match reg {
            Register::R0 => self.registers[0],
            Register::R1 => self.registers[1],
            Register::R2 => self.registers[2],
            Register::R3 => self.registers[3],
            Register::R4 => self.registers[4],
            Register::R5 => self.registers[5],
            Register::R6 => self.registers[6],
            Register::R7 => self.registers[7],
            Register::R8 => self.registers[8],
            Register::R9 => self.registers[9],
            Register::RA => self.registers[10],
            Register::RB => self.registers[11],
            Register::RC => self.registers[12],
            Register::RD => self.registers[13],
            Register::SP => self.sp,
            Register::BP => self.bp,
        }
    }

    fn set(&mut self, reg: Register, value: u32) {
        match reg {
            Register::R0 => self.registers[0] = value,
            Register::R1 => self.registers[1] = value,
            Register::R2 => self.registers[2] = value,
            Register::R3 => self.registers[3] = value,
            Register::R4 => self.registers[4] = value,
            Register::R5 => self.registers[5] = value,
            Register::R6 => self.registers[6] = value,
            Register::R7 => self.registers[7] = value,
            Register::R8 => self.registers[8] = value,
            Register::R9 => self.registers[9] = value,
            Register::RA => self.registers[10] = value,
            Register::RB => self.registers[11] = value,
            Register::RC => self.registers[12] = value,
            Register::RD => self.registers[13] = value,
            Register::SP => self.sp = value,
            Register::BP => self.bp = value,
        }
    }

    fn push(&mut self, value: u32, mem: &mut Memory) {
        self.sp -= 4;
        mem.write_u32(self.sp, value);
    }

    fn pop(&mut self, mem: &Memory) -> u32 {
        let value = mem.read_u32(self.sp);
        self.sp += 4;
        value
    }

    fn cmp(&mut self, a: u32, b: u32) {
        self.flags = 0;
        if a == b {
            self.flags |= FLAG_ZERO;
        }
        if (a as i32) < (b as i32) {
            self.flags |= FLAG_NEG;
        }
        if a < b {
            self.flags |= FLAG_CARRY;
        }
    }

    fn get_instruction(&self, mem: &Memory) -> Option<Instruction> {
        Instruction::try_from(mem.read_u64(self.pc)).ok()
    }

    fn execute_instruction(&mut self, instr: Instruction, mem: &mut Memory) -> bool {
        match instr {
            Instruction::E { opcode } => self.execute_e(opcode, mem),
            Instruction::R { opcode, reg1 } => self.execute_r(opcode, reg1, mem),
            Instruction::RR { opcode, reg1, reg2 } => self.execute_rr(opcode, reg1, reg2),
            Instruction::RI { opcode, reg1, imm } => self.execute_ri(opcode, reg1, imm),
            Instruction::RRR {
                opcode,
                reg1,
                reg2,
                reg3,
            } => self.execute_rrr(opcode, reg1, reg2, reg3),
            Instruction::RRI {
                opcode,
                reg1,
                reg2,
                imm,
            } => self.execute_rri(opcode, reg1, reg2, imm),
            Instruction::M {
                opcode,
                mode,
                reg1,
                reg2,
                imm,
            } => self.execute_m(opcode, mode, reg1, reg2, imm, mem),
            Instruction::J { opcode, imm } => self.execute_j(opcode, imm, mem),
        }
    }

    fn execute_e(&mut self, opcode: Opcode, mem: &Memory) -> bool {
        match opcode {
            Opcode::RET => {
                let ret_addr = mem.read_u32(self.bp);
                let old_bp = mem.read_u32(self.bp + 4);
                self.sp = self.bp + 8;
                self.bp = old_bp;
                self.pc = ret_addr;
                false
            }
            Opcode::NOP => false,
            Opcode::HALT => true,
            _ => unreachable!(),
        }
    }

    fn execute_r(&mut self, opcode: Opcode, reg1: Register, mem: &mut Memory) -> bool {
        match opcode {
            Opcode::NOT => {
                let value = self.get(reg1);
                self.set(reg1, !value);
                false
            }
            Opcode::PUSH => {
                let value = self.get(reg1);
                self.push(value, mem);
                false
            }
            Opcode::POP => {
                let value = self.pop(mem);
                self.set(reg1, value);
                false
            }
            Opcode::CALLR => {
                let target = self.get(reg1);
                self.push(self.bp, mem);
                self.push(self.pc, mem);
                self.bp = self.sp;
                self.pc = target;
                false
            }
            _ => unreachable!(),
        }
    }

    fn execute_rr(&mut self, opcode: Opcode, reg1: Register, reg2: Register) -> bool {
        match opcode {
            Opcode::MOV => {
                let value = self.get(reg2);
                self.set(reg1, value);
                false
            }
            Opcode::CMP => {
                let value1 = self.get(reg1);
                let value2 = self.get(reg2);
                self.cmp(value1, value2);
                false
            }
            _ => unreachable!(),
        }
    }

    fn execute_ri(&mut self, opcode: Opcode, reg1: Register, imm: u32) -> bool {
        match opcode {
            Opcode::MOVI => {
                self.set(reg1, imm);
                false
            }
            Opcode::NOTI => {
                self.set(reg1, !imm);
                false
            }
            Opcode::CMPI => {
                let value = self.get(reg1);
                self.cmp(value, imm);
                false
            }
            _ => unreachable!(),
        }
    }

    fn execute_rrr(
        &mut self,
        opcode: Opcode,
        reg1: Register,
        reg2: Register,
        reg3: Register,
    ) -> bool {
        let value1 = self.get(reg2);
        let value2 = self.get(reg3);
        match opcode {
            Opcode::ADD => {
                self.set(reg1, value1.wrapping_add(value2));
            }
            Opcode::SUB => {
                self.set(reg1, value1.wrapping_sub(value2));
            }
            Opcode::MUL => {
                self.set(reg1, value1.wrapping_mul(value2));
            }
            Opcode::DIV => {
                if value2 == 0 {
                    panic!("Division by zero");
                }
                self.set(reg1, value1.wrapping_div(value2));
            }
            Opcode::MOD => {
                if value2 == 0 {
                    panic!("Division by zero");
                }
                self.set(reg1, value1.wrapping_rem(value2));
            }
            Opcode::AND => {
                self.set(reg1, value1 & value2);
            }
            Opcode::OR => {
                self.set(reg1, value1 | value2);
            }
            Opcode::XOR => {
                self.set(reg1, value1 ^ value2);
            }
            _ => unreachable!(),
        }
        false
    }

    fn execute_rri(&mut self, opcode: Opcode, reg1: Register, reg2: Register, imm: u32) -> bool {
        let value = self.get(reg2);
        match opcode {
            Opcode::ADDI => {
                self.set(reg1, value.wrapping_add(imm));
            }
            Opcode::SUBI => {
                self.set(reg1, value.wrapping_sub(imm));
            }
            Opcode::MULI => {
                self.set(reg1, value.wrapping_mul(imm));
            }
            Opcode::DIVI => {
                if imm == 0 {
                    panic!("Division by zero");
                }
                self.set(reg1, value.wrapping_div(imm));
            }
            Opcode::MODI => {
                if imm == 0 {
                    panic!("Division by zero");
                }
                self.set(reg1, value.wrapping_rem(imm));
            }
            Opcode::ANDI => {
                self.set(reg1, value & imm);
            }
            Opcode::ORI => {
                self.set(reg1, value | imm);
            }
            Opcode::XORI => {
                self.set(reg1, value ^ imm);
            }
            _ => unreachable!(),
        }
        false
    }

    fn execute_m(
        &mut self,
        opcode: Opcode,
        mode: MemoryMode,
        reg1: Register,
        reg2: Register,
        imm: u32,
        mem: &mut Memory,
    ) -> bool {
        let addr = match mode {
            MemoryMode::Direct => imm,
            MemoryMode::Indirect => self.get(reg2),
            MemoryMode::Indexed => self.get(reg2).wrapping_add(imm),
        };
        match opcode {
            Opcode::LOAD => {
                let value = mem.read_u32(addr);
                self.set(reg1, value);
            }
            Opcode::STORE => {
                let value = self.get(reg1);
                mem.write_u32(addr, value);
            }
            Opcode::LOADB => {
                let value = mem.read_u8(addr) as u32;
                self.set(reg1, value);
            }
            Opcode::STOREB => {
                let value = self.get(reg1) as u8;
                mem.write_u8(addr, value);
            }
            _ => unreachable!(),
        }
        false
    }

    fn execute_j(&mut self, opcode: Opcode, imm: u32, mem: &mut Memory) -> bool {
        match opcode {
            Opcode::JMP => {
                self.pc = imm;
                false
            }
            Opcode::JE => {
                if self.flags & FLAG_ZERO != 0 {
                    self.pc = imm;
                }
                false
            }
            Opcode::JNE => {
                if self.flags & FLAG_ZERO == 0 {
                    self.pc = imm;
                }
                false
            }
            Opcode::JG => {
                if (self.flags & FLAG_NEG == 0) && (self.flags & FLAG_ZERO == 0) {
                    self.pc = imm;
                }
                false
            }
            Opcode::JL => {
                if self.flags & FLAG_NEG != 0 {
                    self.pc = imm;
                }
                false
            }
            Opcode::CALL => {
                self.push(self.bp, mem);
                self.push(self.pc, mem);
                self.bp = self.sp;
                self.pc = imm;
                false
            }
            _ => unreachable!(),
        }
    }

    fn execute(&mut self, mem: &mut Memory) -> bool {
        if let Some(instr) = self.get_instruction(mem) {
            self.pc = self.pc.wrapping_add(8);
            self.execute_instruction(instr, mem)
        } else {
            panic!(
                "Invalid instruction at address {:08X}: {:016X}",
                self.pc,
                mem.read_u64(self.pc)
            );
        }
    }

    pub async fn run(&mut self, mem: &mut Memory) {
        loop {
            clear_background(BLACK);

            let dt = get_frame_time();
            let cycles_to_run = dt * (CPU_HZ as f32) + self.leftover_cycles;
            let cycles_this_frame = cycles_to_run.floor() as u32;
            self.leftover_cycles = cycles_to_run - (cycles_this_frame as f32);
            for _ in 0..cycles_this_frame {
                if self.execute(mem) {
                    return;
                }
            }

            // draw the text grid. first byte is character second byte is 8-bit color attribute
            let grid_data = mem.text_grid();
            for i in 0..TEXT_GRID_SIZE {
                let char_byte = grid_data[i * 2];
                let color_byte = grid_data[i * 2 + 1];
                let x = (i % TEXT_GRID_WIDTH) as f32 * 10.0;
                let y = (i / TEXT_GRID_WIDTH) as f32 * 24.0;
                if char_byte == 0 {
                    continue;
                }
                // rgb332
                let r = (color_byte & 0b11100000) >> 5;
                let g = (color_byte & 0b00011100) >> 2;
                let b = color_byte & 0b00000011;
                let color = Color::new(r as f32 / 7.0, g as f32 / 7.0, b as f32 / 3.0, 1.0);
                draw_text(&(char_byte as char).to_string(), x, y + 24.0, 24.0, color);
            }

            next_frame().await;
        }
    }

    pub fn debugger(&mut self, mem: &mut Memory) {
        println!(
            "PC: {:08X} | SP: {:08X} | BP: {:08X} | FLAGS: {:08X}",
            self.pc, self.sp, self.bp, self.flags
        );
        for i in 0..14 {
            println!("R{:X}: {:08X}", i, self.registers[i]);
        }
        loop {
            print!("Enter command (h for help): ");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();
            let parts: Vec<&str> = input.split_whitespace().collect();
            match parts[0] {
                "h" => {
                    println!("Commands:");
                    println!("h - help");
                    println!("s [<steps>] - step through instructions (default 1)");
                    println!("p - peek current instruction");
                    println!("r - registers");
                    println!("m <addr> <len> - dump memory");
                    println!("q - quit");
                }
                "s" => {
                    let steps = if parts.len() > 1 {
                        parts[1].parse::<u32>().unwrap_or(1)
                    } else {
                        1
                    };
                    for _ in 0..steps {
                        println!(
                            "Executing {}",
                            self.get_instruction(mem).unwrap_or_else(|| {
                                panic!(
                                    "Invalid instruction at address {:08X}: {:016X}",
                                    self.pc,
                                    mem.read_u64(self.pc)
                                )
                            })
                        );
                        if self.execute(mem) {
                            println!("Program halted");
                            return;
                        }
                        println!(
                            "PC: {:08X} | SP: {:08X} | BP: {:08X} | FLAGS: {:08X}",
                            self.pc, self.sp, self.bp, self.flags
                        );
                        for i in 0..14 {
                            println!("R{}: {:08X}", i, self.registers[i]);
                        }
                    }
                }
                "p" => {
                    if let Some(instr) = self.get_instruction(mem) {
                        println!("Current instruction: {}", instr);
                    } else {
                        println!(
                            "Invalid instruction at address {:08X}: {:016X}",
                            self.pc,
                            mem.read_u64(self.pc)
                        );
                    }
                }
                "r" => {
                    for i in 0..14 {
                        println!("R{}: {:08X}", i, self.registers[i]);
                    }
                    println!("SP: {:08X}", self.sp);
                    println!("BP: {:08X}", self.bp);
                    println!("FLAGS: {:08X}", self.flags);
                    println!("PC: {:08X}", self.pc);
                }
                "m" => {
                    let parts: Vec<&str> = input.split_whitespace().collect();
                    if parts.len() != 3 {
                        println!("Usage: m <addr> <len>");
                        continue;
                    }
                    let addr =
                        u32::from_str_radix(parts[1].trim_start_matches("0x"), 16).unwrap_or(0);
                    let len = parts[2].parse::<u32>().unwrap_or(0);
                    for i in 0..len {
                        if i % 16 == 0 {
                            print!("\n{:08X}: ", addr + i);
                        }
                        print!("{:02X} ", mem.read_u8(addr + i));
                    }
                    println!();
                }
                "q" => {
                    println!("Exiting debugger");
                    return;
                }
                _ => {
                    println!("Unknown command: {}", input);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mov() {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        mem.write_u64(
            0,
            Instruction::RI {
                opcode: Opcode::MOVI,
                reg1: Register::R0,
                imm: 42,
            }
            .into(),
        );
        mem.write_u64(
            8,
            Instruction::RR {
                opcode: Opcode::MOV,
                reg1: Register::R1,
                reg2: Register::R0,
            }
            .into(),
        );
        mem.write_u64(
            16,
            Instruction::E {
                opcode: Opcode::HALT,
            }
            .into(),
        );
        loop {
            if cpu.execute(&mut mem) {
                break;
            }
        }
        assert_eq!(cpu.get(Register::R0), 42);
        assert_eq!(cpu.get(Register::R1), 42);
    }

    #[test]
    fn test_add() {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        mem.write_u64(
            0,
            Instruction::RI {
                opcode: Opcode::MOVI,
                reg1: Register::R0,
                imm: 10,
            }
            .into(),
        );
        mem.write_u64(
            8,
            Instruction::RI {
                opcode: Opcode::MOVI,
                reg1: Register::R1,
                imm: 20,
            }
            .into(),
        );
        mem.write_u64(
            16,
            Instruction::RRR {
                opcode: Opcode::ADD,
                reg1: Register::R2,
                reg2: Register::R0,
                reg3: Register::R1,
            }
            .into(),
        );
        mem.write_u64(
            24,
            Instruction::E {
                opcode: Opcode::HALT,
            }
            .into(),
        );
        loop {
            if cpu.execute(&mut mem) {
                break;
            }
        }
        assert_eq!(cpu.get(Register::R2), 30);
    }

    #[test]
    fn test_cmp() {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        mem.write_u64(
            0,
            Instruction::RI {
                opcode: Opcode::MOVI,
                reg1: Register::R0,
                imm: 10,
            }
            .into(),
        );
        mem.write_u64(
            8,
            Instruction::RI {
                opcode: Opcode::MOVI,
                reg1: Register::R1,
                imm: 20,
            }
            .into(),
        );
        mem.write_u64(
            16,
            Instruction::RR {
                opcode: Opcode::CMP,
                reg1: Register::R0,
                reg2: Register::R1,
            }
            .into(),
        );
        mem.write_u64(
            24,
            Instruction::E {
                opcode: Opcode::HALT,
            }
            .into(),
        );
        loop {
            if cpu.execute(&mut mem) {
                break;
            }
        }
        assert_eq!(cpu.flags & FLAG_ZERO, 0);
        assert_eq!(cpu.flags & FLAG_NEG, FLAG_NEG);
        assert_eq!(cpu.flags & FLAG_CARRY, FLAG_CARRY);
    }

    #[test]
    fn test_jmp() {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        mem.write_u64(
            0,
            Instruction::J {
                opcode: Opcode::JMP,
                imm: 16,
            }
            .into(),
        );
        mem.write_u64(
            8,
            Instruction::E {
                opcode: Opcode::HALT,
            }
            .into(),
        );
        mem.write_u64(
            16,
            Instruction::RI {
                opcode: Opcode::MOVI,
                reg1: Register::R0,
                imm: 42,
            }
            .into(),
        );
        mem.write_u64(
            24,
            Instruction::E {
                opcode: Opcode::HALT,
            }
            .into(),
        );
        loop {
            if cpu.execute(&mut mem) {
                break;
            }
        }
        assert_eq!(cpu.get(Register::R0), 42);
    }

    #[test]
    fn test_call() {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        mem.write_u64(
            0,
            Instruction::J {
                opcode: Opcode::CALL,
                imm: 16,
            }
            .into(),
        );
        mem.write_u64(
            8,
            Instruction::E {
                opcode: Opcode::HALT,
            }
            .into(),
        );
        mem.write_u64(
            16,
            Instruction::RI {
                opcode: Opcode::MOVI,
                reg1: Register::R0,
                imm: 42,
            }
            .into(),
        );
        mem.write_u64(
            24,
            Instruction::E {
                opcode: Opcode::RET,
            }
            .into(),
        );
        loop {
            if cpu.execute(&mut mem) {
                break;
            }
        }
        assert_eq!(cpu.get(Register::R0), 42);
    }

    #[test]
    fn test_memory() {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        mem.write_u64(
            0,
            Instruction::RI {
                opcode: Opcode::MOVI,
                reg1: Register::R0,
                imm: 42,
            }
            .into(),
        );
        mem.write_u64(
            8,
            Instruction::M {
                opcode: Opcode::STORE,
                mode: MemoryMode::Direct,
                reg1: Register::R0,
                reg2: Register::R0,
                imm: 0x100,
            }
            .into(),
        );
        mem.write_u64(
            16,
            Instruction::M {
                opcode: Opcode::LOAD,
                mode: MemoryMode::Direct,
                reg1: Register::R1,
                reg2: Register::R0,
                imm: 0x100,
            }
            .into(),
        );
        mem.write_u64(
            24,
            Instruction::E {
                opcode: Opcode::HALT,
            }
            .into(),
        );
        loop {
            if cpu.execute(&mut mem) {
                break;
            }
        }
        assert_eq!(cpu.get(Register::R1), 42);
    }

    #[test]
    #[should_panic(expected = "Division by zero")]
    fn test_division_by_zero() {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        mem.write_u64(
            0,
            Instruction::RI {
                opcode: Opcode::MOVI,
                reg1: Register::R0,
                imm: 10,
            }
            .into(),
        );
        mem.write_u64(
            8,
            Instruction::RI {
                opcode: Opcode::MOVI,
                reg1: Register::R1,
                imm: 0,
            }
            .into(),
        );
        mem.write_u64(
            16,
            Instruction::RRR {
                opcode: Opcode::DIV,
                reg1: Register::R2,
                reg2: Register::R0,
                reg3: Register::R1,
            }
            .into(),
        );
        mem.write_u64(
            24,
            Instruction::E {
                opcode: Opcode::HALT,
            }
            .into(),
        );
        loop {
            if cpu.execute(&mut mem) {
                break;
            }
        }
    }
}
