use std::io;

pub use self::error::Error;

mod error;

pub mod device;
pub mod process;

fn main() -> Error {
    if process::id() != 0 || process::user_id() != 0 || process::group_id() != 0 {
        return Error::new(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Only the kernel is permitted to execute init",
        ));
    }

    Error::unrecoverable(io::Error::new(io::ErrorKind::InvalidInput, "Oh no!"))
}
