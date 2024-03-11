use clap::Parser;

#[derive(Debug, Parser)]
pub enum Args {
    /// Power on the radio.
    On,
    /// Power off the radio.
    Off,
    /// Read the current state of the radio.
    Status,
}

fn main() {
    let _args = Args::parse();
}
