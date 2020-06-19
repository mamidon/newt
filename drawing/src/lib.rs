use crate::backend::{Gpu, SealedGpuFrame};
use crate::resource_table::ResourceTable;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use winit::EventsLoop;

mod backend;
mod resource_table;

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

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Extent {
    x: i64,
    y: i64,
    width: u32,
    height: u32,
}

pub struct Corners {
    corner: usize,
    extent: Extent,
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

    pub fn corners(&self) -> Corners {
        Corners {
            corner: 0,
            extent: *self,
        }
    }

    pub fn logical_device_coordinates(index: usize) -> [f32; 2] {
        let left = -1.0;
        let right = 1.0;
        let top = -1.0;
        let bottom = 1.0;

        let corner = match index {
            0 => Some([left, top]),
            1 => Some([right, top]),
            2 => Some([left, bottom]),
            3 => Some([right, top]),
            4 => Some([right, bottom]),
            5 => Some([left, bottom]),
            _ => None,
        };

        corner.expect("I should probably create an enum for this")
    }

    pub fn uv_coordinates(index: usize) -> [f32; 2] {
        let left = 0.0;
        let right = 1.0;
        let top = 0.0;
        let bottom = 1.0;

        let corner = match index {
            0 => Some([left, top]),
            1 => Some([right, top]),
            2 => Some([left, bottom]),
            3 => Some([right, top]),
            4 => Some([right, bottom]),
            5 => Some([left, bottom]),
            _ => None,
        };

        corner.expect("I should probably create an enum for this")
    }
}

impl Iterator for Corners {
    type Item = [i64; 2];

    fn next(&mut self) -> Option<Self::Item> {
        let left = self.extent.x;
        let right = self.extent.x + self.extent.width as i64;
        let top = self.extent.y;
        let bottom = self.extent.y + self.extent.height as i64;

        let corner = self.corner;
        self.corner += 1;

        match corner {
            0 => Some([left, top]),
            1 => Some([right, top]),
            2 => Some([left, bottom]),
            3 => Some([right, top]),
            4 => Some([right, bottom]),
            5 => Some([left, bottom]),
            _ => None,
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
    resource_table: Arc<ResourceTable>,
}

#[derive(Copy, Clone)]
pub struct DrawingOptions {
    pub width: usize,
    pub height: usize,
}
pub struct MaterialCollection;
pub struct DrawList {
    // TODO stack of transforms, which manipulate commands as they come into the draw list
    // TODO z-ordering
    // TODO incorporate text rendering into Drawing, such that end users do not have to handle glyph texture management
    /* TODO consider how layout intersects with Drawing...
            ...perhaps define a Drawable & Layoutable traits; implementors of the former can supply DrawCommands
            ...the latter can supply a tree of LayoutItems (which in turn can supply Drawables)
            ...Layout items push transformations onto a transforms stack inside of the draw list..
    */
    shapes: Vec<ShapeDrawData>,
    glyphs: HashMap<DrawCommandKind, Vec<GlyphDrawData>>,
    masks: HashMap<DrawCommandKind, Vec<MaskDrawData>>,
}

impl Drawing {
    pub fn initialize(event_loop: &EventsLoop, options: DrawingOptions) -> DrawingResult<Self> {
        let resource_table = Arc::new(ResourceTable::new());
        let backend_gpu = Gpu::initialize(event_loop, resource_table.clone(), options)?;

        Ok(Drawing {
            backend_gpu,
            resource_table,
        })
    }

    pub fn create_draw_list(&self) -> DrawingResult<DrawList> {
        Ok(DrawList::empty())
    }

    pub fn submit_draw_list(&mut self, draw_list: &DrawList, force_recreate: bool) {
        let frame = self.backend_gpu.begin_frame(force_recreate);

        let sealed_frame = frame
            .build_command_buffer(&draw_list)
            .expect("Failed to build_command_buffer");

        self.backend_gpu.end_frame(sealed_frame);
    }

    pub fn load_rgba_texture(
        &mut self,
        width: u32,
        height: u32,
        bytes: &[u8],
    ) -> DrawingResult<SurfaceId> {
        let gpu_surface = self.backend_gpu.load_surface(width, height, bytes)?;
        let handle = self.resource_table.register_surface(gpu_surface.clone());

        Ok(handle)
    }

    pub fn load_mask_texture(
        &mut self,
        width: u32,
        height: u32,
        bytes: &[u8],
    ) -> DrawingResult<MaskId> {
        let gpu_mask = self.backend_gpu.load_mask(width, height, bytes)?;
        let handle = self.resource_table.register_mask(gpu_mask.clone());

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

    pub fn push_list(&mut self, other: DrawList) {
        self.shapes.extend(other.shapes);
        self.glyphs.extend(other.glyphs);
        self.masks.extend(other.masks);
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
