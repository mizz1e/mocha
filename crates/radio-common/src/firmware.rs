use {
    binrw::BinRead,
    std::{
        fmt,
        io::{self, Read, Seek},
    },
};

macro_rules! labels {
    ($($(#[$meta:meta])* ($variant:ident, $constant:ident, $bytes:literal)),*$(,)?) => {
        /// The 12-byte label of a ToC entry.
        #[binrw::binrw]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        #[brw(little)]
        pub enum Label {
            $($(#[$meta])* #[brw(magic = $bytes)] $variant,)*
        }

        impl Label {
            // This also guarantees that `$bytes` is exactly 12 bytes.
            $($(#[$meta])* pub const $constant: [u8; 12] = *$bytes;)*

            /// Returns a string of this label.
            pub fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => stringify!($constant),)*
                }
            }
        }

        impl fmt::Display for Label {
            #[inline]
            fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(self.as_str(), fmt)
            }
        }
    };
}

labels! {
    /// ToC header.
    (Toc, TOC, b"TOC\0\0\0\0\0\0\0\0\0"),
    /// Boot binary, radio bootloader.
    (Boot, BOOT, b"BOOT\0\0\0\0\0\0\0\0"),
    /// Main binary, the Shannon OS.
    (Main, MAIN, b"MAIN\0\0\0\0\0\0\0\0"),
    /// Yet to find out.
    (Vss, VSS, b"VSS\0\0\0\0\0\0\0\0\0"),
    /// Yet to find out.
    (Apm, APM, b"APM\0\0\0\0\0\0\0\0\0"),
    /// NV data, stored in the EFS partition.
    (Nv, NV, b"NV\0\0\0\0\0\0\0\0\0\0"),
    /// Yet to find out.
    (Offset, OFFSET, b"OFFSET\0\0\0\0\0\0"),
}

/// Table of Contents (ToC), describes where within radio firmware blobs are located,
/// and what memory address they shall be loaded to.
#[binrw::binrw]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[brw(little)]
pub struct Toc {
    /// Label of the blob.
    pub label: Label,
    /// Offset of the blob (always `0` for the ToC header).
    pub offset: u32,
    /// Address the blob will be loaded into.
    pub address: u32,
    /// Length, in bytes, of the blob.
    pub len: u32,
    /// CRC sum of the blob.
    // TODO: Implement CRC validation.
    pub crc: u32,
    /// ID of this blob or the count of blobs (for the ToC header).
    pub misc: u32,
}

/// ToC entries of a firmware blob.
#[binrw::binrw]
#[brw(little)]
#[brw(assert(header.label == Label::Toc))]
#[brw(assert(header.offset == 0))]
struct Header {
    /// The ToC header, provides how many blobs there are.
    header: Toc,
    /// ToC entries referring to the blobs.
    #[br(count = match header.misc {
       5 => 4,
       6 => 4,
       _ => unimplemented!("implement support for this firmware") 
    })]
    blobs: Vec<Toc>,
}

/// Read a ToC entry from the provided reader.
pub fn read_toc<R: Read + Seek>(reader: &mut R) -> io::Result<Toc> {
    BinRead::read(reader).map_err(io::Error::other)
}

/// Read ToC entries from a firmware blob.
pub fn read<R: Read + Seek>(reader: &mut R) -> io::Result<Vec<Toc>> {
    let header: Header = BinRead::read(reader).map_err(io::Error::other)?;

    Ok(header.blobs)
}
