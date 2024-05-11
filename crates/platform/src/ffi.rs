/// Define a C string in const contexts.
macro_rules! c_str {
    ($string:expr) => {
        unsafe { ::core::ffi::CStr::from_bytes_with_nul_unchecked($string.as_bytes()) }
    };
}

pub(crate) use c_str;
