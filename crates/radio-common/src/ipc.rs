use {
    crate::device::Device,
    binrw::BinRead,
    bytes::{Buf, BytesMut},
    futures_util::StreamExt,
    std::{io, marker::PhantomData, mem},
    tokio_util::codec::{Decoder, FramedRead},
    tracing::trace,
};

#[binrw::binrw]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[brw(little)]
#[repr(C)]
pub struct IpcHeader {
    /// Length of this packet (in bytes).
    ///
    /// Includes the size of the header (7 bytes).
    pub len: u16,
    pub message_sequence: u8,
    pub acknowledge_sequence: u8,
    pub group: u8,
    pub index: u8,
    pub kind: u8,
}

#[binrw::binrw]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[brw(little)]
#[repr(C)]
pub struct RfsHeader {
    /// Length of this packet (in bytes).
    ///
    /// Includes the size of the header (6 bytes).
    pub len: u32,
    pub index: u8,
    pub id: u8,
}

impl IpcHeader {
    /// Maximum length of this packet.
    pub const MAX_LEN: usize = u16::MAX as usize;

    /// Parse an IPC header from the provided bytes.
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        BinRead::read(&mut io::Cursor::new(bytes)).map_err(io::Error::other)
    }
}

impl RfsHeader {
    /// Maximum length of this packet.
    pub const MAX_LEN: usize = u32::MAX as usize;

    /// Parse an RFS header from the provided bytes.
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        BinRead::read(&mut io::Cursor::new(bytes)).map_err(io::Error::other)
    }
}

/// IPC event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IpcEvent {
    Call,
    Config,
    General,
    Gps,
    Imei,
    Init,
    Misc,
    Network,
    Power,
    Rfs,
    Sap,
    Sat,
    Security,
    Service,
    Sms,
    Sound,

    Unknown {
        message_sequence: u8,
        acknowledge_sequence: u8,
        group: u8,
        index: u8,
        kind: u8,
        data: Vec<u8>,
    },
}

/// Radio IPC event decoder.
#[derive(Default)]
pub struct IpcDecoder {
    _phantom: PhantomData<()>,
}

impl IpcDecoder {
    /// Create a new IPC event decoder.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Decoder for IpcDecoder {
    type Item = IpcEvent;
    type Error = io::Error;

    fn decode(&mut self, source: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Until there is enough data for a header.
        if source.len() < mem::size_of::<IpcHeader>() {
            source.reserve(mem::size_of::<IpcHeader>() - source.len());

            return Ok(None);
        }

        let IpcHeader {
            len,
            message_sequence,
            acknowledge_sequence,
            group,
            index,
            kind,
        } = IpcHeader::from_bytes(source)?;

        trace!("Decoded IPC header");
        trace!("  Length (in bytes): {len}");
        trace!("  Message sequence: {message_sequence}");
        trace!("  Acknowledge sequence: {acknowledge_sequence}");
        trace!("  Group: 0x{group:02X}");
        trace!("  Index: 0x{index:02X}");
        trace!("  Kind: 0x{kind:02X}");

        let len = len as usize;

        // Until there is enough data for the full IPC event.
        if source.len() < len {
            source.reserve(len - source.len());

            return Ok(None);
        }

        let data = source[mem::size_of::<IpcHeader>()..].to_vec();

        source.advance(len);

        let event = match group {
            0x01 => IpcEvent::Power,
            0x02 => IpcEvent::Call,
            0x03 => IpcEvent::Sms,
            0x13 => IpcEvent::Init,
            _ => IpcEvent::Unknown {
                message_sequence,
                acknowledge_sequence,
                group,
                index,
                kind,
                data,
            },
        };

        Ok(Some(event))
    }
}

/// A radio IPC device.
pub struct IpcDevice {
    reader: FramedRead<Device, IpcDecoder>,
}

impl IpcDevice {
    /// Open the specified device.
    ///
    /// Seems to be that:
    ///
    ///  - `0` for non-5G.
    ///  - `1` for 5G.
    ///
    pub fn open(index: u8) -> io::Result<Self> {
        let path = format!("/dev/umts_ipc{index}");

        Device::open(path).map(|device| {
            let reader = FramedRead::with_capacity(device, IpcDecoder::new(), IpcHeader::MAX_LEN);

            Self { reader }
        })
    }

    /// Wait for the next IPC event from the radio.
    pub async fn next_event(&mut self) -> io::Result<IpcEvent> {
        loop {
            if let Some(result) = self.reader.next().await {
                return result;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use {super::*, std::mem};

    #[test]
    fn header_size() {
        assert_eq!(mem::size_of::<IpcHeader>(), 7);
        assert_eq!(mem::size_of::<RfsHeader>(), 6);
    }
}
