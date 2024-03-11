use {
    futures_util::ready,
    rustix::ioctl::{self, CompileTimeOpcode, Ioctl, NoArg, NoneOpcode, Opcode, Setter},
    std::{
        ffi,
        fs::File,
        io::{self, Read, Write},
        marker::PhantomData,
        path::Path,
        pin::Pin,
        ptr,
        task::{Context, Poll},
    },
    tokio::io::{unix::AsyncFd, AsyncRead, AsyncWrite, ReadBuf},
};

const MODEM_ON: u8 = 0x19;
const MODEM_OFF: u8 = 0x20;
const MODEM_RESET: u8 = 0x21;
const MODEM_BOOT_ON: u8 = 0x22;
const MODEM_BOOT_OFF: u8 = 0x23;
const MODEM_BOOT_DONE: u8 = 0x24;
const MODEM_PROTOCOL_SUSPEND: u8 = 0x25;
const MODEM_PROTOCOL_RESUME: u8 = 0x26;
const MODEM_STATUS: u8 = 0x27;
const MODEM_DL_START: u8 = 0x28;
const MODEM_FW_UPDATE: u8 = 0x29;
const MODEM_NET_SUSPEND: u8 = 0x30;
const MODEM_NET_RESUME: u8 = 0x31;
const MODEM_DUMP_START: u8 = 0x32;
const MODEM_DUMP_UPDATE: u8 = 0x33;
const MODEM_FORCE_CRASH_EXIT: u8 = 0x34;
const MODEM_CP_UPLOAD: u8 = 0x35;
const MODEM_DUMP_RESET: u8 = 0x36;
const LINK_CONNECTED: u8 = 0x33;
const MODEM_SET_TX_LINK: u8 = 0x37;
const MODEM_RAMDUMP_START: u8 = 0xCE;
const MODEM_RAMDUMP_STOP: u8 = 0xCF;
const MODEM_XMIT_BOOT: u8 = 0x40;
const MODEM_GET_SHMEM_INFO: u8 = 0x41;
const DPRAM_INIT_STATUS: u8 = 0x43;
const LINK_DEVICE_RESET: u8 = 0x44;
const MODEM_GET_SHMEM_SRINFO: u8 = 0x45;
const MODEM_SET_SHMEM_SRINFO: u8 = 0x46;
const MODEM_GET_CP_BOOTLOG: u8 = 0x47;
const MODEM_CLR_CP_BOOTLOG: u8 = 0x48;
const MIF_LOG_DUMP: u8 = 0x51;
const MIF_DPRAM_DUMP: u8 = 0x52;
const SECURITY_REQ: u8 = 0x53;
const SHMEM_FULL_DUMP: u8 = 0x54;
const MODEM_CRASH_REASON: u8 = 0x55;
const MODEM_AIRPLANE_MODE: u8 = 0x56;
const VSS_FULL_DUMP: u8 = 0x57;
const ACPM_FULL_DUMP: u8 = 0x58;
const CPLOG_FULL_DUMP: u8 = 0x59;
const DATABUF_FULL_DUMP: u8 = 0x5A;
const REGISTER_PCIE: u8 = 0x65;

/// `_IO('o', number)` - Provides a modem opcode at compile-time.
pub struct ModemOpcode<const NUM: u8>;

impl<const NUM: u8> CompileTimeOpcode for ModemOpcode<NUM> {
    const OPCODE: Opcode = NoneOpcode::<b'o', NUM, ()>::OPCODE;
}

/// A device file.
#[derive(Debug)]
pub struct Device {
    device: AsyncFd<File>,
}

