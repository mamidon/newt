use crate::backend::Gpu;
use std::marker::PhantomData;

mod backend;

pub type DrawingError = &'static str;
pub type DrawingResult<T> = Result<T, DrawingError>;

pub type Color = u64;

#[derive(Copy, Clone)]
pub struct Brush {
    pub foreground: Color,
    pub background: Color,
}
pub struct TextureRGBA {
    width: usize,
    height: usize,
    bytes: Vec<u8>,
}
pub struct TextureGreyScale {
    width: usize,
    height: usize,
    bytes: Vec<u8>,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct Handle {
    generation: usize,
    key: usize,
}

pub type TextureId = Handle;
pub type MaskId = Handle;

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct Extent {
    x: i64,
    y: i64,
    width: u32,
    height: u32,
}

impl From<(u32, u32)> for Extent {
    fn from(tuple: (u32, u32)) -> Self {
        Extent {
            x: 0,
            y: 0,
            width: tuple.0,
            height: tuple.1,
        }
    }
}

#[derive(Copy, Clone)]
pub enum ShapeKind {
    Rectangle,
    Ellipse,
    Line,
}

#[derive(Copy, Clone)]
pub enum DrawCommand {
    Shape {
        brush: Brush,
        kind: ShapeKind,
        extent: Extent,
    },
    Glyph {
        texture: TextureId,
        extent: Extent,
    },
    Mask {
        mask: MaskId,
        brush: Brush,
        extent: Extent,
    },
}

pub struct Drawing {
    options: DrawingOptions,
    backend_gpu: Gpu,
}

#[derive(Copy, Clone)]
pub struct DrawingOptions {
    pub width: usize,
    pub height: usize,
}
pub struct RasterizedDrawList;
pub struct MaterialCollection;
pub struct DrawList {
    commands: Vec<DrawCommand>,
}

impl Drawing {
    pub fn initialize(options: DrawingOptions) -> DrawingResult<Self> {
        Ok(Drawing {
            options,
            backend_gpu: Gpu::initialize()?,
        })
    }

    pub fn create_draw_list(&self) -> DrawingResult<DrawList> {
        Ok(DrawList::empty())
    }

    pub fn rasterize_draw_list(
        &mut self,
        _draw_list: DrawList,
    ) -> DrawingResult<RasterizedDrawList> {
        unimplemented!()
    }
}

impl Default for DrawingOptions {
    fn default() -> Self {
        DrawingOptions {
            width: 64,
            height: 64,
        }
    }
}

impl MaterialCollection {
    pub fn new() -> DrawingResult<MaterialCollection> {
        Ok(MaterialCollection {})
    }

    pub fn create_material_glyph(
        &mut self,
        _material: impl Into<TextureRGBA>,
    ) -> DrawingResult<TextureId> {
        unimplemented!()
    }

    pub fn create_material_mask(
        &mut self,
        _material: impl Into<TextureGreyScale>,
    ) -> DrawingResult<MaskId> {
        unimplemented!()
    }
}

impl DrawList {
    pub fn empty() -> DrawList {
        DrawList {
            commands: Vec::new(),
        }
    }

    pub fn push(&mut self, command: DrawCommand) {
        self.commands.push(command)
    }
}

impl RasterizedDrawList {
    pub(crate) fn new() -> Self {
        RasterizedDrawList {}
    }

    pub fn present_to_screen(&mut self) -> DrawingResult<()> {
        unimplemented!()
    }

    pub fn present_to_texture(&mut self) -> DrawingResult<TextureRGBA> {
        unimplemented!()
    }
}

impl TextureRGBA {
    pub fn new(width: usize, height: usize, bytes: Vec<u8>) -> TextureRGBA {
        TextureRGBA {
            width,
            height,
            bytes,
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> u32 {
        let start_index = self.get_index(x, y);
        let pixel_bytes: Vec<u32> = self.bytes[start_index..start_index + 4]
            .iter()
            .map(|byte| *byte as u32)
            .collect();

        (pixel_bytes[0] << 24 | pixel_bytes[1] << 16 | pixel_bytes[2] << 8 | pixel_bytes[3])
    }

    fn get_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }
}

#[cfg(test)]
mod tests {
    use crate::{Brush, DrawCommand, Drawing, DrawingOptions, ShapeKind};

    struct Harness {
        pub drawing: Drawing,
    }

    impl Harness {
        fn setup() -> Harness {
            Harness {
                drawing: Drawing::initialize(DrawingOptions::default())
                    .expect("Drawing failed to initialize for tests"),
            }
        }
    }

    #[test]
    fn example() {
        let mut harness = Harness::setup();

        let mut draw_list = harness
            .drawing
            .create_draw_list()
            .expect("Failed to create draw list");

        draw_list.push(DrawCommand::Shape {
            kind: ShapeKind::Ellipse,
            brush: Brush {
                foreground: 0xFF0000FF,
                background: 0x00FF00FF,
            },
            extent: (64, 64).into(),
        });

        let mut rasterization = harness
            .drawing
            .rasterize_draw_list(draw_list)
            .expect("Failed to rasterize draw list");

        let texture = rasterization
            .present_to_texture()
            .expect("Failed to retrieve rasterization");

        for x in 0..texture.width {
            for y in 0..texture.height {
                let pixel = texture.get_pixel(x, y);
                assert_eq!(0xFF0000FF, pixel);
            }
        }
    }
}
