use {
    alloc::vec::Vec,
    bytemuck::Pod,
    core::{fmt, marker::PhantomData},
    fontdue::layout,
    glam::Vec2Swizzles,
    palette::{
        blend::{Blend, Compose},
        rgb::PackedAbgr,
        LinSrgba, Srgb, Srgba,
    },
};

pub use glam::{IVec2 as Position, UVec2 as Size};

#[derive(Debug)]
pub enum ImageBufferError {
    SizeUnaddressable,
    SizeMismatch,
    JpegEncode(jpeg_encoder::EncodingError),
}

impl fmt::Display for ImageBufferError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SizeUnaddressable => {
                fmt.write_str("The provided size cannot be addressed on this platform")
            }
            Self::SizeMismatch => {
                fmt.write_str("The provided size do not match the associated buffer")
            }
            Self::JpegEncode(error) => write!(fmt, "JPEG encoder: {error}"),
        }
    }
}

/// Associate size with a buffer.
pub struct ImageBuffer<Pixel, Buffer>
where
    Pixel: Pod,
    Buffer: AsRef<[Pixel]>,
{
    size: Size,
    buffer: Buffer,
    _pixels: PhantomData<[Pixel]>,
}

impl<Pixel, Buffer> ImageBuffer<Pixel, Buffer>
where
    Pixel: Pod,
    Buffer: AsRef<[Pixel]>,
{
    /// Create a new `ImageBuffer`.
    pub fn new(size: Size, buffer: Buffer) -> Result<Self, ImageBufferError> {
        let area: usize = size
            .x
            .checked_mul(size.y)
            .and_then(|area| area.try_into().ok())
            .ok_or(ImageBufferError::SizeUnaddressable)?;

        if buffer.as_ref().len() == area {
            Ok(Self {
                size,
                buffer,
                _pixels: PhantomData,
            })
        } else {
            Err(ImageBufferError::SizeMismatch)
        }
    }

    /// Returns the underlying buffer.
    pub fn into_inner(self) -> Buffer {
        self.buffer
    }

    /// Returns a slice of pixels.
    pub fn as_slice(&self) -> &[Pixel] {
        self.buffer.as_ref()
    }

    /// Returns the width of the image.
    pub fn width(&self) -> u32 {
        self.size.x
    }

    /// Returns the height of the image.
    pub fn height(&self) -> u32 {
        self.size.y
    }

    /// Returns the size of the image.
    pub fn size(&self) -> Size {
        self.size
    }

    /// Returns the offset of `position` within the buffer.
    pub fn offset_of(&self, position: Position) -> Option<usize> {
        if position.is_negative_bitmask() != 0 {
            return None;
        }

        let position: Size = position.try_into().ok()?;

        if position.cmpge(self.size()).any() {
            return None;
        }

        Some((position.y * self.width() + position.x) as usize)
    }

    /// Returns a pixel at `position`.
    pub fn pixel(&self, position: Position) -> Option<Pixel> {
        let offset = self.offset_of(position)?;

        Some(self.as_slice()[offset])
    }
}