impl Device {
    /// Open the device at `path`.
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        File::options()
            .read(true)
            .write(true)
            .open(path)
            .and_then(AsyncFd::new)
            .map(|device| Self { device })
    }

    /// Begin the boot process.
    ///
    /// Corresponds to `ioctl(device, IOCTL_MODEM_BOOT_ON, ...)`.
    #[doc(alias = "IOCTL_MODEM_BOOT_ON")]
    pub fn boot_start(&mut self) -> io::Result<()> {
        unsafe {
            ioctl::ioctl(
                &mut self.device,
                NoArg::<ModemOpcode<{ MODEM_BOOT_ON }>>::new(),
            )
            .map_err(io::Error::from)
        }
    }

    /// Download blobs onto the modem.
    ///
    /// Corresponds to `ioctl(device, IOCTL_MODEM_DL_START, ...)`.
    #[doc(alias = "IOCTL_MODEM_DL_START")]
    pub fn boot_download_blobs(&mut self) -> io::Result<()> {
        unsafe {
            ioctl::ioctl(
                &mut self.device,
                NoArg::<ModemOpcode<{ MODEM_DL_START }>>::new(),
            )
            .map_err(io::Error::from)
        }
    }

    /// Finalize the boot process.
    ///
    /// Corresponds to `ioctl(device, IOCTL_MODEM_BOOT_OFF, ...)`.
    #[doc(alias = "IOCTL_MODEM_BOOT_OFF")]
    pub fn boot_finish(&mut self) -> io::Result<()> {
        unsafe {
            ioctl::ioctl(
                &mut self.device,
                NoArg::<ModemOpcode<{ MODEM_BOOT_OFF }>>::new(),
            )
            .map_err(io::Error::from)
        }
    }

    /// Begin a firmware update.
    ///
    /// Corresponds to `ioctl(device, IOCTL_MODEM_FW_UPDATE, ...)`.
    #[doc(alias = "IOCTL_MODEM_FW_UPDATE")]
    pub fn firmware_update(&mut self, size: u32, mtu: u32, num_frames: u32) -> io::Result<()> {
        let update = FirmwareUpdate {
            size,
            mtu,
            num_frames,
        };

        unsafe {
            let control = Setter::<ModemOpcode<{ MODEM_FW_UPDATE }>, FirmwareUpdate>::new(update);

            ioctl::ioctl(&mut self.device, control).map_err(io::Error::from)
        }
    }

    /// Power off the modem.
    ///
    /// Correponds to `ioctl(device, IOCTL_MODEM_OFF, ...)`.
    #[doc(alias = "IOCTL_MODEM_OFF")]
    pub fn power_off(&mut self) -> io::Result<()> {
        unsafe {
            ioctl::ioctl(&mut self.device, NoArg::<ModemOpcode<{ MODEM_OFF }>>::new())
                .map_err(io::Error::from)
        }
    }

    /// Power on the modem.
    ///
    /// Correponds to `ioctl(device, IOCTL_MODEM_ON, ...)`.
    #[doc(alias = "IOCTL_MODEM_ON")]
    pub fn power_on(&mut self) -> io::Result<()> {
        unsafe {
            ioctl::ioctl(&mut self.device, NoArg::<ModemOpcode<{ MODEM_ON }>>::new())
                .map_err(io::Error::from)
        }
    }

    /// Register a PCIE link to the modem.
    ///
    /// Correponds to `ioctl(device, IOCTL_REGISTER_PCIE, ...)`.
    #[doc(alias = "IOCTL_REGISTER_PCIE")]
    pub fn register_pcie_link(&mut self) -> io::Result<()> {
        unsafe {
            ioctl::ioctl(
                &mut self.device,
                NoArg::<ModemOpcode<{ REGISTER_PCIE }>>::new(),
            )
            .map_err(io::Error::from)
        }
    }

    /// Reset the modem.
    ///
    /// Corresponds to `ioctl(device, IOCTL_MODEM_RESET, ...)`.
    #[doc(alias = "IOCTL_MODEM_RESET")]
    pub fn reset(&mut self) -> io::Result<()> {
        unsafe {
            ioctl::ioctl(
                &mut self.device,
                NoArg::<ModemOpcode<{ MODEM_RESET }>>::new(),
            )
            .map_err(io::Error::from)
        }
    }

    /// Perform a security request.
    ///
    /// Corresponds to `ioctl(device, IOCTL_SECURITY_REQ, ...)`.
    #[doc(alias = "IOCTL_SECURITY_REQ")]
    pub fn security_request(&mut self, request: Option<(u32, u32)>) -> io::Result<()> {
        let request = request
            .map(SecurityRequest::secure)
            .unwrap_or_else(SecurityRequest::insecure);

        unsafe {
            let control = Setter::<ModemOpcode<{ SECURITY_REQ }>, SecurityRequest>::new(request);

            ioctl::ioctl(&mut self.device, control).map_err(io::Error::from)
        }
    }

    /// Returns the current state of the modem.
    ///
    /// Corresponds to `ioctl(device, IOCTL_MODEM_STATUS, ...)`.
    #[doc(alias = "IOCTL_MODEM_STATUS")]
    pub fn state(&mut self) -> io::Result<State> {
        unsafe { ioctl::ioctl(&mut self.device, GetState).map_err(io::Error::from) }
    }

    /// Upload a firmware blob chunk.
    ///
    /// Corresponds to `ioctl(device, IOCTL_MODEM_XMIT_BOOT, ...)`.
    #[doc(alias = "IOCTL_XMIT_BOOT")]
    pub fn upload_blob_chunk(
        &mut self,
        blob: &[u8],
        total_len: u32,
        address: u32,
        offset: u32,
    ) -> io::Result<()> {
        let chunk = BlobChunk::new(blob, total_len, address, offset);

        unsafe {
            let control = Setter::<ModemOpcode<{ MODEM_XMIT_BOOT }>, BlobChunk<'_>>::new(chunk);

            ioctl::ioctl(&mut self.device, control).map_err(io::Error::from)
        }
    }

    /// Verify something.
    pub async fn verify(&mut self, request: Option<[u8; 4]>, response: [u8; 4]) -> io::Result<()> {
        if let Some(request) = request {
            let _amount = self.write(&request)?;
        }

        let mut guard = self.device.readable_mut().await?;
        let mut buf = [0; 4];

        let _amount = guard.get_inner_mut().read(&mut buf)?;

        if buf == response {
            Ok(())
        } else {
            Err(io::Error::other("unexpected verification response"))
        }
    }
}

