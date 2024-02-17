This enables software rendering to the display, for Linux to show a fbcon (not yet).

### Building

Compile this for your device, see supported devices in `Cargo.toml`.

```bash
cargo build --features=<device> --target=aarch64-unknown-none --release
```

This will produce an ELF image, so copy the Linux arm64 boot image out of it:
  
```bash
llvm-objcopy -O binary target/aarch64-unknown-none/release/sboot-handover sboot-handover
```

Use the boot image, `sboot-handover` as your kernel when creating an Android Boot image, for example:

```bash
$ mkbootimg \
  --header_version 1 \
  --os_version 13.0.0 \
  --os_patch_level 2024-12 \
  --kernel sboot-handover \
  --pagesize 0x00000800 \
  --base 0x00000000 \
  --kernel_offset 0x10008000 \
  --ramdisk_offset 0x11000000 \
  --second_offset 0x00000000 \
  --tags_offset 0x10000100 \
  --board SRPSC04B011KU \
  --cmdline "" \
  --output boot.img
```

### Thanks

- [PostmarketOS Wiki - Exynos mainline porting](https://wiki.postmarketos.org/wiki/Exynos_mainline_porting) - Links to various Exynos resources, and code examples.
- [VDavid003's minimal_sboot_wrapper](https://github.com/VDavid003/minimal_sboot_wrapper) - Simple, and clear how this process works.
- [ivoszbg's uniLoader](https://github.com/ivoszbg/uniloader) - Various device configurations.
- [A freestanding Rust binary](https://os.phil-opp.com/freestanding-rust-binary) - As the title states.
- [How U-Boot loads Linux kernel](https://krinkinmu.github.io/2023/08/21/how-u-boot-loads-linux-kernel.html) - Explains the process of U-Boot loading Linux arm64 boot images in-depth.
