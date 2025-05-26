use std::fmt;

use ezbpf_core::{
    opcodes::OpCode,
    instructions::Ix,
};

#[derive(Debug, Clone)]
pub enum DataReference {
    /// Reference to a string
    String(String),
    /// Reference to an integer value
    Integer(i64),
    /// Reference to a function
    Function(String),
    /// Reference to an unknown data type
    Unknown(u64),
}

impl fmt::Display for DataReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataReference::String(s) => write!(f, "\"{}\"", s),
            DataReference::Integer(i) => write!(f, "{}", i),
            DataReference::Function(name) => write!(f, "fn {}", name),
            DataReference::Unknown(addr) => write!(f, "0x{:x}", addr),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RichInstruction {
    /// Address of the instruction
    pub address: u64,
    /// Raw instruction if successfully parsed
    pub instruction: Option<Ix>,
    /// Opcode of the instruction
    pub opcode: OpCode,
    /// Destination register
    pub dst_reg: u8,
    /// Source register
    pub src_reg: u8,
    /// Offset field
    pub offset: i16,
    /// Immediate value
    pub imm: i32,
    /// Any data references found
    pub references: Option<DataReference>,
}

impl RichInstruction {
    pub fn opcode_name(&self) -> &'static str {
        if let Some(instruction) = &self.instruction {
            return Into::<&str>::into(instruction.op.clone());
        }

