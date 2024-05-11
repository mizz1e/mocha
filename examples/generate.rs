use mocha_console::{
    image::{ImageBuffer, ImageBufferError, Size, TextSystem},
    palette::rgb::PackedAbgr,
    Splash,
};

fn main() -> Result<(), ImageBufferError> {
    let splash = Splash {
        board_name: "samsung beyondx //////",
        size: Size::new(1440, 3040),
        ..Splash::debug()
    };

    let buffer: Vec<PackedAbgr> =
        vec![splash.background_color.into(); (splash.width() * splash.height()) as usize];

    let mut text_system = TextSystem::new();
    let mut image = ImageBuffer::new(splash.size, buffer)?;

    splash.draw(&mut image, &mut text_system);

    let jpeg = image.encode_jpeg()?;

    std::fs::write("splash.jpg", jpeg).unwrap();

    Ok(())
}
