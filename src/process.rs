use rustix::process;

pub use std::process::id;

/// Returns the current process' group ID.
pub fn group_id() -> u32 {
    process::getegid().as_raw()
}

/// Returns the current process' user ID.
pub fn user_id() -> u32 {
    process::geteuid().as_raw()
}
