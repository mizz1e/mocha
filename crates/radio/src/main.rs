use {
    binrw::BinRead,
    pal::modem::{self, UmtsBoot},
    rustix::event::{self, PollFd, PollFlags},
    std::{
        fs::{self, File},
        io::{self, Read},
        thread,
        time::Duration,
    },
    tracing::{error, info},
};

fn main() -> io::Result<()> {
    tracing_subscriber::fmt::init();

    info!("open /dev/umts_ipc0");

    let mut umts_ipc = File::options()
        .read(true)
        .write(true)
        .open("/dev/umts_ipc0")?;

    info!("open /dev/umts_rfs0");

    let umts_rfs = File::options()
        .read(true)
        .write(true)
        .open("/dev/umts_rfs0")?;

    // FIXME: Is there a better way to implement this? (async?).
    thread::spawn(move || match boot_modem() {
        Ok(()) => unreachable!(),
        Err(error) => error!("umts boot: {error}"),
    });

    loop {
        let mut fds = [
            PollFd::new(&umts_ipc, PollFlags::IN),
            PollFd::new(&umts_rfs, PollFlags::IN),
        ];

        let index = event::poll(&mut fds, -1)?;

        match index {
            0 => {
                info!("umts ipc: ready");

                let state = modem::ioctl::ioctl_modem_status(&umts_ipc)?;

                info!("umts ipc: {state:?}");

                let mut bytes = [0u8; 7 + u16::MAX as usize];

                match umts_ipc.read(&mut bytes) {
                    Ok(len) => {
                        if len >= 7 {
                            let header =
                                modem::c::SipcFmtHdr::read(&mut io::Cursor::new(&bytes[..len]))
                                    .map_err(io::Error::other)?;

                            let body = &bytes[7..(header.len as usize)];

                            info!("umts ipc: {header:?}");
                            info!("umts ipc: {body:02X?}");
                        } else {
                            info!("umts ipc: unknown packet");
                            info!("umts ipc: {:02X?}", &bytes[..len]);
                        }
                    }
                    Err(error) => {
                        error!("umts ipc: {error}");
                    }
                }
            }
            1 => {
                info!("umts rfs: ready");

                info!("umts rfs: not implemented");
            }
            _ => {
                unreachable!()
            }
        }
    }
}

fn boot_modem() -> io::Result<()> {
    info!("open /dev/block/by-name/radio");

    let modem_firmware = fs::read("/dev/block/by-name/radio")?;

    info!("open /mnt/vendor/efs/nv_data.bin");

    // FIXME: This file is located on a mysterious ""ext4"" partition (/dev/sda3).
    // `dd`-ing the partition, then executing `file` on it, claims it is "data", yet `mount` claims it is `ext4`.
    let nv_data = fs::read("/mnt/vendor/efs/nv_data.bin")?;
    let firmware = modem::c::Firmware::read(&mut io::Cursor::new(&modem_firmware[..]))
        .map_err(io::Error::other)?;

    info!("firmware = {firmware:#?}");

    let boot = &modem_firmware[firmware.boot.range()];
    let main = &modem_firmware[firmware.main.range()];
    let vss = &modem_firmware[firmware.unspecified.range()];

    // FIXME: Implement a method for dumping device firmware blobs.
    // info!("dump boot.bin");

    // fs::write("boot.bin", boot)?;

    // info!("dump main.bin");

    // fs::write("main.bin", main)?;

    // info!("dump vss.bin");

    // fs::write("vss.bin", vss)?;

    let base_address = firmware.boot.load_address;

    info!("open /dev/umts_boot0");

    let mut umts_boot = UmtsBoot::open()?;

    info!("reset modem");

    umts_boot.reset()?;

    info!("set insecure mode");

    umts_boot.set_insecure()?;

    info!("upload boot firmware");

    umts_boot.upload_firmware(base_address, &firmware.boot, boot)?;

    info!("upload main firmware");

    umts_boot.upload_firmware(base_address, &firmware.main, main)?;

    info!("upload vss firmware");

    umts_boot.upload_firmware(base_address, &firmware.unspecified, vss)?;

    info!("upload nv firmware");

    umts_boot.upload_firmware(base_address, &firmware.nv, &nv_data)?;

    info!("set secure mode");

    umts_boot.set_secure(&firmware.boot, &firmware.main)?;

    info!("power on modem");

    umts_boot.power_on()?;

    info!("power on boot");

    umts_boot.power_boot_on()?;

    info!("download modem firmware");

    umts_boot.boot_dowload_firmware()?;

    info!("modem handshake");

    umts_boot.boot_handshake()?;

    info!("power off boot");

    umts_boot.power_boot_off()?;

    info!("umts boot: success");

    loop {
        let mut bytes = [0u8; 512];
        let len = umts_boot.as_device_mut().read(&mut bytes)?;

        if len > 0 {
            let bytes = &bytes[0..len];
            let state = modem::ioctl::ioctl_modem_status(umts_boot.as_device_mut())?;

            info!("umts boot: {state:?}");
            info!("umts boot: {bytes:02X?}");
        } else {
            thread::sleep(Duration::from_secs(1));
        }
    }
}
