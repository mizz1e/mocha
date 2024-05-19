//! Device functionality.

use {
    rustix::system::{self, RebootCommand},
    std::{io, path::Path},
};

pub fn is_efi() -> io::Result<bool> {
    Path::new("/sys/firmware/efi/efivars").try_exists()
}

/// Power off the device.
pub fn power_off() -> io::Result<()> {
    system::reboot(RebootCommand::PowerOff)?;

    Ok(())
}

/// Restart the device.
pub fn restart() -> io::Result<()> {
    system::reboot(RebootCommand::Restart)?;

    Ok(())
}
