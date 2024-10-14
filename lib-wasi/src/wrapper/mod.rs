pub mod preview1;

pub(self) fn unwrap<T, E: std::fmt::Debug>(res: Result<T, E>) -> T {
    match res {
        Ok(val) => val,
        Err(e) => panic!("WASI error: {:?}", e),
    }
}
