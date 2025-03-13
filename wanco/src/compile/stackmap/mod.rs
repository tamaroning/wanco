mod llvm_stackmap;
pub mod regs;
pub use llvm_stackmap::*;

/// Create a stackmap ID from a function index and instruction offset.
pub fn stackmap_id(func_idx: u32, insn_offset: u32) -> u64 {
    ((func_idx as u64) << 32) | insn_offset as u64
}

/// Split a stackmap ID into its function index and instruction offset parts.
pub fn stackmap_id_parts(id: u64) -> (u32, u32) {
    let func_idx = (id >> 32) as u32;
    let insn_offset = id as u32;
    (func_idx, insn_offset)
}
