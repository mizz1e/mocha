use {
    super::c,
    rustix::{
        io,
        ioctl::{self, CompileTimeOpcode, Ioctl, IoctlOutput, NoArg, NoneOpcode, Opcode},
    },
    std::{
        ffi,
        os::fd::{AsFd, AsRawFd, BorrowedFd},
        ptr,
    },
};

/// `_IO('L', number)` - Provides a loop opcode at compile-time.
pub struct LoopOpcode<const NUM: u8>;

impl<const NUM: u8> CompileTimeOpcode for LoopOpcode<NUM> {
    const OPCODE: Opcode = NoneOpcode::<b'L', NUM, ()>::OPCODE;
}

/// `ioctl(fd, LOOP_SET_FD, &fd)` - Set the file descriptor of the loop device.
///
/// # References
/// - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man4/loop.4.html
#[doc(alias = "LOOP_SET_FD")]
#[inline]
pub fn ioctl_loop_set_fd<F: AsFd, B: AsFd>(fd: F, backing_fd: B) -> io::Result<()> {
    unsafe { ioctl::ioctl(fd, LoopSetFd(backing_fd.as_fd())) }
}

struct LoopSetFd<'a>(BorrowedFd<'a>);

unsafe impl<'a> Ioctl for LoopSetFd<'a> {
    type Output = ();

    const IS_MUTATING: bool = true;
    const OPCODE: Opcode = LoopOpcode::<{ c::LOOP_SET_FD }>::OPCODE;

    fn as_ptr(&mut self) -> *mut ffi::c_void {
        self.0.as_raw_fd() as *mut ffi::c_void
    }

    unsafe fn output_from_ptr(
        _ret: IoctlOutput,
        _arg: *mut ffi::c_void,
    ) -> io::Result<Self::Output> {
        Ok(())
    }
}

/// `ioctl(fd, LOOP_CLR_FD)` - Clear the file descriptor of the loop device.
///
/// # References
/// - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man4/loop.4.html
#[doc(alias = "LOOP_CLR_FD")]
#[inline]
pub fn ioctl_loop_clr_fd<F: AsFd>(fd: F) -> io::Result<()> {
    unsafe { ioctl::ioctl(fd, NoArg::<LoopOpcode<{ c::LOOP_CLR_FD }>>::new()) }
}

/// `ioctl(fd, LOOP_SET_STATUS, &status)` - Set the status of the loop device.
///
/// # References
/// - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man4/loop.4.html
#[doc(alias = "LOOP_SET_STATUS")]
#[inline]
pub fn ioctl_loop_set_status<F: AsFd>(fd: F, status: u32) -> io::Result<()> {
    unsafe { ioctl::ioctl(fd, LoopSetStatus(status)) }
}

struct LoopSetStatus(u32);

unsafe impl Ioctl for LoopSetStatus {
    type Output = ();

    const IS_MUTATING: bool = true;
    const OPCODE: Opcode = LoopOpcode::<{ c::LOOP_SET_STATUS }>::OPCODE;

    fn as_ptr(&mut self) -> *mut ffi::c_void {
        self.0 as *mut ffi::c_void
    }

    unsafe fn output_from_ptr(
        _ret: IoctlOutput,
        _arg: *mut ffi::c_void,
    ) -> io::Result<Self::Output> {
        Ok(())
    }
}

/// `ioctl(fd, LOOP_GET_STATUS, &status)` - Get the status of the loop device.
///
/// # References
/// - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man4/loop.4.html
#[doc(alias = "LOOP_GET_STATUS")]
#[inline]
pub fn ioctl_loop_get_status<F: AsFd>(fd: F) -> io::Result<u32> {
    unsafe { ioctl::ioctl(fd, LoopGetStatus) }
}

struct LoopGetStatus;

unsafe impl Ioctl for LoopGetStatus {
    type Output = u32;

    const IS_MUTATING: bool = false;
    const OPCODE: Opcode = LoopOpcode::<{ c::LOOP_GET_STATUS }>::OPCODE;

    fn as_ptr(&mut self) -> *mut ffi::c_void {
        ptr::null_mut()
    }

    unsafe fn output_from_ptr(
        ret: IoctlOutput,
        _arg: *mut ffi::c_void,
    ) -> io::Result<Self::Output> {
        Ok(ret as u32)
    }
}

/// `ioctl(fd, LOOP_SET_STATUS64, &status)` - Set the 64-bit status of the loop device.
///
/// # References
/// - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man4/loop.4.html
#[doc(alias = "LOOP_SET_STATUS64")]
#[inline]
pub fn ioctl_loop_set_status64<F: AsFd>(fd: F, status: u64) -> io::Result<()> {
    unsafe { ioctl::ioctl(fd, LoopSetStatus64(status)) }
}

struct LoopSetStatus64(u64);

unsafe impl Ioctl for LoopSetStatus64 {
    type Output = ();

    const IS_MUTATING: bool = true;
    const OPCODE: Opcode = LoopOpcode::<{ c::LOOP_SET_STATUS64 }>::OPCODE;

