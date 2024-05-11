use {
    device::Device,
    firmware::{Label, Toc},
    serde::{Deserialize, Serialize},
    std::{
        fs::File,
        io::{self, Read, Seek, Write},
        path::{Path, PathBuf},
    },
    tracing::trace,
};

pub mod device;
pub mod firmware;
pub mod ipc;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Firmware {
    pub toc: Vec<PathBuf>,
    #[serde(default)]
    pub toc_5g: Vec<PathBuf>,
    pub nv: Vec<PathBuf>,
    #[serde(default)]
    pub nv_5g: Vec<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Settings {
    pub firmware: Firmware,
}

impl Firmware {
    pub fn toc(&self) -> Option<(File, Vec<Toc>)> {
        self.toc.iter().flat_map(open_toc).next()
    }

    pub fn toc_5g(&self) -> Option<(File, Vec<Toc>)> {
        self.toc_5g.iter().flat_map(open_toc).next()
    }

    pub fn nv(&self) -> Option<File> {
        self.nv.iter().flat_map(File::open).next()
    }

    pub fn nv_5g(&self) -> Option<File> {
        self.nv_5g.iter().flat_map(File::open).next()
    }
}

pub fn open_toc<P: AsRef<Path>>(path: P) -> io::Result<(File, Vec<Toc>)> {
    let mut file = File::open(path)?;
    let toc = firmware::read(&mut file)?;

    Ok((file, toc))
}

fn no_firmware() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, "no firmware provided")
}

const UMTS_HANDSHAKE_1_REQ: [u8; 4] = [13, 144, 0, 0];
const UMTS_HANDSHAKE_1_RES: [u8; 4] = [13, 160, 0, 0];
const UMTS_HANDSHAKE_2_REQ: [u8; 4] = [0, 159, 0, 0];
const UMTS_HANDAHAKE_2_RES: [u8; 4] = [0, 175, 0, 0];

pub struct Boot {
    device: Device,
}

impl Boot {
    pub fn open() -> io::Result<Self> {
        Device::open("/dev/umts_boot0").map(|device| Self { device })
    }

    pub async fn upload_firmware(&mut self, firmware: &Firmware) -> io::Result<()> {
        let (mut toc_reader, tocs) = firmware.toc().ok_or_else(no_firmware)?;
        let mut nv = firmware.nv().ok_or_else(no_firmware)?;
        let base_address = tocs[0].address;

        trace!("Start boot.");

        self.device.reset()?;
        self.device.security_request(None)?;

        for toc in &tocs {
            trace!("Upload: {}", toc.label);
            trace!("  File offset: {}", toc.offset);
            trace!("  Load address: 0x{:08X}", toc.address);
            trace!("  Size (in bytes): {}", toc.len);
            trace!("  CRC: {}", toc.crc);
            trace!("  Entry ID: {}", toc.misc);

            let mut upload = Upload::new(self, toc, base_address);

            if toc.label == Label::Nv {
                io::copy(&mut nv, &mut upload)?;

                continue;
            }

            toc_reader.seek(io::SeekFrom::Start(toc.offset as u64))?;
            io::copy(&mut (&mut toc_reader).take(toc.len as u64), &mut upload)?;
        }

        let device = &mut self.device;

        device.security_request(Some((tocs[0].len, tocs[1].len)))?;
        device.power_on()?;
        device.boot_start()?;
        device.boot_download_blobs()?;

        device
            .verify(Some(UMTS_HANDSHAKE_1_REQ), UMTS_HANDSHAKE_1_RES)
            .await?;

        device
            .verify(Some(UMTS_HANDSHAKE_2_REQ), UMTS_HANDSHAKE_2_REQ)
            .await?;

        device.boot_finish()?;

        trace!("Boot complete.");

        Ok(())
    }
}

pub struct Upload<'boot, 'toc> {
    boot: &'boot mut Boot,
    toc: &'toc firmware::Toc,
    address: u32,
    offset: u32,
}

impl<'boot, 'toc> Upload<'boot, 'toc> {
    pub fn new(boot: &'boot mut Boot, toc: &'toc Toc, base_address: u32) -> Self {
        Self {
            boot,
            toc,
            address: toc.address.saturating_sub(base_address),
            offset: toc.offset,
        }
    }
}

impl<'boot, 'toc> Write for Upload<'boot, 'toc> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let blob = &buf[..buf.len().min(device::BlobChunk::BLOB_MAX_LEN)];

        self.boot
            .device
            .upload_blob_chunk(blob, self.toc.len, self.address, self.offset)?;

        let len: u32 = blob.len().try_into().map_err(io::Error::other)?;

        self.address += len;
        self.offset += len;

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        // No-op for uploading blobs.

        Ok(())
    }
}

