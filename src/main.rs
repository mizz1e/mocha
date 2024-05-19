use {
    self::fs::{FsKind, MountOptions},
    std::io::{self, BufRead, Write},
};

pub use self::error::Error;

mod error;

pub mod device;
pub mod fs;
pub mod process;

fn main() -> Error {
    if
    /*process::id() != 0 ||*/
    process::user_id() != 0 || process::group_id() != 0 {
        return Error::new(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Only the kernel is permitted to execute init",
        ));
    }

    if let Err(error) = mount() {
        return Error::unrecoverable(error);
    }

    // TODO: Perhaps check for unshare/chroot via ``/proc/1/self`, and do magic things.

    if let Err(error) = shell() {
        return Error::unrecoverable(io::Error::new(
            io::ErrorKind::Interrupted,
            format!("Shell exited: {error}"),
        ));
    }

    Error::unrecoverable(io::Error::new(io::ErrorKind::Interrupted, "Shell exited"))
}

fn mount() -> io::Result<()> {
    fs::create_dir("/dev", 0o000).or_else(fs::already_exists)?;
    MountOptions::new()
        .fs_kind(FsKind::DEVTMPFS)
        .special_devices(true)
        .mount("/dev")?;

    fs::create_dir("/dev/pts", 0o000).or_else(fs::already_exists)?;
    MountOptions::new()
        .fs_kind(FsKind::DEVPTS)
        .special_devices(true)
        .mount("/dev/pts")?;

    fs::create_dir("/dev/shm", 0o000).or_else(fs::already_exists)?;
    fs::mount("/dev/shm", FsKind::TMPFS)?;

    fs::create_dir("/proc", 0o000).or_else(fs::already_exists)?;
    MountOptions::new()
        .extra("hidepid=invisible")
        .fs_kind(FsKind::PROC)
        .mount("/proc")?;

    fs::create_dir("/sys", 0o000).or_else(fs::already_exists)?;
    fs::mount("/sys", FsKind::SYSFS)?;

    let _result = fs::mount("/sys/firmware/efi/efivars", FsKind::EFIVARFS);
    let _result = fs::mount("/sys/fs/bpf", FsKind::BPF);
    let _result = fs::mount("/sys/fs/cgroup", FsKind::CGROUP2);
    let _result = fs::mount("/sys/fs/pstore", FsKind::PSTORE);

    Ok(())
}

fn shell() -> io::Result<()> {
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();
    let mut line = String::new();

    loop {
        write!(&mut stdout, " [mocha] ")?;
        stdout.flush()?;

        stdin.read_line(&mut line)?;

        line.clear();
    }
}
