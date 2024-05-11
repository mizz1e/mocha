use {
    crate::ffi::c_str,
    nix::sys::statfs,
    std::{ffi::CStr, fmt, str},
};

/// A filesystem type.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct FsKind {
    inner: Inner,
}

/// Generate the contents of [`FsKind`].
macro_rules! impl_fs_kind {
    ($(($variant:ident, $string:literal, $magic:expr)),* $(,)?) => {
        #[derive(Clone, Copy, Eq, PartialEq)]
        enum Inner {
            $($variant,)*
        }

        impl FsKind {
            $(pub const $variant: Self = Self { inner: Inner::$variant };)*

            /// Returns a string label of this filesystem type.
            pub fn to_str(self) -> &'static str {
                unsafe { str::from_utf8_unchecked(self.to_c_str().to_bytes()) }
            }

            /// Returns a C string label of this filesystem type.
            pub fn to_c_str(self) -> &'static CStr {
                const TABLE: &[&CStr] = &[
                    $(c_str!(concat!($string, "\0")),)*
                ];

                TABLE[self.inner as usize]
            }

            /// Returns the super magic for this filesystem type.
            pub fn magic(self) -> u64 {
                const TABLE: &[u64] = &[
                    $($magic,)*
                ];

                TABLE[self.inner as usize]
            }
        }
    };
}

impl fmt::Debug for FsKind {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.to_str(), fmt)
    }
}

impl_fs_kind! {
    (BPF, "bpf", statfs::BPF_FS_MAGIC.0 as u64),
    (BTRFS, "btrfs", statfs::BTRFS_SUPER_MAGIC.0 as u64),
    (DEVPTS, "devpts", statfs::DEVPTS_SUPER_MAGIC.0 as u64),
    (F2FS, "f2fs", statfs::F2FS_SUPER_MAGIC.0 as u64),
    (FUSE, "fuse", statfs::FUSE_SUPER_MAGIC.0 as u64),
    (OVERLAYFS, "overlayfs", statfs::OVERLAYFS_SUPER_MAGIC.0 as u64),
    (PROC, "proc", statfs::PROC_SUPER_MAGIC.0 as u64),
    (SYSFS, "sysfs", statfs::SYSFS_MAGIC.0 as u64),
    (TMPFS, "tmpfs", statfs::TMPFS_MAGIC.0 as u64),
    (TRACEFS, "tracefs", statfs::TRACEFS_MAGIC.0 as u64),
}