    fn as_ptr(&mut self) -> *mut ffi::c_void {
        self.0 as *mut ffi::c_void
    }

    unsafe fn output_from_ptr(
        _ret: IoctlOutput,
        _arg: *mut ffi::c_void,
    ) -> io::Result<Self::Output> {
        Ok(())
    }
}

/// `ioctl(fd, LOOP_GET_STATUS64, &status)` - Get the 64-bit status of the loop device.
///
/// # References
/// - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man4/loop.4.html
#[doc(alias = "LOOP_GET_STATUS64")]
#[inline]
pub fn ioctl_loop_get_status64<F: AsFd>(fd: F) -> io::Result<u64> {
    unsafe { ioctl::ioctl(fd, LoopGetStatus64) }
}

struct LoopGetStatus64;

unsafe impl Ioctl for LoopGetStatus64 {
    type Output = u64;

    const IS_MUTATING: bool = false;
    const OPCODE: Opcode = LoopOpcode::<{ c::LOOP_GET_STATUS64 }>::OPCODE;

    fn as_ptr(&mut self) -> *mut ffi::c_void {
        ptr::null_mut()
    }

    unsafe fn output_from_ptr(
        ret: IoctlOutput,
        _arg: *mut ffi::c_void,
    ) -> io::Result<Self::Output> {
        Ok(ret as u64)
    }
}

/// `ioctl(fd, LOOP_CHANGE_FD, &fd)` - Change the file descriptor of the loop device.
///
/// # References
/// - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man4/loop.4.html
#[doc(alias = "LOOP_CHANGE_FD")]
#[inline]
pub fn ioctl_loop_change_fd<F: AsFd, B: AsFd>(fd: F, backing_fd: B) -> io::Result<()> {
    unsafe { ioctl::ioctl(fd, LoopChangeFd(backing_fd.as_fd())) }
}

struct LoopChangeFd<'a>(BorrowedFd<'a>);

unsafe impl<'a> Ioctl for LoopChangeFd<'a> {
    type Output = ();

    const IS_MUTATING: bool = true;
    const OPCODE: Opcode = LoopOpcode::<{ c::LOOP_CHANGE_FD }>::OPCODE;

    fn as_ptr(&mut self) -> *mut ffi::c_void {
        self.0.as_raw_fd() as *mut ffi::c_void
    }

    unsafe fn output_from_ptr(
        _ret: IoctlOutput,
        _arg: *mut ffi::c_void,
    ) -> io::Result<Self::Output> {
        Ok(())
    }
}

/// `ioctl(fd, LOOP_SET_CAPACITY, &capacity)` - Set the capacity of the loop device.
///
/// # References
/// - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man4/loop.4.html
#[doc(alias = "LOOP_SET_CAPACITY")]
#[inline]
pub fn ioctl_loop_set_capacity<F: AsFd>(fd: F, capacity: u64) -> io::Result<()> {
    unsafe { ioctl::ioctl(fd, LoopSetCapacity(capacity)) }
}

struct LoopSetCapacity(u64);

unsafe impl Ioctl for LoopSetCapacity {
    type Output = ();

    const IS_MUTATING: bool = true;
    const OPCODE: Opcode = LoopOpcode::<{ c::LOOP_SET_CAPACITY }>::OPCODE;

    fn as_ptr(&mut self) -> *mut ffi::c_void {
        self.0 as *mut ffi::c_void
    }

    unsafe fn output_from_ptr(
        _ret: IoctlOutput,
        _arg: *mut ffi::c_void,
    ) -> io::Result<Self::Output> {
        Ok(())
    }
}

/// `ioctl(fd, LOOP_SET_DIRECT_IO, &flag)` - Set the direct I/O flag of the loop device.
///
/// # References
/// - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man4/loop.4.html
#[doc(alias = "LOOP_SET_DIRECT_IO")]
#[inline]
pub fn ioctl_loop_set_direct_io<F: AsFd>(fd: F, flag: u32) -> io::Result<()> {
    unsafe { ioctl::ioctl(fd, LoopSetDirectIo(flag)) }
}

struct LoopSetDirectIo(u32);

unsafe impl Ioctl for LoopSetDirectIo {
    type Output = ();

    const IS_MUTATING: bool = true;
    const OPCODE: Opcode = LoopOpcode::<{ c::LOOP_SET_DIRECT_IO }>::OPCODE;

    fn as_ptr(&mut self) -> *mut ffi::c_void {
        self.0 as *mut ffi::c_void
    }

    unsafe fn output_from_ptr(
        _ret: IoctlOutput,
        _arg: *mut ffi::c_void,
    ) -> io::Result<Self::Output> {
        Ok(())
    }
}

/// `ioctl(fd, LOOP_SET_BLOCK_SIZE, &size)` - Set the block size of the loop device.
///
/// # References
/// - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man4/loop.4.html
#[doc(alias = "LOOP_SET_BLOCK_SIZE")]
#[inline]
pub fn ioctl_loop_set_block_size<F: AsFd>(fd: F, size: u32) -> io::Result<()> {
    unsafe { ioctl::ioctl(fd, LoopSetBlockSize(size)) }
}

