use {
    clap::Parser,
    radio_common::firmware::{self, Toc},
    std::{
        fs::{self, File},
        io::{self, Read, Seek},
        path::PathBuf,
    },
};

#[derive(Debug, Parser)]
pub enum Args {
    /// Analyze radio firmware.
    Analyze {
        /// Path to firmware (`modem.bin`, `/dev/block/by-name/radio`, etc).
        path: PathBuf,
    },
    /// Extract firmware blobs.
    Extract {
        /// Path to firmware (`modem.bin`, `/dev/block/by-name/radio`, etc).
        path: PathBuf,
        /// Output directory of blobs.
        output_dir: PathBuf,
    },
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    match args {
        Args::Analyze { path } => {
            let mut file = File::open(path)?;
            let tocs = firmware::read(&mut file)?;

            if tocs.is_empty() {
                println!("No tocs.");

                return Ok(());
            }

            for toc in &tocs {
                print_toc(toc);
            }
        }
        Args::Extract { path, output_dir } => {
            let mut file = File::open(path)?;
            let tocs = firmware::read(&mut file)?;

            if tocs.is_empty() {
                println!("No tocs.");

                return Ok(());
            }

            fs::create_dir(&output_dir)?;

            for toc in &tocs {
                print_toc(toc);

                if toc.offset == 0 {
                    println!("External blob, nothing to extract.");
                    println!();

                    continue;
                }

                let output_path = output_dir.join(format!("{}.bin", toc.label));
                let mut output = File::create(&output_path)?;

                file.seek(io::SeekFrom::Start(toc.offset as u64))?;
                io::copy(&mut file.by_ref().take(toc.len as u64), &mut output)?;

                println!("Extracted to {output_path:?}.");
                println!();
            }
        }
    }

    Ok(())
}

/// Print a ToC entry.
fn print_toc(toc: &Toc) {
    println!("Entry: {}", toc.label);
    println!("  File offset: {}", toc.offset);
    println!("  Load address: 0x{:08X}", toc.address);
    println!("  Size (in bytes): {}", toc.len);
    println!("  CRC: {}", toc.crc);
    println!("  Entry ID: {}", toc.misc);
    println!()
}