const NR_HANDSHAKE_1_REQ: [u8; 4] = [0, 145, 0, 0];
const NR_HANDSHAKE_1_RES: [u8; 4] = [0, 161, 0, 0];

pub struct Boot5G {
    spi: Device,
    device: Device,
}

impl Boot5G {
    pub fn open() -> io::Result<Self> {
        let device = Device::open("/dev/nr_boot0")?;
        let spi = Device::open("/dev/modem_boot_spi")?;

        Ok(Self { spi, device })
    }

    pub async fn upload_firmware(&mut self, firmware: &Firmware) -> io::Result<()> {
        let (_toc_reader, tocs) = firmware.toc().ok_or_else(no_firmware)?;
        let _nv = firmware.nv().ok_or_else(no_firmware)?;
        let _base_address = tocs[0].address;

        trace!("Start boot.");

        let Self { spi: _, device } = self;

        device.power_on()?;
        device.boot_start()?;
        device.boot_download_blobs()?;

        //self.spi.upload_blob_chunk(...)?;
        device.register_pcie_link()?;

        device
            .verify(Some(NR_HANDSHAKE_1_REQ), NR_HANDSHAKE_1_RES)
            .await?;

        for toc in &tocs {
            trace!("Upload: {}", toc.label);
            trace!("  File offset: {}", toc.offset);
            trace!("  Load address: 0x{:08X}", toc.address);
            trace!("  Size (in bytes): {}", toc.len);
            trace!("  CRC: {}", toc.crc);
            trace!("  Entry ID: {}", toc.misc);

            //device.firmware_update(, , )?;
            /*let mut upload = Upload::new(self, toc, base_address);

            if toc.label == Label::Nv {
                io::copy(&mut nv, &mut upload)?;

                continue;
            }

            toc_reader.seek(io::SeekFrom::Start(toc.offset as u64))?;
            io::copy(&mut (&mut toc_reader).take(toc.len as u64), &mut upload)?;*/
        }

        let device = &mut self.device;

        device.security_request(Some((tocs[0].len, tocs[1].len)))?;
        device.power_on()?;
        device.boot_start()?;
        device.boot_download_blobs()?;

        device.boot_finish()?;

        trace!("Boot complete.");

        Ok(())
    }
}

/*
pub struct Upload5g<'boot, 'toc> {
    boot: &'boot mut Boot5g,
    toc: &'toc firmware::Toc,
    address: u32,
    offset: u32,
    alternate: bool,
}

impl<'boot, 'toc> Upload5g<'boot, 'toc> {
    pub fn new(boot: &'boot mut Boot5g, toc: &'toc Toc, base_address: u32) -> Self {
        Self {
            boot,
            toc,
            address: toc.address.saturating_sub(base_address),
            offset: toc.offset,
            alternate: false,
        }
    }
}

impl<'boot, 'toc> Write for Upload5g<'boot, 'toc> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let alternate = !self.alternate;
        let max_len = if mem::replace(&mut self.alternate, alternate) {
            2048
        } else {
            2064
        };

        let buf = &buf[..buf.len().min(max_len)];

        self.boot.device.get_mut().write(buf)?;

        let len: u32 = buf.len().try_into().map_err(io::Error::other)?;

        self.address += len;
        self.offset += len;

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        // No-op for uploading blobs.

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum State {
    Off,
    Booting,
    On,
}

enum Inner {
    Off,
    Booting { task: task::JoinHandle<()> },
    On { device: AsyncFd<File> },
}

pub struct Power {
    inner: Inner,
}

impl Power {
    pub fn state(&self) -> State {
        match &self.inner {
            Inner::Off => State::Off,
            Inner::Booting { .. } => State::Booting,
            Inner::On { .. } => State::On,
        }
    }

    pub fn power_on(&mut self) -> io::Result<()> {
        if matches!(&mut self.inner, Inner::Booting { .. } | Inner::On { .. }) {
            return Ok(());
        }

        let task = task::spawn_blocking(|| {});

        self.inner = Inner::Booting { task };

        Ok(())
    }

    pub fn power_off(&mut self) -> io::Result<()> {
        match &mut self.inner {
            Inner::Booting { task } => task.abort(),
            Inner::On { device } => ioctl::power_off(device)?,
            _ => {}
        };

        self.inner = Inner::Off;

        Ok(())
    }
}
*/
