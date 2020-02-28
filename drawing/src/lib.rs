use crate::backend::{Gpu, GpuSurface, SealedGpuFrame};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use vulkano::image::Dimensions;
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

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Hash)]
pub struct Handle {
    generation: usize,
    key: usize,
}

pub type SurfaceId = Handle;
pub type MaskId = Handle;

impl Handle {
    pub fn new(start_from: usize) -> Handle {
        Handle {
            generation: 0,
            key: start_from,
        }
    }

    pub fn next(&mut self) -> Handle {
        let next = Handle {
            generation: self.generation,
            key: self.key,
        };

        self.key += 1;

        next
    }
}

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

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Hash)]
enum DrawCommandKind {
    Shape,
    Glyph(SurfaceId),
    Mask(MaskId),
}

struct ShapeDrawData {
    kind: ShapeKind,
    brush: Brush,
    extent: Extent,
}

struct GlyphDrawData {
    extent: Extent,
}

struct MaskDrawData {
    brush: Brush,
    extent: Extent,
}

pub struct Drawing {
    backend_gpu: Gpu,
    resources_table: DrawingResourcesTable,
}

pub(crate) struct DrawingResourcesTable {
    key_source: Handle,
    surfaces: HashMap<SurfaceId, GpuSurface>,
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
    shapes: Vec<ShapeDrawData>,
    glyphs: HashMap<DrawCommandKind, Vec<GlyphDrawData>>,
    masks: HashMap<DrawCommandKind, Vec<MaskDrawData>>,
}

impl Drawing {
    pub fn initialize(event_loop: &EventsLoop, _options: DrawingOptions) -> DrawingResult<Self> {
        Ok(Drawing {
            backend_gpu: Gpu::initialize(event_loop, _options)?,
            resources_table: DrawingResourcesTable::new(),
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

    pub fn load_rgba_texture(
        &mut self,
        width: u32,
        height: u32,
        bytes: &[u8],
    ) -> DrawingResult<SurfaceId> {
        let gpu_surface = self.backend_gpu.load_surface(width, height, bytes)?;
        let handle = self.resources_table.register_surface(gpu_surface.clone());

        Ok(handle)
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

impl DrawingResourcesTable {
    pub fn new() -> DrawingResourcesTable {
        DrawingResourcesTable {
            key_source: Handle::new(0),
            surfaces: HashMap::new(),
        }
    }

    pub fn register_surface(&mut self, surface: GpuSurface) -> SurfaceId {
        let key = self.key_source.next();
        self.surfaces.entry(key).or_insert(surface.clone());
        key
    }
}

impl MaterialCollection {
    pub fn new() -> DrawingResult<MaterialCollection> {
        Ok(MaterialCollection {})
    }

    pub fn create_material_glyph(
        &mut self,
        _material: impl Into<TextureRGBA>,
    ) -> DrawingResult<SurfaceId> {
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
            shapes: Vec::new(),
            glyphs: HashMap::new(),
            masks: HashMap::new(),
        }
    }

    pub fn push_shape(&mut self, kind: ShapeKind, brush: Brush, extent: Extent) {
        self.shapes.push(ShapeDrawData {
            kind,
            brush,
            extent,
        });
    }

    pub fn push_glyph(&mut self, surface: SurfaceId, extent: Extent) {
        let key = DrawCommandKind::Glyph(surface);
        let data = GlyphDrawData { extent };
        self.glyphs.entry(key).or_insert(Vec::new()).push(data);
    }

    pub fn push_mask(&mut self, mask: MaskId, brush: Brush, extent: Extent) {
        let key = DrawCommandKind::Mask(mask);
        let data = MaskDrawData { brush, extent };
        self.masks.entry(key).or_insert(Vec::new()).push(data);
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
