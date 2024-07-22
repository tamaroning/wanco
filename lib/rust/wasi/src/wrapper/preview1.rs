use wasi_common::snapshots::preview_1::wasi_snapshot_preview1 as preview1;

use crate::{get_ctx_mut, get_runtime, memory, ExecEnv};

use super::unwrap;

macro_rules! wasi_function {
    ($export:ident, $name:ident,  $( $arg_name: ident : $arg_ty: ty ),*) => {
        #[no_mangle]
        pub extern "C" fn $export(exec_env: &ExecEnv, $( $arg_name: $arg_ty ),*) -> i32 {
            eprintln!("[debug] call {}{:?}", stringify!($name), ( ($( $arg_name ),*) ));
            let mut ctx = get_ctx_mut(exec_env).lock().unwrap();
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

wasi_function!(wasi_snapshot_preview1_args_get, args_get, arg0: i32, arg1: i32);
wasi_function!(wasi_snapshot_preview1_args_sizes_get, args_sizes_get, arg0: i32, arg1: i32);
wasi_function!(wasi_snapshot_preview1_clock_res_get, clock_res_get, arg0: i32, arg1: i32);
wasi_function!(wasi_snapshot_preview1_clock_time_get,clock_time_get, arg0: i32, arg1: i64, arg2: i32);
wasi_function!(wasi_snapshot_preview1_environ_get, environ_get, arg0: i32, arg1: i32);
wasi_function!(wasi_snapshot_preview1_environ_sizes_get,environ_sizes_get, arg0: i32, arg1: i32);
wasi_function!(wasi_snapshot_preview1_fd_advise,fd_advise, arg0: i32, arg1: i64, arg2: i64, arg3: i32);
wasi_function!(wasi_snapshot_preview1_fd_allocate,fd_allocate, arg0: i32, arg1: i64, arg2: i64);
wasi_function!(wasi_snapshot_preview1_fd_close,fd_close, arg0: i32);
wasi_function!(wasi_snapshot_preview1_fd_datasync,fd_datasync, arg0: i32);
wasi_function!(wasi_snapshot_preview1_fd_fdstat_get,fd_fdstat_get, arg0: i32, arg1: i32);
wasi_function!(wasi_snapshot_preview1_fd_fdstat_set_flags,fd_fdstat_set_flags, arg0: i32, arg1: i32);
wasi_function!(wasi_snapshot_preview1_fd_fdstat_set_rights,fd_fdstat_set_rights, arg0: i32, arg1: i64, arg2: i64);
wasi_function!(wasi_snapshot_preview1_fd_filestat_get,fd_filestat_get, arg0: i32, arg1: i32);
wasi_function!(wasi_snapshot_preview1_fd_filestat_set_size,fd_filestat_set_size, arg0: i32, arg1: i64);
wasi_function!(wasi_snapshot_preview1_fd_filestat_set_times,fd_filestat_set_times, arg0: i32, arg1: i64, arg2: i64, arg3: i32);
wasi_function!(wasi_snapshot_preview1_fd_pread,fd_pread, arg0: i32, arg1: i32, arg2: i32, arg3: i64, arg4: i32);
wasi_function!(wasi_snapshot_preview1_fd_prestat_dir_name,fd_prestat_dir_name, arg0: i32, arg1: i32, arg2: i32);
wasi_function!(wasi_snapshot_preview1_fd_prestat_get,fd_prestat_get, arg0: i32, arg1: i32);
wasi_function!(wasi_snapshot_preview1_fd_pwrite,fd_pwrite, arg0: i32, arg1: i32, arg2: i32, arg3: i64, arg4: i32);
wasi_function!(wasi_snapshot_preview1_fd_read,fd_read, arg0: i32, arg1: i32, arg2: i32, arg3: i32);
wasi_function!(wasi_snapshot_preview1_fd_readdir,fd_readdir, arg0: i32, arg1: i32, arg2: i32, arg3: i64, arg4: i32);
wasi_function!(wasi_snapshot_preview1_fd_renumber,fd_renumber, arg0: i32, arg1: i32);
wasi_function!(wasi_snapshot_preview1_fd_seek,fd_seek, arg0: i32, arg1: i64, arg2: i32, arg3: i32);
wasi_function!(wasi_snapshot_preview1_fd_sync,fd_sync, arg0: i32);
wasi_function!(wasi_snapshot_preview1_fd_tell,fd_tell, arg0: i32, arg1: i32);
wasi_function!(wasi_snapshot_preview1_fd_write,fd_write, arg0: i32, arg1: i32, arg2: i32, arg3: i32);
wasi_function!(wasi_snapshot_preview1_path_create_directory,path_create_directory, arg0: i32, arg1: i32, arg2: i32);
wasi_function!(wasi_snapshot_preview1_path_filestat_get,path_filestat_get, arg0: i32, arg1: i32, arg2: i32, arg3 :i32, arg4: i32);
wasi_function!(wasi_snapshot_preview1_path_filestat_set_times,path_filestat_set_times, arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i64, arg5: i64, arg6: i32);
wasi_function!(wasi_snapshot_preview1_path_link,path_link, arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32, arg5: i32, arg6: i32);
wasi_function!(wasi_snapshot_preview1_path_open,path_open, arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32, arg5: i64, arg6: i64, arg7: i32, arg8:i32);
wasi_function!(wasi_snapshot_preview1_path_readlink,path_readlink, arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32, arg5: i32);
wasi_function!(wasi_snapshot_preview1_path_remove_directory,path_remove_directory, arg0: i32, arg1: i32, arg2: i32);
wasi_function!(wasi_snapshot_preview1_path_rename,path_rename, arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32, arg5: i32);
wasi_function!(wasi_snapshot_preview1_path_symlink,path_symlink, arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32);
wasi_function!(wasi_snapshot_preview1_path_unlink_file,path_unlink_file, arg0: i32, arg1: i32, arg2: i32);
wasi_function!(wasi_snapshot_preview1_poll_oneoff,poll_oneoff, arg0: i32, arg1: i32, arg2: i32, arg3: i32);
wasi_function!(wasi_snapshot_preview1_proc_raise,proc_raise, arg0: i32);
wasi_function!(wasi_snapshot_preview1_random_get,random_get, arg0: i32, arg1: i32);
wasi_function!(wasi_snapshot_preview1_sched_yield, sched_yield,);
wasi_function!(wasi_snapshot_preview1_sock_accept,sock_accept, arg0: i32, arg1: i32, arg2: i32);
wasi_function!(wasi_snapshot_preview1_sock_recv,sock_recv, arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32, arg5: i32);
wasi_function!(wasi_snapshot_preview1_sock_send,sock_send, arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32);
wasi_function!(wasi_snapshot_preview1_sock_shutdown,sock_shutdown, arg0: i32, arg1: i32);

#[no_mangle]
pub extern "C" fn wasi_snapshot_preview1_proc_exit(exec_env: &ExecEnv, arg0: i32) -> () {
    eprintln!("[debug] call proc_exit{:?}", (arg0));
    let mut ctx = get_ctx_mut(exec_env).lock().unwrap();
    let mut memory = memory(exec_env);
    let res = get_runtime().block_on(preview1::proc_exit(&mut *ctx, &mut memory, arg0));
    unwrap(res)
}
