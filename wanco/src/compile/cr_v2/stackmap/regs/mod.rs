#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
pub use self::x86_64::*;

pub trait AsStr {
    fn as_str(&self) -> &str;
}
