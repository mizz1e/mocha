use {
    rustix::{
        mount::{self, MountFlags},
        path::Arg,
    },
    std::{
        borrow::Cow,
        ffi::{CStr, OsStr, OsString},
        fmt,
        fs::DirBuilder,
        io,
        os::unix::fs::DirBuilderExt,
        path::Path,
        str,
    },
};

macro_rules! fs_kind {
    ($(const $ident:ident = $value:literal;)*) => {
        const TABLE: &[&'static CStr] = &[$($value,)*];

        #[allow(clippy::upper_case_acronyms)]
        #[derive(Clone, Copy, Eq, PartialEq)]
        #[repr(usize)]
        enum FsKindInner {
            $($ident,)*
        }

        #[derive(Clone, Copy, Eq, PartialEq)]
        pub struct FsKind {
            inner: FsKindInner,
        }

        impl FsKind {
            $(pub const $ident: Self = Self {
                inner: FsKindInner::$ident,
            };)*

            pub const fn to_cstr(&self) -> &'static CStr {
                TABLE[self.inner as usize]
            }

            pub const fn to_str(&self) -> &str {
                unsafe { str::from_utf8_unchecked(self.to_cstr().to_bytes()) }
            }
        }
    };
}

fs_kind! {
    const BPF = c"bpf";
    const CGROUP2 = c"cgroup2";
    const CONFIGFS = c"configfs";
    const DEBUGFS = c"debugfs";
    const DEVPTS = c"devpts";
    const DEVTMPFS = c"devtmpfs";
    const EFIVARFS = c"efivars";
    const FUSECTL = c"fusectl";
    const HUGETLBFS = c"hugetlbfs";
    const MQUEUE = c"mqueue";
    const PROC = c"proc";
    const PSTORE = c"pstore";
    const SYSFS = c"sysfs";
    const TMPFS = c"tmpfs";
}

impl Arg for FsKind {
    #[inline]
    fn as_str(&self) -> rustix::io::Result<&str> {
        Ok(self.to_str())
    }

    #[inline]
    fn to_string_lossy(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.to_str())
    }

    #[inline]
    fn as_cow_c_str(&self) -> rustix::io::Result<Cow<'_, CStr>> {
        Ok(Cow::Borrowed(self.to_cstr()))
    }

    #[inline]
    fn into_c_str<'b>(self) -> rustix::io::Result<Cow<'b, CStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Borrowed(self.to_cstr()))
    }

    #[inline]
    fn into_with_c_str<T, F>(self, f: F) -> rustix::io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&CStr) -> rustix::io::Result<T>,
    {
        f(self.to_cstr())
    }
}

impl fmt::Debug for FsKind {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_tuple("FsKind").field(&self.to_str()).finish()
    }
}

const DEFAULT_FLAGS: MountFlags = MountFlags::empty()
    .union(MountFlags::NOATIME)
    .union(MountFlags::NODEV)
    .union(MountFlags::NODIRATIME)
    .union(MountFlags::NOEXEC)
    .union(MountFlags::NOSUID);

#[derive(Clone, Debug, Default)]
pub struct MountOptions {
    executable: bool,
    extra: Option<OsString>,
    fs_kind: Option<FsKind>,
    special_devices: bool,
}

impl MountOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn executable(&mut self, executable: bool) -> &mut Self {
        self.executable = executable;
        self
    }

    pub fn fs_kind(&mut self, fs_kind: FsKind) -> &mut Self {
        self.fs_kind = Some(fs_kind);
        self
    }

    pub fn special_devices(&mut self, special_devices: bool) -> &mut Self {
        self.special_devices = special_devices;
        self
    }

    pub fn extra<S: AsRef<OsStr>>(&mut self, extra: S) -> &mut Self {
        self.extra = Some(extra.as_ref().into());
        self
    }

    pub fn mount<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let target = path.as_ref();

        let fs_kind = self.fs_kind;
        let source = fs_kind;
        let mut flags = DEFAULT_FLAGS;

        flags.set(MountFlags::NOEXEC, !self.executable);
        flags.set(MountFlags::NODEV, !self.special_devices);

        rustix::path::option_into_with_c_str(self.extra.as_deref(), |extra| {
            mount::mount2(source, target, fs_kind, flags, extra)
        })
        .or_else(|error| {
            // already mounted
            if error == rustix::io::Errno::BUSY {
                Ok(())
            } else {
                Err(error)
            }
        })
        .map_err(|error| {
            let target = target.display();
            let error = io::Error::from(error);

            io::Error::new(error.kind(), format!("{target}: {error}"))
        })?;

        Ok(())
    }
}

pub fn create_dir<P: AsRef<Path>>(path: P, permissions: u32) -> io::Result<()> {
    DirBuilder::new().mode(permissions).create(path)?;

    Ok(())
}

pub fn mount<P: AsRef<Path>>(path: P, kind: FsKind) -> io::Result<()> {
    MountOptions::new().fs_kind(kind).mount(path)?;

    Ok(())
}

pub fn already_exists(error: io::Error) -> io::Result<()> {
    if error.kind() == io::ErrorKind::AlreadyExists {
        Ok(())
    } else {
        Err(error)
    }
}