        match self.opcode {
            OpCode::Lddw => "lddw",
            OpCode::Ldxb => "ldxb",
            OpCode::Ldxh => "ldxh",
            OpCode::Ldxw => "ldxw",
            OpCode::Ldxdw => "ldxdw",
            OpCode::Stb => "stb",
            OpCode::Sth => "sth",
            OpCode::Stw => "stw",
            OpCode::Stdw => "stdw",
            OpCode::Stxb => "stxb",
            OpCode::Stxh => "stxh",
            OpCode::Stxw => "stxw",
            OpCode::Stxdw => "stxdw",
            OpCode::Add32Imm => "add32",
            OpCode::Add32Reg => "add32",
            OpCode::Sub32Imm => "sub32",
            OpCode::Sub32Reg => "sub32",
            OpCode::Mul32Imm => "mul32",
            OpCode::Mul32Reg => "mul32",
            OpCode::Div32Imm => "div32",
            OpCode::Div32Reg => "div32",
            OpCode::Or32Imm => "or32",
            OpCode::Or32Reg => "or32",
            OpCode::And32Imm => "and32",
            OpCode::And32Reg => "and32",
            OpCode::Lsh32Imm => "lsh32",
            OpCode::Lsh32Reg => "lsh32",
            OpCode::Rsh32Imm => "rsh32",
            OpCode::Rsh32Reg => "rsh32",
            OpCode::Neg32 => "neg32",
            OpCode::Mod32Imm => "mod32",
            OpCode::Mod32Reg => "mod32",
            OpCode::Xor32Imm => "xor32",
            OpCode::Xor32Reg => "xor32",
            OpCode::Mov32Imm => "mov32",
            OpCode::Mov32Reg => "mov32",
            OpCode::Arsh32Imm => "arsh32",
            OpCode::Arsh32Reg => "arsh32",
            OpCode::Lmul32Imm => "lmul32",
            OpCode::Lmul32Reg => "lmul32",
            OpCode::Udiv32Imm => "udiv32",
            OpCode::Udiv32Reg => "udiv32",
            OpCode::Urem32Imm => "urem32",
            OpCode::Urem32Reg => "urem32",
            OpCode::Sdiv32Imm => "sdiv32",
            OpCode::Sdiv32Reg => "sdiv32",
            OpCode::Srem32Imm => "srem32",
            OpCode::Srem32Reg => "srem32",
            OpCode::Le => "le",
            OpCode::Be => "be",
            OpCode::Add64Imm => "add64",
            OpCode::Add64Reg => "add64",
            OpCode::Sub64Imm => "sub64",
            OpCode::Sub64Reg => "sub64",
            OpCode::Mul64Imm => "mul64",
            OpCode::Mul64Reg => "mul64",
            OpCode::Div64Imm => "div64",
            OpCode::Div64Reg => "div64",
            OpCode::Or64Imm => "or64",
            OpCode::Or64Reg => "or64",
            OpCode::And64Imm => "and64",
            OpCode::And64Reg => "and64",
            OpCode::Lsh64Imm => "lsh64",
            OpCode::Lsh64Reg => "lsh64",
            OpCode::Rsh64Imm => "rsh64",
            OpCode::Rsh64Reg => "rsh64",
            OpCode::Neg64 => "neg64",
            OpCode::Mod64Imm => "mod64",
            OpCode::Mod64Reg => "mod64",
            OpCode::Xor64Imm => "xor64",
            OpCode::Xor64Reg => "xor64",
            OpCode::Mov64Imm => "mov64",
            OpCode::Mov64Reg => "mov64",
            OpCode::Arsh64Imm => "arsh64",
            OpCode::Arsh64Reg => "arsh64",
            OpCode::Hor64Imm => "hor64",
            OpCode::Lmul64Imm => "lmul64",
            OpCode::Lmul64Reg => "lmul64",
            OpCode::Uhmul64Imm => "uhmul64",
            OpCode::Uhmul64Reg => "uhmul64",
            OpCode::Udiv64Imm => "udiv64",
            OpCode::Udiv64Reg => "udiv64",
            OpCode::Urem64Imm => "urem64",
            OpCode::Urem64Reg => "urem64",
            OpCode::Shmul64Imm => "shmul64",
            OpCode::Shmul64Reg => "shmul64",
            OpCode::Sdiv64Imm => "sdiv64",
            OpCode::Sdiv64Reg => "sdiv64",
            OpCode::Srem64Imm => "srem64",
            OpCode::Srem64Reg => "srem64",
            OpCode::Ja => "ja",
            OpCode::JeqImm => "jeq",
            OpCode::JeqReg => "jeq",
            OpCode::JgtImm => "jgt",
            OpCode::JgtReg => "jgt",
            OpCode::JgeImm => "jge",
            OpCode::JgeReg => "jge",
            OpCode::JltImm => "jlt",
            OpCode::JltReg => "jlt",
            OpCode::JleImm => "jle",
            OpCode::JleReg => "jle",
            OpCode::JsetImm => "jset",
            OpCode::JsetReg => "jset",
            OpCode::JneImm => "jne",
            OpCode::JneReg => "jne",
            OpCode::JsgtImm => "jsgt",
            OpCode::JsgtReg => "jsgt",
            OpCode::JsgeImm => "jsge",
            OpCode::JsgeReg => "jsge",
            OpCode::JsltImm => "jslt",
            OpCode::JsltReg => "jslt",
            OpCode::JsleImm => "jsle",
            OpCode::JsleReg => "jsle",
            OpCode::Call => "call",
            OpCode::Callx => "callx",
            OpCode::Exit => "exit",
            _ => "UNKNOWN",
        }
    }

    /// Format the instruction as a string with reference information
    pub fn to_string(&self) -> String {
        let base = format!("0x{:08x}: ", self.address);
        
        // If we have a parsed instruction, use ezbpf_core's to_asm
        if let Some(instruction) = &self.instruction {
            if let Ok(asm) = instruction.to_asm() {
                // For LDDW with string reference, add the reference information
                if instruction.op == OpCode::Lddw && self.references.is_some() {
                    return format!("{}{} ; {}", base, asm, self.references.as_ref().unwrap());
                }
                return format!("{}{}", base, asm);
            }
        }
        let op_name = self.opcode_name();
        let mut result = format!("{}{} R{}", base, op_name, self.dst_reg);
        match self.opcode {
            OpCode::Mov64Imm => result += &format!(", 0x{:x}", self.imm), 
            OpCode::Div64Reg => result += &format!(", 0x{:x}", self.imm), 
            OpCode::Call => result += &format!(", helper[{}]", self.imm), 
            OpCode::Lddw => { 
                // lddw will often have a reference
                if let Some(reference) = &self.references {
                    result += &format!(", {}", reference);
                } else {
                    result += &format!(", 0x{:x}", self.imm);
                }
            }
            _ => {
                if self.src_reg != 0 {
                    result += &format!(", R{}", self.src_reg);
                }
                if self.imm != 0 {
                    result += &format!(", 0x{:x}", self.imm);
                }
                if self.offset != 0 {
                    result += &format!(", off {}", self.offset);
                }
            }
        }
        
        result
    }
}

#[derive(Debug, Clone)]
pub struct MemoryAccess {
    /// Address being accessed
    pub address: u64,
    /// Type of access (read/write)
    pub access_type: AccessType,
    /// Size of access in bytes
    pub size: u32,
    /// Instruction performing the access
    pub instruction: RichInstruction,
}

#[derive(Debug, Clone)]
pub enum AccessType {
    Read,
    Write,
}
