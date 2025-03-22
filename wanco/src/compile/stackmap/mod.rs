mod llvm_stackmap;
pub mod regs;
pub use llvm_stackmap::*;

/// Create a stackmap ID from a function index and instruction offset.
pub fn stackmap_id(func_idx: u32, insn_offset: u32) -> u64 {
    ((func_idx as u64) << 32) | insn_offset as u64
}
