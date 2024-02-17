/// Define framebuffer parameters.
macro_rules! framebuffer {
    ($($board:literal = $address:literal @ $width:literal x $height:literal,)*) => {
        {
            $(#[cfg(feature = $board)] {
                unsafe {
                    $crate::framebuffer::Framebuffer::from_raw_parts($address, $width, $height)
                }
            })*

            #[cfg(not(any($(feature = $board,)*)))] {
                compile_error!("Unsupported SoC")
            }
        }
    };
}

/// Bootloader provided framebuffer.
pub struct Framebuffer {
    address: *mut Rgba,
    width: u16,
    height: u16,
}

impl Framebuffer {
    /// Create a new `Display` from raw parts.
    ///
    /// # Safety
    ///
    /// `address` must point to the bootloader provided framebuffer,
    /// `width` must match the framebuffer width exactly.
    /// `height` must match the framebuffer height exactly.
    pub unsafe fn from_raw_parts(address: usize, width: u16, height: u16) -> Self {
        Self {
            address: address as *mut Rgba,
            width,
            height,
        }
    }

    /// Returns a const pointer to the address of the framebuffer
    pub fn as_ptr(&self) -> *const Rgba {
        self.address as *const Rgba
    }

    /// Returns a mutable pointer to the address of the framebuffer
    pub fn as_mut_ptr(&mut self) -> *mut Rgba {
        self.address
    }

    /// Returns the width of the framebuffer.
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Returns the height of the framebuffer.
    pub fn height(&self) -> u16 {
        self.height
    }

    /// Returns the total area of the framebuffer.
    pub fn area(&self) -> u32 {
        self.width as u32 * self.height as u32
    }

    /// Set the framebuffer to the specified `color`.
    pub fn clear(&mut self, color: Rgba) {
        for i in 0..self.area() {
            unsafe {
                self.as_mut_ptr().add(i as usize).write_volatile(color);
            }
        }
    }
}

/// An 8-bit RGBA color.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Rgba {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Rgba {
    /// From a packed RGBA color (u32).
    pub const fn from_packed(packed: u32) -> Self {
        let [red, green, blue, alpha] = packed.to_ne_bytes();

        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

pub(crate) use framebuffer;
