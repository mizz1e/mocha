use {rustix::io, std::ops::Range};

pub const MODEM_ON: u8 = 0x19;
pub const MODEM_OFF: u8 = 0x20;
pub const MODEM_RESET: u8 = 0x21;
pub const MODEM_BOOT_ON: u8 = 0x22;
pub const MODEM_BOOT_OFF: u8 = 0x23;
pub const MODEM_BOOT_DONE: u8 = 0x24;
pub const MODEM_PROTOCOL_SUSPEND: u8 = 0x25;
pub const MODEM_PROTOCOL_RESUME: u8 = 0x26;
pub const MODEM_STATUS: u8 = 0x27;
pub const MODEM_DL_START: u8 = 0x28;
pub const MODEM_FW_UPDATE: u8 = 0x29;
pub const MODEM_NET_SUSPEND: u8 = 0x30;
pub const MODEM_NET_RESUME: u8 = 0x31;
pub const MODEM_DUMP_START: u8 = 0x32;
pub const MODEM_DUMP_UPDATE: u8 = 0x33;
pub const MODEM_FORCE_CRASH_EXIT: u8 = 0x34;
pub const MODEM_CP_UPLOAD: u8 = 0x35;
pub const MODEM_DUMP_RESET: u8 = 0x36;
pub const LINK_CONNECTED: u8 = 0x33;
pub const MODEM_SET_TX_LINK: u8 = 0x37;
pub const MODEM_RAMDUMP_START: u8 = 0xCE;
pub const MODEM_RAMDUMP_STOP: u8 = 0xCF;
pub const MODEM_XMIT_BOOT: u8 = 0x40;
pub const MODEM_GET_SHMEM_INFO: u8 = 0x41;
pub const DPRAM_INIT_STATUS: u8 = 0x43;
pub const LINK_DEVICE_RESET: u8 = 0x44;
pub const MODEM_GET_SHMEM_SRINFO: u8 = 0x45;
pub const MODEM_SET_SHMEM_SRINFO: u8 = 0x46;
pub const MODEM_GET_CP_BOOTLOG: u8 = 0x47;
pub const MODEM_CLR_CP_BOOTLOG: u8 = 0x48;
pub const MIF_LOG_DUMP: u8 = 0x51;
pub const MIF_DPRAM_DUMP: u8 = 0x52;
pub const SECURITY_REQ: u8 = 0x53;
pub const SHMEM_FULL_DUMP: u8 = 0x54;
pub const MODEM_CRASH_REASON: u8 = 0x55;
pub const MODEM_AIRPLANE_MODE: u8 = 0x56;
pub const VSS_FULL_DUMP: u8 = 0x57;
pub const ACPM_FULL_DUMP: u8 = 0x58;
pub const CPLOG_FULL_DUMP: u8 = 0x59;
pub const DATABUF_FULL_DUMP: u8 = 0x5A;
pub const REGISTER_PCIE: u8 = 0x65;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum ModemState {
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

impl TryFrom<u32> for ModemState {
    type Error = io::Errno;

    fn try_from(state: u32) -> Result<Self, Self::Error> {
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
            _ => return Err(io::Errno::INVAL),
        };

        Ok(state)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum LinkState {
    Offline = 0,
    IPC = 1,
    CPCrash = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct SimState {
    pub online: bool,
    pub changed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C, packed)]
pub struct ModemFirmware {
    pub binary: u64,
    pub size: u32,
    pub m_offset: u32,
    pub b_offset: u32,
    pub mode: u32,
    pub len: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C, packed)]
pub struct ModemSecReq {
    pub mode: u32,
    pub param2: u32,
    pub param3: u32,
    pub param4: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum CpBootMode {
    Normal = 0,
    Dump = 1,
    ReInit = 2,
    ReqCpRamLogging = 5,
    Manual = 7,
    MaxCpBootMode = 8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C, packed)]
pub struct SecInfo {
    pub mode: CpBootMode,
    pub size: u32,
}

#[binrw::binrw]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TocName {
    #[brw(magic = b"TOC\0\0\0\0\0\0\0\0\0")]
    Header,

    #[brw(magic = b"BOOT\0\0\0\0\0\0\0\0")]
    Boot,

    #[brw(magic = b"MAIN\0\0\0\0\0\0\0\0")]
    Main,

    #[brw(magic = b"VSS\0\0\0\0\0\0\0\0\0")]
    Vss,

    #[brw(magic = b"APM\0\0\0\0\0\0\0\0\0")]
    Apm,

    #[brw(magic = b"NV\0\0\0\0\0\0\0\0\0\0")]
    Nv,
}

#[binrw::binrw]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Toc {
    pub name: TocName,
    pub offset: u32,
    pub load_address: u32,
    pub size: u32,
    pub crc: u32,
    pub entry_id: u32,
}

impl Toc {
    pub fn range(&self) -> Range<usize> {
        let offset = self.offset as usize;
        let size = self.size as usize;

        offset..(offset + size)
    }
}

#[binrw::binrw]
#[brw(little)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Firmware {
    pub header: Toc,
    pub boot: Toc,
    pub main: Toc,
    pub unspecified: Toc,
    pub nv: Toc,
}

#[binrw::binrw]
#[brw(little)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SipcFmtHdr {
    pub len: u16,
    pub msg_seq: u8,
    pub ack_seq: u8,
    pub main_cmd: u8,
    pub sub_cmd: u8,
    pub cmd_type: u8,
}
