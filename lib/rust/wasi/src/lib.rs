use tokio::runtime::Runtime;
use wasi_common::snapshots::preview_1::wasi_snapshot_preview1 as preview1;
use wasi_common::sync::WasiCtxBuilder;
use wasi_common::WasiCtx;

use core::slice;
use std::cell::UnsafeCell;
use std::sync::{Mutex, OnceLock};

const PAGE_SIZE: usize = 4096;

static CTX: OnceLock<Mutex<WasiCtx>> = OnceLock::new();
static RUNTIME: OnceLock<Runtime> = OnceLock::new();

#[repr(C)]
pub struct ExecEnv {
    memory: *mut u8,
    memory_size: i32,
    migration_state: i32,
}

fn memory<'a>(exec_env: &'a ExecEnv) -> wiggle::GuestMemory<'a> {
    let slice = unsafe {
        slice::from_raw_parts_mut(exec_env.memory, exec_env.memory_size as usize * PAGE_SIZE)
    };
    let cell_slice: &[UnsafeCell<u8>] = unsafe { &*(slice as *mut [u8] as *mut [UnsafeCell<u8>]) };
    let memory = wiggle::GuestMemory::Shared(cell_slice);
    memory
}

fn get_ctx_mut() -> &'static Mutex<WasiCtx> {
    CTX.get_or_init(|| {
        let wctx = WasiCtxBuilder::new()
            .inherit_stdin()
            .inherit_stdout()
            .inherit_stderr()
            .build();
        Mutex::new(wctx)
    })
}

fn get_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        let runtime = Runtime::new().unwrap();
        runtime
    })
}

fn unwrap<E: std::fmt::Debug>(res: Result<i32, E>) -> i32 {
    match res {
        Ok(val) => val,
        Err(e) => panic!("WASI error: {:?}", e),
    }
}

#[no_mangle]
pub extern "C" fn fd_write(exec_env: &ExecEnv, arg0: i32, arg1: i32, arg2: i32, arg3: i32) -> i32 {
    let mut ctx = get_ctx_mut().lock().unwrap();
    let mut memory = memory(exec_env);
    let res = get_runtime().block_on(preview1::fd_write(
        &mut *ctx,
        &mut memory,
        arg0,
        arg1,
        arg2,
        arg3,
    ));
    unwrap(res)
}