impl<Pixel, Buffer> ImageBuffer<Pixel, Buffer>
where
    Pixel: Pod,
    Buffer: AsRef<[Pixel]> + AsMut<[Pixel]>,
{
    /// Returns a mutable slice of pixels.
    pub fn as_slice_mut(&mut self) -> &mut [Pixel] {
        self.buffer.as_mut()
    }

    /// Set a pixel at `position`.
    pub fn put_pixel(&mut self, position: Position, pixel: Pixel) -> bool {
        let Some(offset) = self.offset_of(position) else {
            return false;
        };

        self.as_slice_mut()[offset] = pixel;

        true
    }

    /// Draw a region, filled with pixels from `for_pixel`.
    pub fn draw_with<ForPixel>(
        &mut self,
        position: Position,
        mut size: Size,
        mut for_pixel: ForPixel,
    ) where
        ForPixel: FnMut(Position, Pixel) -> Pixel,
    {
        // Shift the size
        if position.x.is_negative() {
            size.x = size.x.saturating_sub(position.x.unsigned_abs());
        }

        if position.y.is_negative() {
            size.y = size.y.saturating_sub(position.y.unsigned_abs());
        }

        // Completely not-visible.
        if size == Size::ZERO {
            return;
        }

        let size = size.min(self.size).try_into().unwrap_or(Position::MAX);

        for x in 0..size.x {
            for y in 0..size.y {
                let other_position = Position::new(x, y);
                let self_position = position + other_position;
                let pixel = self.pixel(self_position).unwrap();

                self.put_pixel(self_position, for_pixel(other_position, pixel));
            }
        }
    }

    /// Draw a rentangle.
    pub fn draw_rectangle(&mut self, position: Position, size: Size, pixel: Pixel) {
        self.draw_with(position, size, |_position, _pixel| pixel);
    }

    /// Draw a buffer.
    pub fn draw_buffer<OtherPixel, OtherBuffer, MapPixel>(
        &mut self,
        position: Position,
        image: &ImageBuffer<OtherPixel, OtherBuffer>,
        mut map_pixel: MapPixel,
    ) where
        OtherPixel: Pod,
        OtherBuffer: AsRef<[OtherPixel]>,
        MapPixel: FnMut(OtherPixel) -> Pixel,
    {
        self.draw_with(position, image.size(), |position, _pixel| {
            map_pixel(image.pixel(position).unwrap())
        });
    }
}

fn encode_jpeg(
    buffer: &[u8],
    width: u32,
    height: u32,
    kind: jpeg_encoder::ColorType,
) -> Result<Vec<u8>, ImageBufferError> {
    let mut bytes = Vec::new();

    let width = width
        .try_into()
        .map_err(|_error| ImageBufferError::SizeUnaddressable)?;

    let height = height
        .try_into()
        .map_err(|_error| ImageBufferError::SizeUnaddressable)?;

    jpeg_encoder::Encoder::new(&mut bytes, 100)
        .encode(buffer, width, height, kind)
        .map_err(ImageBufferError::JpegEncode)?;

    Ok(bytes)
}

impl<Buffer> ImageBuffer<Srgb<u8>, Buffer>
where
    Buffer: AsRef<[Srgb<u8>]>,
{
    /// Encode the buffer into a JPEG image.
    pub fn encode_jpeg(&self) -> Result<Vec<u8>, ImageBufferError> {
        let buffer = bytemuck::cast_slice(self.as_slice());
        let width = self.width();
        let height = self.height();
        let kind = jpeg_encoder::ColorType::Rgb;

        encode_jpeg(buffer, width, height, kind)
    }
}

impl<Buffer> ImageBuffer<PackedAbgr, Buffer>
where
    Buffer: AsRef<[PackedAbgr]>,
{
    /// Encode the buffer into a JPEG image.
    pub fn encode_jpeg(&self) -> Result<Vec<u8>, ImageBufferError> {
        let buffer = bytemuck::cast_slice(self.as_slice());
        let width = self.width();
        let height = self.height();
        let kind = jpeg_encoder::ColorType::Rgba;

        encode_jpeg(buffer, width, height, kind)
    }
}

