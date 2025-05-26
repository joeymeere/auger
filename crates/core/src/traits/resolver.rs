use crate::models::{MemoryAccess, TypeRegistry};
use crate::memory::MemoryMap;

pub trait AugerResolver {
    fn name(&self) -> &'static str;
    /// Resolve types using this resolver's strategy
    fn resolve(&self, memory_map: &MemoryMap, type_registry: &mut TypeRegistry);
    /// Check if this resolver can handle the given memory access
    fn can_handle(&self, access: &MemoryAccess) -> bool;
}