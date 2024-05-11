use {
    rustix::mount::{self, MountFlags},
    std::{fs, io, os::unix::process::CommandExt, process::Command},
};

fn main() -> io::Result<()> {
    fs::create_dir("/dev/pts")?;
    fs::create_dir("/dev/shm")?;

    mount::mount("devpts", "/dev/pts", "devpts", MountFlags::empty(), "")?;
    mount::mount("proc", "/proc", "proc", MountFlags::empty(), "")?;
    mount::mount("sysfs", "/sys", "sysfs", MountFlags::empty(), "")?;

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name(env!("CARGO_PKG_NAME"))
        .build()?
        .block_on(run())?;

    Ok(())
}

async fn run() -> io::Result<()> {
    Command::new("/usr/bin/fish")
        .gid(1000)
        .uid(1000)
        .current_dir("/home/you")
        .spawn()?;

    Ok(())
}
