use std::collections::HashMap;

use ezbpf_core::{
    program::Program,
    opcodes::OpCode,
    instructions::Ix,
};

use crate::models::{DataReference, MemoryAccess, RichInstruction};

pub struct MemoryMap {
    /// ELF sections (name -> (addr, size, content))
    pub sections: HashMap<String, (u64, u64, Vec<Ix>, Vec<u8>)>,
    pub strings: HashMap<u64, String>,
    pub references: HashMap<u64, Vec<u64>>,
    pub instructions: Vec<RichInstruction>,
    pub access_patterns: Vec<MemoryAccess>,
    pub syscall_signatures: HashMap<u64, String>,
}

impl MemoryMap {
    pub fn new(program: &Program, data: &[u8]) -> Self {
        let mut map = MemoryMap {
            sections: HashMap::new(),
            strings: HashMap::new(),
            references: HashMap::new(),
            instructions: Vec::new(),
            access_patterns: Vec::new(),
            syscall_signatures: HashMap::new(),
        };
        
        let mut index = 0;
        for section in &program.section_header_entries {
            let offset = section.offset as usize;
            let size = section.data.len();
            let ixs = program.section_header_entries[index].ixs.clone();

            let mut mem_addr = offset as u64;
            
            for ix in ixs.iter() {
                let instruction = ix.clone();
                
                map.instructions.push(RichInstruction {
                    instruction: Some(instruction.clone()),
                    address: mem_addr,
                    opcode: instruction.op,
                    dst_reg: instruction.dst,
                    src_reg: instruction.src,
                    offset: instruction.off,
                    imm: instruction.imm as i32,
                    references: None,
                });

                // lddw takes up two slots, thus 16 bytes
                if ix.op == OpCode::Lddw {
                    mem_addr += 16;
                } else {
                    mem_addr += 8;
                }
            }
            
            index += 1;
            
            if offset + size <= data.len() {
                map.sections.insert(
                    section.label.clone(), 
                    (section.offset as u64, size as u64, ixs, data[offset..offset+size].to_vec())
                );
            }
        }
        
        map.scan_for_strings();
        
        map.find_ebpf_references();
        
        map
    }

    fn scan_for_strings(&mut self) {
        // look in all sections, especially .rodata and .data
        for (name, (base_addr, _, _, content)) in &self.sections {
            if name == ".text" {
                continue;
            }
            
            let mut pos = 0;
            while pos < content.len() {
                // sequence of printable chars followed by null
                let start = pos;
                while pos < content.len() && 
                      (content[pos] >= 32 && content[pos] < 127 || 
                       content[pos] == b'\t' || content[pos] == b'\n') {
                    pos += 1;
                }
                
                if pos < content.len() && content[pos] == 0 && pos - start > 3 {
                    if let Ok(s) = std::str::from_utf8(&content[start..pos]) {
                        let addr = *base_addr + start as u64;
                        self.strings.insert(addr, s.to_string());
                    }
                }
                pos += 1;
            }
        }
    }

    fn find_ebpf_references(&mut self) {
        if let Some((text_addr, _, _, content)) = self.sections.get(".text") {
            let mut pos = 0;
            while pos + 8 <= content.len() {
                let instr_addr = *text_addr + pos as u64;

                let parsed_instruction = if pos + 16 <= content.len() {
                    // could be lddw, which is 2 slots
                    Ix::from_bytes(&content[pos..pos+16]).ok()
                        .or_else(|| Ix::from_bytes(&content[pos..pos+8]).ok())
                } else {
                    Ix::from_bytes(&content[pos..pos+8]).ok()
                };
                
                if let Some(instruction) = &parsed_instruction {
                    let opcode = &instruction.op;
                    let dst_reg = instruction.dst;
                    let src_reg = instruction.src;
                    let offset = instruction.off;
                    let imm = instruction.imm;
                    
                    if *opcode == OpCode::Lddw && pos + 16 <= content.len() {
                        // extract 64-bit immediate
                        let imm_lo = u32::from_le_bytes([
                            content[pos+4], content[pos+5], content[pos+6], content[pos+7]
                        ]) as u64;
                        let imm_hi = u32::from_le_bytes([
                            content[pos+12], content[pos+13], content[pos+14], content[pos+15]
                        ]) as u64;
                        let imm_64 = imm_lo | (imm_hi << 32);
                        
                        // check if the immediate value points to an identified string
                        let reference = self.strings.get(&imm_64)
                            .map(|s| {
                                self.references.entry(imm_64)
                                    .or_default()
                                    .push(instr_addr);
                                DataReference::String(s.clone())
                            });
                        
                        self.instructions.push(RichInstruction {
                            address: instr_addr,
                            instruction: parsed_instruction.clone(),
                            opcode: opcode.clone(),
                            dst_reg,
                            src_reg,
                            offset,
                            imm: imm as i32,
                            references: reference,
                        });
                        
                        pos += 16
                    } else {
                        self.instructions.push(RichInstruction {
                            address: instr_addr,
                            instruction: parsed_instruction.clone(),
                            opcode: opcode.clone(),
                            dst_reg,
                            src_reg,
                            offset,
                            imm: imm as i32,
                            references: None,
                        });
                        
                        pos += 8;
                    }
                } else {
                    pos += 8;
                }
            }
        }
    }

    pub fn track_access(&mut self, access: MemoryAccess) {
        self.access_patterns.push(access);
    }

    pub fn get_access_patterns(&self, start_addr: u64, end_addr: u64) -> Vec<&MemoryAccess> {
        self.access_patterns.iter()
            .filter(|access| access.address >= start_addr && access.address < end_addr)
            .collect()
    }

    pub fn register_syscall(&mut self, address: u64, signature: String) {
        self.syscall_signatures.insert(address, signature);
    }

    pub fn get_syscall_signature(&self, address: u64) -> Option<&String> {
        self.syscall_signatures.get(&address)
    }

    pub fn get_instructions(&self) -> &[RichInstruction] {
        &self.instructions
    }

    pub fn get_strings(&self) -> &HashMap<u64, String> {
        &self.strings
    }

    pub fn get_references(&self) -> &HashMap<u64, Vec<u64>> {
        &self.references
    }
}