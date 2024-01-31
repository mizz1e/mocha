use {
    super::c,
    rustix::{
        io,
        ioctl::{self, CompileTimeOpcode, Ioctl, NoArg, NoneOpcode, Opcode, Setter},
    },
    std::{ffi, os::fd::AsFd, ptr},
};

/// `_IO('o', number)` - Provides a modem opcode at compile-time.
pub struct ModemOpcode<const NUM: u8>;

impl<const NUM: u8> CompileTimeOpcode for ModemOpcode<NUM> {
    const OPCODE: Opcode = NoneOpcode::<b'o', NUM, ()>::OPCODE;
}

/// `ioctl(fd, IOCTL_MODEM_RESET)`
#[inline]
pub fn ioctl_modem_reset<F: AsFd>(fd: F) -> io::Result<()> {
    unsafe { ioctl::ioctl(fd, NoArg::<ModemOpcode<{ c::MODEM_RESET }>>::new()) }
}

/// `ioctl(fd, IOCTL_SECURITY_REQ, request)`
#[inline]
pub fn ioctl_security_req<F: AsFd>(fd: F, request: c::ModemSecReq) -> io::Result<()> {
    unsafe {
        let control = Setter::<ModemOpcode<{ c::SECURITY_REQ }>, c::ModemSecReq>::new(request);

        ioctl::ioctl(fd, control)
    }
}

/// `ioctl(fd, IOCTL_MODEM_XMIT_BOOT, firmware)`
#[inline]
pub fn ioctl_modem_xmit_boot<F: AsFd>(fd: F, firmware: c::ModemFirmware) -> io::Result<()> {
    unsafe {
        let control =
            Setter::<ModemOpcode<{ c::MODEM_XMIT_BOOT }>, c::ModemFirmware>::new(firmware);

        ioctl::ioctl(fd, control)
    }
}

/// `ioctl(fd, IOCTL_MODEM_ON)`
#[inline]
pub fn ioctl_modem_on<F: AsFd>(fd: F) -> io::Result<()> {
    unsafe { ioctl::ioctl(fd, NoArg::<ModemOpcode<{ c::MODEM_ON }>>::new()) }
}

/// `ioctl(fd, IOCTL_MODEM_OFF)`
#[inline]
pub fn ioctl_modem_off<F: AsFd>(fd: F) -> io::Result<()> {
    unsafe { ioctl::ioctl(fd, NoArg::<ModemOpcode<{ c::MODEM_OFF }>>::new()) }
}

/// `ioctl(fd, IOCTL_MODEM_BOOT_ON)`
#[inline]
pub fn ioctl_modem_boot_on<F: AsFd>(fd: F) -> io::Result<()> {
    unsafe { ioctl::ioctl(fd, NoArg::<ModemOpcode<{ c::MODEM_BOOT_ON }>>::new()) }
}

/// `ioctl(fd, IOCTL_MODEM_BOOT_OFF)`
#[inline]
pub fn ioctl_modem_boot_off<F: AsFd>(fd: F) -> io::Result<()> {
    unsafe { ioctl::ioctl(fd, NoArg::<ModemOpcode<{ c::MODEM_BOOT_OFF }>>::new()) }
}

/// `ioctl(fd, IOCTL_MODEM_DL_START)`
#[inline]
pub fn ioctl_modem_dl_start<F: AsFd>(fd: F) -> io::Result<()> {
    unsafe { ioctl::ioctl(fd, NoArg::<ModemOpcode<{ c::MODEM_DL_START }>>::new()) }
}

/// `ioctl(fd, IOCTL_MODEM_STATUS)`
#[inline]
pub fn ioctl_modem_status<F: AsFd>(fd: F) -> io::Result<c::ModemState> {
    unsafe { ioctl::ioctl(fd, ModemStatus) }
}

struct ModemStatus;

unsafe impl Ioctl for ModemStatus {
    type Output = c::ModemState;

    const IS_MUTATING: bool = false;
    const OPCODE: Opcode = ModemOpcode::<{ c::MODEM_STATUS }>::OPCODE;

    fn as_ptr(&mut self) -> *mut ffi::c_void {
        ptr::null_mut()
    }

    unsafe fn output_from_ptr(
        ret: ioctl::IoctlOutput,
        _arg: *mut ffi::c_void,
    ) -> io::Result<Self::Output> {
        c::ModemState::try_from(ret as u32)
    }
}
