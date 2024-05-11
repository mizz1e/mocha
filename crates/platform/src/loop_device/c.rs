use std::os::fd::BorrowedFd;

// `/dev/loop{index}` `ioctl`s.
pub const LOOP_SET_FD: u8 = 0x00;
pub const LOOP_CLR_FD: u8 = 0x01;
pub const LOOP_SET_STATUS: u8 = 0x02;
pub const LOOP_GET_STATUS: u8 = 0x03;
pub const LOOP_SET_STATUS64: u8 = 0x04;
pub const LOOP_GET_STATUS64: u8 = 0x05;
pub const LOOP_CHANGE_FD: u8 = 0x06;
pub const LOOP_SET_CAPACITY: u8 = 0x07;
pub const LOOP_SET_DIRECT_IO: u8 = 0x08;
pub const LOOP_SET_BLOCK_SIZE: u8 = 0x09;
pub const LOOP_CONFIGURE: u8 = 0x0A;

// `/dev/loop-control` `ioctl`s.
pub const LOOP_CTL_ADD: u8 = 0x80;
pub const LOOP_CTL_REMOVE: u8 = 0x81;
pub const LOOP_CTL_GET_FREE: u8 = 0x82;

// Shared structure constants.
pub const LO_NAME_SIZE: usize = 64;
pub const LO_KEY_SIZE: usize = 32;

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    #[repr(transparent)]
    pub struct LoopFlags: u32 {
        const READ_ONLY = 1;
        const AUTOCLEAR = 4;
        const PARTSCAN = 8;
        const DIRECT_IO = 16;

        const _ = !0;
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct LoopInfo {
    pub lo_number: i32,
    pub lo_device: u32,
    pub lo_inode: u64,
    pub lo_rdevice: u32,
    pub lo_offset: i32,
    pub lo_encrypt_type: i32,
    pub lo_encrypt_key_size: i32,
    pub lo_flags: LoopFlags,
    pub lo_name: [u8; LO_NAME_SIZE],
    pub lo_encrypt_key: [u8; LO_KEY_SIZE],
    pub lo_init: [u64; 2],
    pub reserved: [u8; 4],
}

#[derive(Debug)]
#[repr(C)]
pub struct LoopInfo64 {
    pub lo_device: u64,
    pub lo_inode: u64,
    pub lo_rdevice: u64,
    pub lo_offset: u64,
    pub lo_sizelimit: u64,
    pub lo_number: u32,
    pub lo_encrypt_type: u32,
    pub lo_encrypt_key_size: u32,
    pub lo_flags: LoopFlags,
    pub lo_file_name: [u8; LO_NAME_SIZE],
    pub lo_crypt_name: [u8; LO_NAME_SIZE],
    pub lo_encrypt_key: [u8; LO_KEY_SIZE],
    pub lo_init: [u64; 2],
}

#[derive(Debug)]
#[repr(C)]
pub struct LoopConfig<'a> {
    pub fd: BorrowedFd<'a>,
    pub block_size: u32,
    pub info: LoopInfo64,
    pub __reserved: [u64; 8],
}
