mod compile_function;
mod compile_global;
mod compile_memory;
mod compile_module;
mod compile_type;
pub mod control;
pub mod cr;
pub mod cr_v2;
pub mod helper;
mod synthesize;

pub use compile_module::compile_module;
