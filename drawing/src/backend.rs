use crate::{DrawingResult, TextureGreyScale, TextureRGBA};

pub(crate) struct Gpu;

impl Gpu {
    pub fn initialize() -> DrawingResult<Gpu> {
        unimplemented!()
    }

    pub fn load_texture_rgba(&self, texture: &TextureRGBA) -> DrawingResult<()> {
        unimplemented!()
    }

    pub fn load_texture_greyscale(&self, texture: &TextureGreyScale) -> DrawingResult<()> {
        unimplemented!()
    }
}