impl Read for Device {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.device.get_mut().read(buf)
    }
}

impl Write for Device {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.device.get_mut().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        // No-op for device files.

        Ok(())
    }
}

impl AsyncRead for Device {
    fn poll_read(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        loop {
            let mut guard = ready!(self.device.poll_read_ready_mut(context))?;
            let unfilled = buf.initialize_unfilled();

            match guard.try_io(|inner| inner.get_mut().read(unfilled)) {
                Ok(Ok(0)) => {
                    return Poll::Pending;
                }
                Ok(Ok(len)) => {
                    buf.advance(len);

                    return Poll::Ready(Ok(()));
                }
                Ok(Err(err)) => return Poll::Ready(Err(err)),
                Err(_would_block) => continue,
            }
        }
    }
}

impl AsyncWrite for Device {
    fn poll_write(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        loop {
            let mut guard = ready!(self.device.poll_write_ready_mut(context))?;

            match guard.try_io(|inner| inner.get_mut().write(buf)) {
                Ok(result) => return Poll::Ready(result),
                Err(_would_block) => continue,
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        // No-op for device files.

        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        _context: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        // No-op for device files.

        Poll::Ready(Ok(()))
    }
}

/// Ioctl type for `IOCTL_MODEM_STATUS`.
struct GetState;

unsafe impl Ioctl for GetState {
    type Output = State;

    const IS_MUTATING: bool = false;
    const OPCODE: Opcode = ModemOpcode::<{ MODEM_STATUS }>::OPCODE;

    fn as_ptr(&mut self) -> *mut ffi::c_void {
        ptr::null_mut()
    }

    unsafe fn output_from_ptr(
        ret: ioctl::IoctlOutput,
        _arg: *mut ffi::c_void,
    ) -> rustix::io::Result<Self::Output> {
        State::try_from_raw(ret as u32)
    }
}

/// Structure used in `IOCTL_MODEM_XMIT_BOOT`.
#[derive(Debug, Default)]
#[repr(C, packed)]
pub struct BlobChunk<'blob> {
    /// Pointer to the blob chunk.
    blob_ptr: u64,
    /// Total length of the blob.
    total_len: u32,
    /// Address this blob chunk is loaded to.
    address: u32,
    /// File ofrset this blob chunk originated from.
    offset: u32,
    /// Unknown, always `0`.
    mode: u32,
    /// Length (in bytes) of this blob chunk.
    len: u32,
    /// `blob_ptr` and `len` is valid for the lifetime of the byte slice.
    _blob: PhantomData<&'blob [u8]>,
}

impl<'blob> BlobChunk<'blob> {
    pub const BLOB_MAX_LEN: usize = 62 * 1024;

