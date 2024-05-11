#![no_std]

extern crate alloc;

use {
    crate::image::{ImageBuffer, Size, Text, TextSystem, Transform},
    bytemuck::Pod,
    glam::Vec2Swizzles,
    palette::Srgba,
};

pub use bytemuck;
pub use glam;
pub use palette;

pub mod image;

pub struct Splash {
    pub background_color: Srgba<u8>,
    pub board_name: &'static str,
    pub board_color: Srgba<u8>,
    pub line_color: Srgba<u8>,
    pub size: Size,
}

impl Splash {
    pub fn dark() -> Self {
        Self {
            background_color: Srgba::from(0xFF000000),
            board_name: "unknown //////",
            board_color: Srgba::from(0xFF606060),
            line_color: Srgba::from(0xFF303030),
            size: Size::ZERO,
        }
    }

    pub fn light() -> Self {
        Self {
            background_color: Srgba::from(0xFFF3F6Fb),
            board_name: "unknown //////",
            board_color: Srgba::from(0xFF93969B),
            line_color: Srgba::from(0xFFC3C6CB),
            size: Size::ZERO,
        }
    }

    pub fn debug() -> Self {
        Self {
            background_color: Srgba::from(0xFFFF0000),
            board_name: "unknown //////",
            board_color: Srgba::from(0xFF00FF00),
            line_color: Srgba::from(0xFF0000FF),
            size: Size::new(1440, 3040),
        }
    }

    pub fn width(&self) -> u32 {
        self.size.x
    }

    pub fn height(&self) -> u32 {
        self.size.y
    }

    pub fn draw<Pixel, Buffer>(
        &self,
        image: &mut ImageBuffer<Pixel, Buffer>,
        text_system: &mut TextSystem,
    ) where
        Pixel: Pod + From<Srgba<u8>> + Into<Srgba<u8>>,
        Buffer: AsRef<[Pixel]> + AsMut<[Pixel]>,
    {
        let board_name = self.board_name;
        let board_color = self.board_color;
        let line_color = Pixel::from(self.line_color);
        let size = self.size;

        let max = size.max_element();

        let padding = max / 8;
        let spacing = max / 64;
        let thickness = max / 256;

        let start = Size::new(spacing * 2, spacing);
        let line = Size::new(padding, thickness);
        let end_h = size - start.xy() - line.xy();
        let end_v = size - start.yx() - line.yx();

        // Top left.
        image.draw_rectangle(start.xy().as_ivec2(), line.xy(), line_color);
        image.draw_rectangle(start.yx().as_ivec2(), line.yx(), line_color);

        // Bottom right.
        image.draw_rectangle(end_h.as_ivec2(), line.xy(), line_color);
        image.draw_rectangle(end_v.as_ivec2(), line.yx(), line_color);

        // Top right.
        image.draw_rectangle(
            Size::new(end_h.x, start.y).as_ivec2(),
            line.xy(),
            line_color,
        );

        image.draw_rectangle(
            Size::new(end_v.x, start.x).as_ivec2(),
            line.yx(),
            line_color,
        );

        // Bottom left.
        image.draw_rectangle(
            Size::new(start.x, end_h.y).as_ivec2(),
            line.xy(),
            line_color,
        );

        image.draw_rectangle(
            Size::new(start.y, end_v.y).as_ivec2(),
            line.yx(),
            line_color,
        );

        let text_size = Size::new(64, 1200);
        let text = Text::new(board_name).color(board_color).scale(64.0);

        image.draw_text(
            text_system,
            Size::new(end_h.x + line.x - text_size.x, start.x).as_ivec2(),
            text_size,
            text.transform(Transform::Rotate90),
        );

        image.draw_text(
            text_system,
            Size::new(start.x, (end_v.y + line.x) - text_size.y).as_ivec2(),
            text_size,
            text.transform(Transform::Rotate180),
        );
    }
}
