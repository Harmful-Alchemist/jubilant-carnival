use std::arch::asm;
use std::vec::Vec;

use crate::assembly::InstructionFormat::*;
use crate::assembly::Register::*;
use crate::assembly::RegisterOrOffset::*;
use crate::assembly::SupportedInstruction::*;

#[derive(Debug)]
enum SupportedInstruction {
    Add,
    AddI,
    Blt,
    //   Comment,
    //   Label, TODO later, they're also not instructions...
}

impl SupportedInstruction {
    pub fn info(&self) -> InstructionInfo {
        match self {
            Add => InstructionInfo {
                format: R,
                opcode: 0b0110011,
                funct3: 0,
                funct7: 0,
            },
            AddI => InstructionInfo {
                format: I,
                opcode: 0b0010011,
                funct3: 0,
                funct7: 0,
            },
            _ => panic!("Asked for info for an instruction that won't be really exectued!"),
        }
    }
}

#[derive(Debug)]
enum Register {
    X0,
    X5,
    X6,
    X7,
    X28,
    X31,
}

impl Register {
    fn parse(reg: &str) -> Result<Self, String> {
        Self::from_str(reg.replace(',', "").trim())
    }

    fn from_str(reg: &str) -> Result<Self, String> {
        match reg {
            "x0" => Ok(X0),
            "x5" => Ok(X5),
            "x6" => Ok(X6),
            "x7" => Ok(X7),
            "x28" => Ok(X28),
            "x31" => Ok(X31),
            _ => Err(format!(
                "I only understand a limited number of registers, not '{}'",
                reg
            )),
        }
    }

    fn to_code(&self) -> i16 {
        match self {
            X0 => 0,
            X5 => 5,
            X6 => 6,
            X7 => 7,
            X28 => 28,
            X31 => 31,
        }
    }
}

#[derive(Debug)]
enum RegisterOrOffset {
    Offset(i16),
    Register_(Register),
}

impl RegisterOrOffset {
    fn to_code(&self) -> i16 {
        match self {
            Offset(n) => *n,
            Register_(r) => r.to_code(),
        }
    }
}

#[derive(Debug)]
struct Instruction {
    instruction: SupportedInstruction,
    rd: Register, //Could be options but whatever
    rs1: Register,
    offset_or_rs2: RegisterOrOffset,
}

impl Instruction {
    fn parse(line: &str) -> Result<Instruction, String> {
        let line = line.trim();
        //    TODO instuctions   if ["#", "//", "--"].iter().any(|x| line.starts_with(x)) {
        //         return Ok(Instruction::comment());
        //   }
        //
        let split1: Vec<&str> = line.split(' ').collect();
        if split1.len() != 4 {
            return Err(format!("Can't read '{line}'"));
        }

        let supported;
        let rd;
        let rs1;
        let offset_or_rs2;
        match split1[0] {
            "add" | "ADD" => {
                supported = Add;
                rd = Register::parse(split1[1])?;
                rs1 = Register::parse(split1[2])?;
                offset_or_rs2 = Register_(Register::parse(split1[3])?);
            }
            "addi" | "ADDI" => {
                supported = AddI;
                rd = Register::parse(split1[1])?;
                rs1 = Register::parse(split1[2])?;
                let offset = split1[3]
                    .parse()
                    .map_err(|_| format!("Can't parse '{}' as an offset", split1[3]))?;
                offset_or_rs2 = Offset(offset);
            }
            "blt" | "BLT" => {
                supported = Blt;
                rd = Register::parse(split1[1])?;
                rs1 = Register::parse(split1[2])?;
                let offset = split1[3]
                    .parse()
                    .map_err(|_| format!("Can't parse '{}' as an offset", split1[3]))?;
                if !(-2048..=2047).contains(&offset) {
                    return Err(format!("{offset} can only be between -2048 and 2047"));
                }
                offset_or_rs2 = Offset(offset);
            }
            x => {
                return Err(format!(
                    "I only know a very limited amount of instructions and not '{x}'"
                ))
            }
        }

        Ok(Instruction {
            instruction: supported,
            rd,
            rs1,
            offset_or_rs2,
        })
    }

    pub fn to_code(&self) -> i32 {
        let InstructionInfo {
            format,
            opcode,
            funct3,
            funct7,
        } = self.instruction.info();
        let (rd, rs1, rs2_or_imm) = (
            self.rd.to_code(),
            self.rs1.to_code(),
            self.offset_or_rs2.to_code(),
        );
        match format {
            R => i32::from_str_radix(
                &format!("{funct7:07b}{rs2_or_imm:05b}{rs1:05b}{funct3:03b}{rd:05b}{opcode:07b}"),
                2,
            )
            .unwrap(), //TODO bubble errors or we know parsed etc. here.
            I => {
                let rs2_or_imm = rs2_or_imm & 0xFFF; //Take only the last 12 bits.
                let bin_str =
                    &format!("{rs2_or_imm:012b}{rs1:05b}{funct3:03b}{rd:05b}{opcode:07b}");
                u32::from_str_radix(bin_str, 2).unwrap() as i32
            }
        }
    }
}