    pub fn new(blob: &'blob [u8], total_len: u32, address: u32, offset: u32) -> Self {
        assert!(blob.len() <= Self::BLOB_MAX_LEN);

        Self {
            blob_ptr: blob.as_ptr() as u64,
            total_len,
            address,
            offset,
            mode: 0,
            len: blob.len() as u32,
            _blob: PhantomData,
        }
    }
}

/// Structure used in `IOCTL_MODEM_FW_UPDATE`.
#[derive(Debug, Default)]
#[repr(C, packed)]
pub struct FirmwareUpdate {
    pub size: u32,
    /// Maximum Transmission Unit (MTU).
    pub mtu: u32,
    pub num_frames: u32,
}

/// Current state of the modem.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C)]
pub enum State {
    #[default]
    Offline = 0,
    CrashReset = 1,
    CrashExit = 2,
    Booting = 3,
    Online = 4,
    NvRebuilding = 5,
    LoaderDone = 6,
    SimAttach = 7,
    SimDetach = 8,
    CrashWatchdog = 9,
}

impl State {
    fn try_from_raw(state: u32) -> rustix::io::Result<Self> {
        let state = match state {
            0 => Self::Offline,
            1 => Self::CrashReset,
            2 => Self::CrashExit,
            3 => Self::Booting,
            4 => Self::Online,
            5 => Self::NvRebuilding,
            6 => Self::LoaderDone,
            7 => Self::SimAttach,
            8 => Self::SimDetach,
            9 => Self::CrashWatchdog,
            _ => return Err(rustix::io::Errno::INVAL),
        };

        Ok(state)
    }
}

/// Security modes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u32)]
pub enum SecurityMode {
    #[default]
    Secure = 0,
    Insecure = 2,
}

/// Structure used in `IOCTL_SECURITY_REQ`.
#[derive(Debug, Default)]
#[repr(C, packed)]
pub struct SecurityRequest {
    /// The security mode.
    mode: SecurityMode,
    /// Length (in bytes) of the `BOOT` blob, required for secure mode.
    boot_len: u32,
    /// Length (in bytes) of the `MAIN` blob, required for secure mode.
    main_len: u32,
    /// Doesn't appear to be used.
    unknown: u32,
}

impl SecurityRequest {
    /// Secure mode, requires the length of `BOOT`, and `MAIN`, blobs.
    pub fn secure((boot_len, main_len): (u32, u32)) -> Self {
        Self {
            mode: SecurityMode::Secure,
            boot_len,
            main_len,
            ..Self::default()
        }
    }

    /// Insecure mode.
    pub fn insecure() -> Self {
        Self {
            mode: SecurityMode::Insecure,
            ..Self::default()
        }
    }
}