impl<Pixel, Buffer> ImageBuffer<Pixel, Buffer>
where
    Pixel: Pod + From<Srgba<u8>> + Into<Srgba<u8>>,
    Buffer: AsRef<[Pixel]> + AsMut<[Pixel]>,
{
    pub fn draw_text(
        &mut self,
        text_system: &mut TextSystem,
        position: Position,
        size: Size,
        text: Text<'_>,
    ) {
        // Swap axes for vertical transforms.
        let text_size = if matches!(text.transform, Transform::Rotate90 | Transform::Rotate180) {
            size.yx()
        } else {
            size
        };

        text_system.reset(text_size);
        text_system.push(&text);

        for (mut glyph_offset, glyph_image) in text_system.glyphs() {
            let mut glyph_size = glyph_image.size();

            // FIXME: Why does this correct the position?
            glyph_offset.y -= 20;

            // Swap axes for vertical transforms.
            if matches!(text.transform, Transform::Rotate90 | Transform::Rotate180) {
                glyph_offset = glyph_offset.yx();
                glyph_size = glyph_size.yx();
            }

            let mut glyph_position = position + glyph_offset;

            if matches!(text.transform, Transform::Rotate90) {
                glyph_position.x =
                    position.x + size.x as i32 - glyph_offset.x - glyph_size.x as i32;
            } else if matches!(text.transform, Transform::Rotate180) {
                glyph_position.x = position.x + glyph_offset.x;
                glyph_position.y =
                    position.y + size.y as i32 - glyph_offset.y - glyph_size.y as i32;
            }

            self.draw_with(glyph_position, glyph_size, |mut glyph_position, pixel| {
                // Swap axes for vertical transforms.
                if matches!(text.transform, Transform::Rotate90 | Transform::Rotate180) {
                    glyph_position = glyph_position.yx();
                }

                if matches!(text.transform, Transform::Rotate90) {
                    glyph_position.y = glyph_size
                        .x
                        .saturating_sub(1)
                        .saturating_sub(glyph_position.y as u32)
                        as i32;
                } else if matches!(text.transform, Transform::Rotate180) {
                    glyph_position.x = glyph_size
                        .y
                        .saturating_sub(1)
                        .saturating_sub(glyph_position.x as u32)
                        as i32;
                }

                let background: Srgba<u8> = pixel.into();
                let desired: Srgba<u8> = text.color;
                let glyph_alpha: u8 = glyph_image.pixel(glyph_position).unwrap();
                let glyph: Srgba<u8> = Srgba {
                    color: Srgb::from(0xFFFFFF00),
                    alpha: glyph_alpha,
                };

                let background: LinSrgba<f32> = background.into_linear();
                let desired: LinSrgba<f32> = desired.into_linear();
                let glyph: LinSrgba<f32> = glyph.into_linear();

                let glyph = desired.multiply(glyph);
                let result = glyph.over(background);

                Pixel::from(Srgba::<u8>::from_linear(result))
            });
        }

        text_system.clear();
    }
}

pub struct TextSystem {
    fonts: [fontdue::Font; 1],
    layout: layout::Layout,
}

impl TextSystem {
    pub fn new() -> Self {
        let roboto_mono = fontdue::Font::from_bytes(
            include_bytes!("../assets/fonts/RobotoMono-Regular.ttf") as &[u8],
            fontdue::FontSettings::default(),
        )
        .unwrap();

        let fonts = [roboto_mono];
        let layout = layout::Layout::new(layout::CoordinateSystem::PositiveYDown);

        Self { fonts, layout }
    }

    fn reset(&mut self, size: Size) {
        self.layout.reset(&layout::LayoutSettings {
            max_width: Some(size.x as f32),
            max_height: Some(size.y as f32),
            ..Default::default()
        });
    }

    fn push(&mut self, text: &Text<'_>) {
        self.layout.append(
            &self.fonts,
            &layout::TextStyle::new(text.text, text.scale, 0),
        );
    }

    fn glyphs(&self) -> impl Iterator<Item = (Position, ImageBuffer<u8, Vec<u8>>)> + '_ {
        self.layout.glyphs().iter().flat_map(|glyph| {
            let glyph_width: u32 = glyph.width.try_into().ok()?;
            let glyph_height: u32 = glyph.height.try_into().ok()?;

            let position = Position::new(glyph.x as i32, glyph.y as i32);
            let size = Size::new(glyph_width, glyph_height);

            let (_metrics, buffer) = self.fonts[glyph.font_index].rasterize_config(glyph.key);
            let image = ImageBuffer::new(size, buffer).unwrap();

            Some((position, image))
        })
    }

    fn clear(&mut self) {
        self.layout.clear();
    }
}

impl Default for TextSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Text<'a> {
    color: Srgba<u8>,
    scale: f32,
    text: &'a str,
    transform: Transform,
}

impl<'a> Text<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            color: Srgba::from(0xFFFFFFFF),
            scale: 40.0,
            text,
            transform: Transform::default(),
        }
    }

    pub fn color(mut self, color: Srgba<u8>) -> Self {
        self.color = color;
        self
    }

    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    pub fn transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Transform {
    #[default]
    None,
    Rotate90,
    Rotate180,
}
