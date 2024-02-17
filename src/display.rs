use core::ffi;

/// Define display parameters.
macro_rules! display {
    (
        decon {
            $($decon_soc:literal = $decon_address:literal,)*
        },

        control {
            $($control_soc:literal = ($control_offset:literal, $control_enable:literal),)*
            _ => ($default_control_offset:literal, $default_control_enable:literal),
        },
    ) => {
        {
            const CONTROL: (usize, u32) = {
                $(#[cfg(feature = $control_soc)] {
                    ($control_offset, $control_enable)
                })*

                #[cfg(not(any($(feature = $control_soc,)*)))] {
                    ($default_control_offset, $default_control_enable)
                }
            };

            $(#[cfg(feature = $decon_soc)] {
                const ADDRESS: usize = $decon_address;

                unsafe {
                    $crate::display::Display::from_raw_parts(ADDRESS, CONTROL.0, CONTROL.1)
                }
            })*

            #[cfg(not(any($(feature = $decon_soc,)*)))] {
                compile_error!("Unsupported board")
            }
        }
    };
}

/// The device's display.
pub struct Display {
    address: *mut ffi::c_void,
    control_offset: usize,
    control_enable: u32,
}

impl Display {
    /// Create a new `Display` from raw parts.
    ///
    /// # Safety
    ///
    /// `address` must point to the display,
    /// `control_offset` must point to the control of the display.
    /// `control_enable` must be the code to enable software rendering.
    pub unsafe fn from_raw_parts(
        address: usize,
        control_offset: usize,
        control_enable: u32,
    ) -> Self {
        Self {
            address: address as *mut ffi::c_void,
            control_offset,
            control_enable,
        }
    }

    /// Returns a mutable pointer to the address of the display.
    fn as_mut_ptr(&mut self) -> *mut ffi::c_void {
        self.address
    }

    /// Enable software control of the display.
    pub fn set_software_control(&mut self) {
        unsafe {
            self.as_mut_ptr()
                .byte_add(self.control_offset)
                .cast::<u32>()
                .write_volatile(self.control_enable)
        }
    }
}

pub(crate) use display;
