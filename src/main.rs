#![feature(naked_functions)]
#![no_std]
#![no_main]

use {
    crate::{
        display::display,
        framebuffer::{framebuffer, Rgba},
    },
    core::{arch, panic::PanicInfo},
};

pub mod display;
pub mod framebuffer;

#[naked]
#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    arch::asm!(
        // code0
        "b {main}",
        // code1
        ".4byte 0",
        // text_offset
        ".8byte 0x80000",
        // image_size
        // TODO: Automate determining the size?
        ".8byte 168",
        // flags
        ".8byte 0b1010",
        // res2
        ".8byte 0",
        // res3
        ".8byte 0",
        // res4
        ".8byte 0",
        // magic
        ".ascii \"ARM\x64\"",
        // res5
        ".4byte 0",
        main = sym main,
        options(noreturn),
    )
}

fn main() -> ! {
    // Display-related addresses and values.
    //
    // `decon` can be found from downstream kernels, search for `decon_f@0x` in DTBs.
    //
    // For example, Exynos 9820 devices have `decon_f@0x19030000`:
    //
    // ```
    // arch/arm64/boot/dts/exynos/exynos9820.dts
    // 1936:   decon_f: decon_f@0x19030000 {
    // ```
    //
    // As for `control`, I have no idea, these links mention the values values:
    //
    // - [uniLoader](https://github.com/ivoszbg/uniLoader)
    // - [PostmarketOS Wiki - Samsung Galaxy S7](https://wiki.postmarketos.org/wiki/Samsung_Galaxy_S7_(samsung-herolte))
    let mut display = display! {
        decon {
            "exynos7420" = 0x1393_0000,
            "exynos7570" = 0x1483_0000,
            "exynos7885" = 0x1486_0000,
            "exynos8895" = 0x1286_0000,
            "exynos9810" = 0x1603_0000,
            "exynos9820" = 0x1903_0000,
            "exynos990" = 0x1905_0000,
        },

        control {
            "exynos7420" = (0x6B0, 0x2058),
            _ => (0x70, 0x1281),
        },
    };

    // S-Boot passes this value to Linux.
    // Check `/proc/cmdline` for similar to `s3cfb.bootloaderfb=0xca00000`.
    // Resolution can be obtained from various online sources (GSMArena).
    let mut framebuffer = framebuffer! {
        // Samsung Galaxy A8 (2018)
        "jackpotlte" = 0xEC00_0000 @ 1080 x 2220,
        // Samsung Galaxy Note 5
        "noblelte" = 0x0E2A_0000 @ 1440 x 2560,
        // Samsung Galaxy S6
        "zeroflte" = 0x0E2A_0000 @ 1440 x 2560,
        // Samsung Galaxy S8
        "dreamlte" = 0x0CC0_0000 @ 1440 x 2960,
        // Samsung Galaxy S9
        "starlte" = 0x0CC0_0000 @ 1440 x 2960,
        // Samsung Galaxy S10 5G
        "beyondx" = 0xCA00_0000 @ 1440 x 3040,
        // Samsung Galaxy S20
        "x1s" = 0xF100_0000 @ 1440 x 3200,
        // Samsung Galaxy Note 20
        "c1s" = 0xF100_0000 @ 1440 x 3200,
    };

    // Enable software control of the display.
    display.set_software_control();

    // Clear the framebuffer with red.
    // TODO: Some way to represent pixel formats, this is BGRA.
    framebuffer.clear(Rgba::from_packed(0x0000FFFF));

    #[allow(clippy::empty_loop)]
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    #[allow(clippy::empty_loop)]
    loop {}
}
