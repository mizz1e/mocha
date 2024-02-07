# Devices

Devices are in the form `/dev/<network>_<interface><index>`.

### Network

- `/dev/umts_*` - Shannon310 baseband - 2G GSM, 3G UMTS, 4G LTE.
- `/dev/nr_*` - Shannon5100 baseband - 5G NR.

### Interfaces

- `boot` - Power management, boot process, administrative commands.
- `ipc` and `rfs` - IPC between Shannon baseband and Linux.

# Prior art

- [How to boot the Samsung Galaxy S7 modem with plain Linux kernel interfaces only](https://eighty-twenty.org/2020/09/10/booting-samsung-galaxy-s7-modem)
  - Demonstrates the Shannon310 boot process (`/dev/umts_boot0`), and describes the ToC format.
- [Reversing & Emulating Samsung's Shannon Baseband](https://hardwear.io/netherlands-2020/presentation/samsung-baseband-hardwear-io-nl-2020.pdf)
  - Demonstrates various details of the Shannon baseband, and describes the ToC format.
- [Replicant's libsamsung-ipc](https://redmine.replicant.us/projects/replicant/wiki/Libsamsung-ipc)
  - Definitions of various IPC commands, full RIL stack for older devices.

