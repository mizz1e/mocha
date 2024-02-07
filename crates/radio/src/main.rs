use {
    radio_common::ipc,
    std::{fs, io},
    tokio::io::AsyncReadExt,
    tracing::info,
};

fn main() -> io::Result<()> {
    tracing_subscriber::fmt::init();

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .thread_name(env!("CARGO_PKG_NAME"))
        .build()?
        .block_on(run())
}

async fn run() -> io::Result<()> {
    let settings = fs::read_to_string("/etc/radio/settings.toml")?;
    let settings: radio_common::Settings = toml::from_str(&settings).map_err(io::Error::other)?;

    let mut boot = radio_common::Boot::open()?;

    boot.upload_firmware(&settings.firmware).await?;

    let mut ipc = ipc::IpcDevice::open(0)?;
    let mut ipc_5g = ipc::IpcDevice::open(1)?;
    let mut rfs = radio_common::device::Device::open("/dev/umts_rfs0")?;

    loop {
        let mut buf = [0u8; u8::MAX as usize];

        tokio::select! {
            result = ipc.next_event() => info!("ipc: {result:#?}"),
            result = ipc_5g.next_event() => info!("ipc 5g:n{result:#?}"),
            result = rfs.read(&mut buf) => info!("rfs: {result:#?}"),
        };
    }
}