#[derive(Debug)]
struct InstructionInfo {
    format: InstructionFormat,
    opcode: u8,
    funct3: u8,
    funct7: u8,
}

#[derive(Debug)]
enum InstructionFormat {
    R,
    I,
    //    SB,
}

#[derive(Debug)]
pub struct Interpreter {
    pub line: usize,
    program: Vec<Instruction>,
    //    instruction_info: &'static HashMap<Instruction, InstructionInfo>,
    pub registers: [i32; 31],
}

impl Interpreter {
    pub fn new(in_program: Vec<String>) -> Result<Self, String> {
        let mut program = Vec::new();
        for (i, line) in in_program.into_iter().enumerate() {
            match Instruction::parse(&line) {
                Ok(instruction) => program.push(instruction),
                Err(e) => return Err(format!("Error on line {i}: {e}")),
            }
        }

        Ok(Self {
            line: 0,
            program,
            registers: [0; 31],
        })
    }

    pub fn step(&mut self) -> Option<()> {
        if self.line >= self.program.len() {
            return None;
        }

        let instruction = &self.program[self.line];
        match instruction.instruction {
            Add | AddI => self.real_step(instruction.to_code()),
            Blt => {
                if let Offset(n) = instruction.offset_or_rs2
                    && self.registers(&instruction.rd) < self.registers(&instruction.rs1) + 1
                {
                    //let n = if n < 0 { n - 1 } else { n }; //Have to subtract current line too.
                    self.line = (self.line as i64 + n as i64) as usize;
                    //if self.line >= self.program.len() {
                    //    return None;
                    //}
                    return Some(());
                }
            }
        }

        self.line += 1;
        Some(())
    }

    fn registers(&self, register: &Register) -> i32 {
        match register {
            X0 => 0,
            X5 => self.registers[4],
            X6 => self.registers[5],
            X7 => self.registers[6],
            X28 => self.registers[27],
            X31 => self.registers[30],
        }
    }

    fn real_step(&mut self, code: i32) {
        #[allow(unused_mut)]
        #[allow(unused_variables)]
        #[allow(unused_assignments)]
        let [mut x1, mut x2, mut x3, mut x4, mut x5, mut x6, mut x7, mut x8, mut x9, mut x10, mut x11, mut x12, mut x13, mut x14, mut x15, mut x16, mut x17, mut x18, mut x19, mut x20, mut x21, mut x22, mut x23, mut x24, mut x25, mut x26, mut x27, mut x28, mut x29, mut x30, mut x31] =
            self.registers;

        let instruction: [i32; 2] = [
            code,
            0b00000000000000001000000001100111, //jalr x0, 0(x1)
        ];

        let instruction_pointer: *const [i32; 2] = &instruction;
        x30 = instruction_pointer as i32;

        // Oh hmm DRAM vs IRAM https://www.espressif.com/sites/default/files/documentation/esp32-c3_technical_reference_manual_en.pdf#sysmem
        let diff_data_and_instruction_bus = 0x70_0000;
        x29 = diff_data_and_instruction_bus;

        unsafe {
            asm!(
            "addi sp, sp, -4", // Add space on stack
            "sw x1, 0(sp)", // Store return address on stack
            "add x30, x30, x29", //Up the data bus known address to get the shadowed
                                 //instruction bus address.
            "jalr x1, 0(x30)",
            "lw x1, 0(sp)", // Restore return address
            "addi sp, sp, 4", //pop!

            // Only the temporaries allow input (for the end-user not the code here), looks like we take off two (x29 and x30) from six, pfew.... Maybe
            // misuse argument registers? Or something the **caller** should have saved? Check
            // it.
            // Not all registers can be used as out...
            out("x1") x1,
            inout("x5") x5,
            inout("x6") x6,
            inout("x7") x7,
            out("x10") x10,
            out("x11") x11,
            out("x12") x12,
            out("x13") x13,
            out("x14") x14,
            out("x15") x15,
            out("x16") x16,
            out("x17") x17,
            out("x18") x18,
            out("x19") x19,
            out("x20") x20,
            out("x21") x21,
            out("x22") x22,
            out("x23") x23,
            out("x24") x24,
            out("x25") x25,
            out("x26") x26,
            out("x27") x27,
            inout("x28") x28,
            inout("x29") x29,
            inout("x30") x30,
            inout("x31") x31,
            );
        }

        self.registers = [
            x1, x2, x3, x4, x5, x6, x7, x8, x9, x10, x11, x12, x13, x14, x15, x16, x17, x18, x19,
            x20, x21, x22, x23, x24, x25, x26, x27, x28, x29, x30, x31,
        ];
    }
}
