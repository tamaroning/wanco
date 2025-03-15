mod wrapper;

use tokio::runtime::Runtime;
use wasi_common::sync::WasiCtxBuilder;
use wasi_common::WasiCtx;

use core::slice;
use std::cell::UnsafeCell;
use std::os::raw::c_char;
use std::sync::{Mutex, OnceLock};

//pub(crate) const PAGE_SIZE: usize = 65536;

static CTX: OnceLock<Mutex<WasiCtx>> = OnceLock::new();
static RUNTIME: OnceLock<Runtime> = OnceLock::new();

#[repr(C, packed)]
pub(crate) struct ExecEnv {
    memory: *mut u8,
    memory_size: i32,
    migration_state: i32,
    argc: i32,
    argv: *mut *mut c_char,
    safepoint: *const i8,
}

pub(crate) fn memory<'a>(exec_env: &'a ExecEnv) -> wiggle::GuestMemory<'a> {
    let memory_size = 4 * 1024 * 1024;
    let slice = unsafe { slice::from_raw_parts_mut(exec_env.memory, memory_size) };
    let cell_slice: &[UnsafeCell<u8>] = unsafe { &*(slice as *mut [u8] as *mut [UnsafeCell<u8>]) };
    let memory = wiggle::GuestMemory::Shared(cell_slice);
    memory
}

pub(crate) fn get_ctx_mut(exec_env: &ExecEnv) -> &'static Mutex<WasiCtx> {
    CTX.get_or_init(|| {
        let mut builder = WasiCtxBuilder::new();
        let mut builder = builder.inherit_stdin().inherit_stdout().inherit_stderr();
        //let mut buider = builder.inherit_args();
        builder = builder.inherit_env().unwrap();
        let dir = cap_std::fs::Dir::from_std_file(std::fs::File::open(".").unwrap());
        builder = builder.preopened_dir(dir, "/").unwrap();
        let mut saw_dashdash = false;
        for i in 0..exec_env.argc {
            let arg = unsafe { *exec_env.argv.offset(i as isize) };
            let arg = unsafe { std::ffi::CStr::from_ptr(arg as *const c_char) };
            let arg = arg.to_str().unwrap();
            if arg == "--" {
                saw_dashdash = true;
                continue;
            } else if i != 0 && !saw_dashdash {
                continue;
            }
            builder = builder.arg(arg).expect("failed to load CLI arguments");
        }

        Mutex::new(builder.build())
    })
}

pub(crate) fn get_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        let runtime = Runtime::new().unwrap();
        runtime
    })
}
