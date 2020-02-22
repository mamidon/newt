use crate::backend::{Gpu, SealedGpuFrame};
use winit::EventsLoop;

mod backend;

pub type DrawingError = &'static str;
pub type DrawingResult<T> = Result<T, DrawingError>;

pub type Color = u32;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Brush {
    pub foreground: Color,
    pub background: Color,
}
pub struct TextureRGBA {
    width: usize,
    bytes: Vec<u8>,
}
pub struct TextureGreyScale {}

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

impl Extent {
    pub fn new(x: i64, y: i64, width: u32, height: u32) -> Extent {
        Extent {
            x,
            y,
            width,
            height,
        }
    }
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
    backend_gpu: Gpu,
}

#[derive(Copy, Clone)]
pub struct DrawingOptions {
    pub width: usize,
    pub height: usize,
}
pub struct SealedDrawList {
    sealed_frame: SealedGpuFrame,
}
pub struct MaterialCollection;
pub struct DrawList {
    commands: Vec<DrawCommand>,
}

impl Drawing {
    pub fn initialize(event_loop: &EventsLoop, _options: DrawingOptions) -> DrawingResult<Self> {
        Ok(Drawing {
            backend_gpu: Gpu::initialize(event_loop, _options)?,
        })
    }

    pub fn create_draw_list(&self) -> DrawingResult<DrawList> {
        Ok(DrawList::empty())
    }

    pub fn seal_draw_list(
        &mut self,
        draw_list: DrawList,
        force_recreate: bool,
    ) -> DrawingResult<SealedDrawList> {
        let frame = self.backend_gpu.begin_frame(force_recreate);

        let sealed_frame = frame
            .build_command_buffer(&draw_list)
            .expect("Failed to build_command_buffer");

        Ok(SealedDrawList::new(sealed_frame))
    }

    pub fn submit_sealed_draw_list(&mut self, sealed_draw_list: SealedDrawList) {
        self.backend_gpu.end_frame(sealed_draw_list.sealed_frame);
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

impl SealedDrawList {
    pub(crate) fn new(sealed_frame: SealedGpuFrame) -> Self {
        SealedDrawList { sealed_frame }
    }

    pub fn present_to_screen(&mut self) -> DrawingResult<()> {
        unimplemented!()
    }

    pub fn present_to_texture(&mut self) -> DrawingResult<TextureRGBA> {
        unimplemented!()
    }
}

impl TextureRGBA {
    pub fn new(width: usize, _height: usize, bytes: Vec<u8>) -> TextureRGBA {
        TextureRGBA { width, bytes }
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
    use winit::EventsLoop;

    struct Harness {
        pub drawing: Drawing,
    }

    impl Harness {
        fn setup() -> Harness {
            Harness {
                drawing: Drawing::initialize(&EventsLoop::new(), DrawingOptions::default())
                    .expect("Drawing failed to initialize for tests"),
            }
        }
    }

    #[test]
    fn example() {
        let mut harness = Harness::setup();
        println!("create_draw_list");
        let mut draw_list = harness
            .drawing
            .create_draw_list()
            .expect("Failed to create draw list");
        println!("push");
        draw_list.push(DrawCommand::Shape {
            kind: ShapeKind::Ellipse,
            brush: Brush {
                foreground: 0xFF0000FF,
                background: 0x00FF00FF,
            },
            extent: (64, 64).into(),
        });
        println!("seal");

        let sealed_draw_list = harness
            .drawing
            .seal_draw_list(draw_list, false)
            .expect("Failed to seal draw list");
        println!("submit");
        harness.drawing.submit_sealed_draw_list(sealed_draw_list);
        /*
        let texture = rasterization
            .present_to_texture()
            .expect("Failed to retrieve rasterization");

        for x in 0..texture.width {
            for y in 0..texture.height {
                let pixel = texture.get_pixel(x, y);
                assert_eq!(0xFF0000FF, pixel);
            }
        }
        */
    }
}
