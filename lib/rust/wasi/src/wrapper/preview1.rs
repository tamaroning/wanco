use wasi_common::snapshots::preview_1::wasi_snapshot_preview1 as preview1;

use crate::{get_ctx_mut, get_runtime, memory, ExecEnv};

use super::unwrap;

macro_rules! wasi_function {
    ($name:ident,  $( $arg_name: ident : $arg_ty: ty ),*) => {
        #[no_mangle]
        pub extern "C" fn $name(exec_env: &ExecEnv, $( $arg_name: $arg_ty ),*) -> i32 {
            let mut ctx = get_ctx_mut().lock().unwrap();
            let mut memory = memory(exec_env);
            let res = get_runtime().block_on(preview1::$name(
                &mut *ctx,
                &mut memory,
                $( $arg_name ),*
            ));
            unwrap(res)
        }
    }
}

wasi_function!(args_get, arg0: i32, arg1: i32);
wasi_function!(args_sizes_get, arg0: i32, arg1: i32);
wasi_function!(clock_res_get, arg0: i32, arg1: i32);
wasi_function!(clock_time_get, arg0: i32, arg1: i64, arg2: i32);
wasi_function!(environ_get, arg0: i32, arg1: i32);
wasi_function!(environ_sizes_get, arg0: i32, arg1: i32);
wasi_function!(fd_advise, arg0: i32, arg1: i64, arg2: i64, arg3: i32);
wasi_function!(fd_allocate, arg0: i32, arg1: i64, arg2: i64);
wasi_function!(fd_close, arg0: i32);
wasi_function!(fd_datasync, arg0: i32);
wasi_function!(fd_fdstat_get, arg0: i32, arg1: i32);
wasi_function!(fd_fdstat_set_flags, arg0: i32, arg1: i32);
wasi_function!(fd_fdstat_set_rights, arg0: i32, arg1: i64, arg2: i64);
wasi_function!(fd_filestat_get, arg0: i32, arg1: i32);
wasi_function!(fd_filestat_set_size, arg0: i32, arg1: i64);
wasi_function!(fd_filestat_set_times, arg0: i32, arg1: i64, arg2: i64, arg3: i32);
wasi_function!(fd_pread, arg0: i32, arg1: i32, arg2: i32, arg3: i64, arg4: i32);
wasi_function!(fd_prestat_dir_name, arg0: i32, arg1: i32, arg2: i32);
wasi_function!(fd_prestat_get, arg0: i32, arg1: i32);
wasi_function!(fd_pwrite, arg0: i32, arg1: i32, arg2: i32, arg3: i64, arg4: i32);
wasi_function!(fd_read, arg0: i32, arg1: i32, arg2: i32, arg3: i32);
wasi_function!(fd_readdir, arg0: i32, arg1: i32, arg2: i32, arg3: i64, arg4: i32);
wasi_function!(fd_renumber, arg0: i32, arg1: i32);
wasi_function!(fd_seek, arg0: i32, arg1: i64, arg2: i32, arg3: i32);
wasi_function!(fd_sync, arg0: i32);
wasi_function!(fd_tell, arg0: i32, arg1: i32);
wasi_function!(fd_write, arg0: i32, arg1: i32, arg2: i32, arg3: i32);
wasi_function!(path_create_directory, arg0: i32, arg1: i32, arg2: i32);
wasi_function!(path_filestat_get, arg0: i32, arg1: i32, arg2: i32, arg3 :i32, arg4: i32);
wasi_function!(path_filestat_set_times, arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i64, arg5: i64, arg6: i32);
wasi_function!(path_link, arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32, arg5: i32, arg6: i32);
wasi_function!(path_open, arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32, arg5: i64, arg6: i64, arg7: i32, arg8:i32);
wasi_function!(path_readlink, arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32, arg5: i32);
wasi_function!(path_remove_directory, arg0: i32, arg1: i32, arg2: i32);
wasi_function!(path_rename, arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32, arg5: i32);
wasi_function!(path_symlink, arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32);
wasi_function!(path_unlink_file, arg0: i32, arg1: i32, arg2: i32);
wasi_function!(poll_oneoff, arg0: i32, arg1: i32, arg2: i32, arg3: i32);
wasi_function!(proc_raise, arg0: i32);
wasi_function!(random_get, arg0: i32, arg1: i32);
wasi_function!(sched_yield,);
wasi_function!(sock_accept, arg0: i32, arg1: i32, arg2: i32);
wasi_function!(sock_recv, arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32, arg5: i32);
wasi_function!(sock_send, arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32);
wasi_function!(sock_shutdown, arg0: i32, arg1: i32);

#[no_mangle]
pub extern "C" fn proc_exit(exec_env: &ExecEnv, arg0: i32) -> () {
    let mut ctx = get_ctx_mut().lock().unwrap();
    let mut memory = memory(exec_env);
    let res = get_runtime().block_on(preview1::proc_exit(
        &mut *ctx,
        &mut memory,
        arg0,
    ));
    unwrap(res)
}
