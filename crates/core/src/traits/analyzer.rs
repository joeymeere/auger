use crate::memory::MemoryMap;
use crate::models::{FunctionBlock, ControlFlow, MemoryReference};

pub trait AugerAnalyzer {
    fn name(&self) -> &'static str;
    /// Analyze the program and return function blocks
    fn find_functions(&self, memory_map: &MemoryMap) -> Vec<FunctionBlock>;
    /// Map control flow between function blocks
    fn map_control_flow(&self, memory_map: &MemoryMap, functions: &[FunctionBlock]) -> Vec<ControlFlow>;
    /// Find and map memory references
    fn find_memory_refs(&self, memory_map: &MemoryMap) -> Vec<MemoryReference>;
    /// Check if this analyzer can handle the given program
    fn can_handle(&self, memory_map: &MemoryMap) -> bool;
}