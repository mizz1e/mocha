use {
    rustix::{fs::OFlags, path::Arg},
    std::{
        io,
        os::fd::{AsFd, BorrowedFd, OwnedFd},
    },
};

pub use self::{kind::FsKind, resolve_options::ResolveOptions};

mod kind;
mod resolve_options;
mod wrap;

pub struct Dir {
    fd: Option<OwnedFd>,
}

pub struct OpenOptions<'a> {
    parent_dir: &'a Dir,
    resolve_options: ResolveOptions,
}

impl Dir {
    pub const ROOT: Dir = Dir { fd: None };

    pub(crate) fn as_fd(&self) -> Option<BorrowedFd<'_>> {
        self.fd.as_ref().map(AsFd::as_fd)
    }

    pub fn open_options(&self) -> OpenOptions<'_> {
        OpenOptions::new(self)
    }

    /// Flush filesystem data.
    ///
    /// If `Dir` is `ROOT`, then all filesystems are flushed.
    pub fn flush(&self) -> io::Result<()> {
        wrap::flush(self.as_fd())
    }
}

impl<'a> OpenOptions<'a> {
    pub(crate) fn new(parent_dir: &'a Dir) -> Self {
        OpenOptions {
            parent_dir,
            resolve_options: ResolveOptions::default(),
        }
    }

    /// Sets the option for restricted path resolution to the parent directory.
    pub fn restricted(&mut self, restricted: bool) -> &mut Self {
        self.resolve_options.restricted(restricted);
        self
    }

    pub fn same_file_system(&mut self, same_file_system: bool) -> &mut Self {
        self.resolve_options.same_file_system(same_file_system);
        self
    }

    pub fn open<P: Arg>(&self, path: P) -> io::Result<Dir> {
        let Self {
            parent_dir,
            resolve_options,
        } = self;

        let fd = wrap::openat2(
            parent_dir.as_fd(),
            path,
            OFlags::CLOEXEC,
            0,
            *resolve_options,
        )?;

        Ok(Dir { fd: Some(fd) })
    }
}
