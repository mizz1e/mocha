use {
    binrw::BinRead,
    pal::modem::{self, UmtsBoot},
    std::{fs, io, thread},
    tracing::{error, info},
};

/// Network names
///
/// 2G GSM (Global System for Mobile Communications)
/// 3G UMTS (Universal Mobile Telecommunications System)
/// 4G LTE (Long-Term Evolution)
/// 5G NR (New Radio)
///
mod ril {
    use {
        std::{
            fs::File,
            io::{self, Read},
            path::Path,
        },
        tokio::io::{
            unix::{AsyncFd, AsyncFdReadyMutGuard},
            Interest,
        },
        tracing::info,
    };

    const BOOT_MAX_LEN: usize = 512;

    const IPC_HEADER_LEN: usize = 7;
    const IPC_BODY_LEN: usize = u8::MAX as usize;
    const IPC_MAX_LEN: usize = IPC_HEADER_LEN + IPC_BODY_LEN;

    enum Which<'a> {
        Boot(AsyncFdReadyMutGuard<'a, File>),
        Ipc(AsyncFdReadyMutGuard<'a, File>),
        Ipc5g(AsyncFdReadyMutGuard<'a, File>),
        Rfs(AsyncFdReadyMutGuard<'a, File>),
    }

    pub struct Ril {
        boot: AsyncFd<File>,
        ipc: AsyncFd<File>,
        ipc_5g: AsyncFd<File>,
        rfs: AsyncFd<File>,
    }

    impl Ril {
        pub fn open() -> io::Result<Self> {
            Ok(Self {
                boot: open_device("/dev/umts_boot0")?,
                ipc: open_device("/dev/umts_ipc0")?,
                ipc_5g: open_device("/dev/umts_ipc1")?,
                rfs: open_device("/dev/umts_rfs0")?,
            })
        }

        pub async fn next_event(&mut self) -> io::Result<()> {
            while let Ok(which) = tokio::select! {
                result = self.boot.ready_mut(Interest::READABLE) => result.map(Which::Boot),
                result = self.ipc.ready_mut(Interest::READABLE) => result.map(Which::Ipc),
                result = self.ipc_5g.ready_mut(Interest::READABLE) => result.map(Which::Ipc5g),
                result = self.rfs.ready_mut(Interest::READABLE) => result.map(Which::Rfs),
            } {
                match which {
                    Which::Boot(mut guard) => {
                        let mut buf = [0; BOOT_MAX_LEN];
                        let amount = guard.get_inner_mut().read(&mut buf)?;
                        let buf = &buf[..amount];

                        info!("UMTS boot message: {buf:02X?}");
                    }
                    Which::Ipc(mut guard) => {
                        let mut buf = [0; IPC_MAX_LEN];
                        let amount = guard.get_inner_mut().read(&mut buf)?;
                        let buf = &buf[..amount];

                        info!("UMTS IPC message: {buf:02X?}");
                    }
                    Which::Ipc5g(mut guard) => {
                        let mut buf = [0; IPC_MAX_LEN];
                        let amount = guard.get_inner_mut().read(&mut buf)?;
                        let _ipc_5g_buf = &buf[..amount];

                        info!("UMTS IPC 5G message: {buf:02X?}");
                    }
                    Which::Rfs(mut guard) => {
                        let mut buf = [0; IPC_MAX_LEN];
                        let amount = guard.get_inner_mut().read(&mut buf)?;
                        let _rfs_buf = &buf[..amount];

                        info!("UMTS RFS message: {buf:02X?}");
                    }
                }
            }

            Ok(())
        }
    }

    fn open_device<P: AsRef<Path>>(path: P) -> io::Result<AsyncFd<File>> {
        let path = path.as_ref();

        info!("Open device {}", path.display());

        File::options()
            .read(true)
            .write(true)
            .open(path)
            .and_then(AsyncFd::new)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt::init();

    // FIXME: Is there a better way to implement this? (async?).
    boot_modem()?;

    let mut ril = ril::Ril::open()?;

    while ril.next_event().await.is_ok() {}

    Ok(())
}

/*use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Firmware {
    pub radio: PathBuf,
    pub radio_5g: PathBuf,
    pub nv_partition: PathBuf,
    pub nv_5g_partition: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub firmware: Firmware,
}*/

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

    Ok(())
}
