use {
    rustix::event::{self, PollFd, PollFlags},
    std::{
        fs::File,
        io::{self, Read, Write},
        slice,
    },
};

pub mod c;
pub mod ioctl;

pub struct UmtsBoot {
    device: File,
}

impl UmtsBoot {
    pub fn open() -> io::Result<Self> {
        File::options()
            .read(true)
            .write(true)
            .open("/dev/umts_boot0")
            .map(|device| Self { device })
    }

    pub fn reset(&mut self) -> io::Result<()> {
        ioctl::ioctl_modem_reset(&self.device)?;

        Ok(())
    }

    pub fn security_request(&mut self, request: c::ModemSecReq) -> io::Result<()> {
        ioctl::ioctl_security_req(&self.device, request)?;

        Ok(())
    }

    pub fn set_insecure(&mut self) -> io::Result<()> {
        let request = c::ModemSecReq {
            mode: 2,
            param2: 0,
            param3: 0,
            param4: 0,
        };

        self.security_request(request)
    }

    pub fn set_secure(&mut self, boot: &c::Toc, main: &c::Toc) -> io::Result<()> {
        let request = c::ModemSecReq {
            mode: 0,
            param2: boot.size,
            param3: main.size,
            param4: 0,
        };

        self.security_request(request)
    }

    fn upload_firmware_chunk(
        &mut self,
        size: u32,
        file_offset: u32,
        load_offset: u32,
        data: &[u8],
    ) -> io::Result<()> {
        let firmware = c::ModemFirmware {
            binary: data.as_ptr() as u64,
            size,
            m_offset: load_offset,
            b_offset: file_offset,
            mode: 0,
            len: data.len() as u32,
        };

        ioctl::ioctl_modem_xmit_boot(&self.device, firmware)?;

        Ok(())
    }

    pub fn upload_firmware(
        &mut self,
        base_address: u32,
        toc: &c::Toc,
        firmware: &[u8],
    ) -> io::Result<()> {
        let size = toc.size;
        let mut file_offset = toc.offset;
        let mut load_offset = toc.load_address - base_address;

        for data in firmware.chunks(62 * 1024) {
            self.upload_firmware_chunk(size, file_offset, load_offset, data)?;

            file_offset += data.len() as u32;
            load_offset += data.len() as u32;
        }

        Ok(())
    }

    pub fn power_on(&mut self) -> io::Result<()> {
        ioctl::ioctl_modem_on(&self.device)?;

        Ok(())
    }

    pub fn power_off(&mut self) -> io::Result<()> {
        ioctl::ioctl_modem_off(&self.device)?;

        Ok(())
    }

    pub fn power_boot_on(&mut self) -> io::Result<()> {
        ioctl::ioctl_modem_boot_on(&self.device)?;

        Ok(())
    }

    pub fn power_boot_off(&mut self) -> io::Result<()> {
        ioctl::ioctl_modem_boot_off(&self.device)?;

        Ok(())
    }

    pub fn boot_dowload_firmware(&mut self) -> io::Result<()> {
        ioctl::ioctl_modem_dl_start(&self.device)?;

        Ok(())
    }

    pub fn boot_handshake(&mut self) -> io::Result<()> {
        let _ = self.device.write(&[13, 144, 0, 0])?;

        let mut poll = PollFd::new(&self.device, PollFlags::IN);

        event::poll(slice::from_mut(&mut poll), 30)?;

        let mut bytes = [0u8; 4];
        let _ = self.device.read(&mut bytes)?;

        let _ = self.device.write(&[0, 159, 0, 0])?;

        let mut poll = PollFd::new(&self.device, PollFlags::IN);

        event::poll(slice::from_mut(&mut poll), 30)?;

        let mut bytes = [0u8; 4];
        let _ = self.device.read(&mut bytes)?;

        Ok(())
    }

    pub fn as_device_mut(&mut self) -> &mut File {
        &mut self.device
    }
}
