use {platform::fs::Dir, std::io};

fn main() -> io::Result<()> {
    let home = Dir::ROOT.options().open("/tmp")?;
    let _user = home.options().same_file_system(false).open("you")?;

    Ok(())
}
