mod wrapper;

use tokio::runtime::Runtime;
use wasi_common::sync::WasiCtxBuilder;
use wasi_common::WasiCtx;

use core::slice;
use std::cell::UnsafeCell;
use std::sync::{Mutex, OnceLock};

pub const PAGE_SIZE: usize = 4096;

static CTX: OnceLock<Mutex<WasiCtx>> = OnceLock::new();
static RUNTIME: OnceLock<Runtime> = OnceLock::new();

#[repr(C)]
pub(crate) struct ExecEnv {
    memory: *mut u8,
    memory_size: i32,
    migration_state: i32,
    argc: i32,
    argv: *mut *mut u8,
}

pub(crate) fn memory<'a>(exec_env: &'a ExecEnv) -> wiggle::GuestMemory<'a> {
    let slice = unsafe {
        slice::from_raw_parts_mut(exec_env.memory, exec_env.memory_size as usize * PAGE_SIZE)
    };
    let cell_slice: &[UnsafeCell<u8>] = unsafe { &*(slice as *mut [u8] as *mut [UnsafeCell<u8>]) };
    let memory = wiggle::GuestMemory::Shared(cell_slice);
    memory
}

pub(crate) fn get_ctx_mut(exec_env: &ExecEnv) -> &'static Mutex<WasiCtx> {
    CTX.get_or_init(|| {
        let mut builder = WasiCtxBuilder::new();
        let mut builder = builder.inherit_stdin().inherit_stdout().inherit_stderr();
        let mut buider = builder.inherit_args();
        let mut builder = builder.inherit_env().unwrap();
        /*
        for i in 0..exec_env.argc {
            let arg = unsafe { *exec_env.argv.offset(i as isize) };
            let arg = unsafe { std::ffi::CStr::from_ptr(arg as *const i8) };
            let arg = arg.to_str().unwrap();
            builder = builder.arg(arg).expect("failed to load CLI arguments");
        }
        */
        Mutex::new(builder.build())
    })
}

pub(crate) fn get_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        let runtime = Runtime::new().unwrap();
        runtime
    })
}