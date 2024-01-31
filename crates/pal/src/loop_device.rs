use std::{fs::File, io, os::fd::AsFd};

pub mod c;
pub mod ioctl;

pub struct LoopControl {
    device: File,
}

impl LoopControl {
    pub fn open() -> io::Result<Self> {
        File::open("/dev/loop-control").map(|device| Self { device })
    }

    pub fn create_device(&mut self, index: u32) -> io::Result<()> {
        ioctl::ioctl_loop_ctl_add(&self.device, index)?;

        Ok(())
    }

    pub fn remove_device(&mut self, index: u32) -> io::Result<()> {
        ioctl::ioctl_loop_ctl_remove(&self.device, index)?;

        Ok(())
    }

    pub fn next_free_device(&mut self) -> io::Result<u32> {
        let index = ioctl::ioctl_loop_ctl_get_free(&self.device)?;

        Ok(index)
    }
}

pub struct LoopDevice {
    device: File,
}

impl LoopDevice {
    pub fn open(index: u32) -> io::Result<Self> {
        File::open(format!("/dev/loop{index}")).map(|device| Self { device })
    }

    pub fn set_backing_file<F: AsFd>(&mut self, backing_file: F) -> io::Result<()> {
        ioctl::ioctl_loop_set_fd(&self.device, backing_file)?;

        Ok(())
    }

    pub fn clear_backing_file(&mut self) -> io::Result<()> {
        ioctl::ioctl_loop_clr_fd(&self.device)?;

        Ok(())
    }
}
