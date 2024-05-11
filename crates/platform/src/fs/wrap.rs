use {
    super::ResolveOptions,
    crate::io::IoErrorExt,
    rustix::{fs::OFlags, path::Arg},
    std::{
        ffi::CStr,
        io,
        mem::{self, MaybeUninit},
        os::fd::{AsFd, BorrowedFd, FromRawFd, OwnedFd, RawFd},
        ptr,
    },
};

/// Open and possibly create a file (extended).
pub fn openat2<Fd, Path>(
    parent_fd: Option<Fd>,
    path: Path,
    oflags: OFlags,
    mode: u32,
    resolve_options: ResolveOptions,
) -> io::Result<OwnedFd>
where
    Fd: AsFd,
    Path: Arg,
{
    fn open_how(
        in_root: bool,
        oflags: OFlags,
        mode: u32,
        resolve_options: ResolveOptions,
    ) -> libc::open_how {
        let mut how = unsafe { MaybeUninit::<libc::open_how>::zeroed().assume_init() };

        how.flags = oflags.bits().into();
        how.mode = mode.into();
        how.resolve = resolve_options.to_bits(in_root);
        how
    }

    fn openat2(
        parent_fd: Option<BorrowedFd<'_>>,
        path: &CStr,
        how: &libc::open_how,
    ) -> rustix::io::Result<OwnedFd> {
        let result = unsafe {
            libc::syscall(
                libc::SYS_openat2,
                parent_fd,
                path.as_ptr(),
                ptr::from_ref(how),
                mem::size_of_val(how),
            )
        };

        if result >= 0 {
            Ok(unsafe { OwnedFd::from_raw_fd(result as RawFd) })
        } else {
            Err(io::Error::last_os_error_rustix())
        }
    }

    let parent_fd = parent_fd.as_ref().map(AsFd::as_fd);
    let how = open_how(parent_fd.is_none(), oflags, mode, resolve_options);
    let fd = path.into_with_c_str(|path| openat2(parent_fd, path, &how))?;

    Ok(fd)
}

/// Flush one or all filesystems.
pub fn flush<Fd: AsFd>(fd: Option<Fd>) -> io::Result<()> {
    fn flush(fd: Option<BorrowedFd<'_>>) -> io::Result<()> {
        match fd {
            Some(fd) => rustix::fs::syncfs(fd)?,
            None => rustix::fs::sync(),
        }

        Ok(())
    }

    flush(fd.as_ref().map(AsFd::as_fd))
}
