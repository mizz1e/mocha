use std::io;

pub trait IoErrorExt {
    /// Convert an [`io::Error`] to [`io::Errno`](rustix::io::Errno).
    ///
    /// # Safety
    ///
    /// Caller must ensure this [`io::Error`] is an OS error.
    unsafe fn into_rustix(self) -> rustix::io::Errno;

    /// Returns an error representing the last OS error which occurred.
    fn last_os_error_rustix() -> rustix::io::Errno;
}

impl IoErrorExt for io::Error {
    unsafe fn into_rustix(self) -> rustix::io::Errno {
        rustix::io::Errno::from_io_error(&self).unwrap_unchecked()
    }

    fn last_os_error_rustix() -> rustix::io::Errno {
        // SAFETY: [`io::Error::last_os_error`] is always an OS error.
        unsafe { io::Error::last_os_error().into_rustix() }
    }
}