struct LoopSetBlockSize(u32);

unsafe impl Ioctl for LoopSetBlockSize {
    type Output = ();

    const IS_MUTATING: bool = true;
    const OPCODE: Opcode = LoopOpcode::<{ c::LOOP_SET_BLOCK_SIZE }>::OPCODE;

    fn as_ptr(&mut self) -> *mut ffi::c_void {
        self.0 as *mut ffi::c_void
    }

    unsafe fn output_from_ptr(
        _ret: IoctlOutput,
        _arg: *mut ffi::c_void,
    ) -> io::Result<Self::Output> {
        Ok(())
    }
}

/// `ioctl(fd, LOOP_CONFIGURE, &config)` - Setup and configure loop device parameters.
///
/// # References
/// - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man4/loop.4.html
#[doc(alias = "LOOP_CONFIGURE")]
#[inline]
pub fn ioctl_loop_configure<F: AsFd>(fd: F, config: &c::LoopConfig<'_>) -> io::Result<()> {
    unsafe { ioctl::ioctl(fd, LoopConfigure(config)) }
}

struct LoopConfigure<'a, 'b>(&'a c::LoopConfig<'b>);

unsafe impl<'a, 'b> Ioctl for LoopConfigure<'a, 'b> {
    type Output = ();

    const IS_MUTATING: bool = false;
    const OPCODE: Opcode = LoopOpcode::<{ c::LOOP_CONFIGURE }>::OPCODE;

    fn as_ptr(&mut self) -> *mut ffi::c_void {
        self.0 as *const c::LoopConfig<'_> as *const ffi::c_void as *mut ffi::c_void
    }

    unsafe fn output_from_ptr(
        _ret: IoctlOutput,
        _arg: *mut ffi::c_void,
    ) -> io::Result<Self::Output> {
        Ok(())
    }
}

/// `ioctl(fd, LOOP_CTL_ADD, index)` - Add the new loop device specified by `index`.
///
/// # References
/// - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man4/loop.4.html
#[doc(alias = "LOOP_CTL_ADD")]
#[inline]
pub fn ioctl_loop_ctl_add<F: AsFd>(fd: F, index: u32) -> io::Result<u32> {
    unsafe { ioctl::ioctl(fd, LoopCtlAdd(index)) }
}

struct LoopCtlAdd(u32);

unsafe impl Ioctl for LoopCtlAdd {
    type Output = u32;

    const IS_MUTATING: bool = false;
    const OPCODE: Opcode = LoopOpcode::<{ c::LOOP_CTL_ADD }>::OPCODE;

    fn as_ptr(&mut self) -> *mut ffi::c_void {
        self.0 as *mut ffi::c_void
    }

    unsafe fn output_from_ptr(
        ret: IoctlOutput,
        _arg: *mut ffi::c_void,
    ) -> io::Result<Self::Output> {
        Ok(ret as u32)
    }
}

/// `ioctl(fd, LOOP_CTL_REMOVE, index)` - Remove the loop device specified by `index`.
///
/// # References
/// - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man4/loop.4.html
#[doc(alias = "LOOP_CTL_REMOVE")]
#[inline]
pub fn ioctl_loop_ctl_remove<F: AsFd>(fd: F, index: u32) -> io::Result<()> {
    unsafe { ioctl::ioctl(fd, LoopCtlRemove(index)) }
}

struct LoopCtlRemove(u32);

unsafe impl Ioctl for LoopCtlRemove {
    type Output = ();

    const IS_MUTATING: bool = false;
    const OPCODE: Opcode = LoopOpcode::<{ c::LOOP_CTL_REMOVE }>::OPCODE;

    fn as_ptr(&mut self) -> *mut ffi::c_void {
        self.0 as *mut ffi::c_void
    }

    unsafe fn output_from_ptr(
        _ret: IoctlOutput,
        _arg: *mut ffi::c_void,
    ) -> io::Result<Self::Output> {
        Ok(())
    }
}

/// `ioctl(fd, LOOP_CTL_GET_FREE)` - Allocate or find a free loop device for use.
///
/// # References
/// - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man4/loop.4.html
#[doc(alias = "LOOP_CTL_GET_FREE")]
#[inline]
pub fn ioctl_loop_ctl_get_free<F: AsFd>(fd: F) -> io::Result<u32> {
    unsafe { ioctl::ioctl(fd, LoopCtlGetFree) }
}

struct LoopCtlGetFree;

unsafe impl Ioctl for LoopCtlGetFree {
    type Output = u32;

    const IS_MUTATING: bool = false;
    const OPCODE: Opcode = LoopOpcode::<{ c::LOOP_CTL_GET_FREE }>::OPCODE;

    fn as_ptr(&mut self) -> *mut ffi::c_void {
        ptr::null_mut()
    }

    unsafe fn output_from_ptr(
        ret: IoctlOutput,
        _arg: *mut ffi::c_void,
    ) -> io::Result<Self::Output> {
        Ok(ret as u32)
    }
}
