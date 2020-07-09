use crate::backend::GpuSurface;
use crate::{Handle, MaskId, SurfaceId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

struct GpuInnerResourceTable {
    key_source: Handle,
    surfaces: HashMap<SurfaceId, GpuSurface>,
    glyph_map: HashMap<u32, MaskId>,
}

impl GpuInnerResourceTable {
    fn new() -> GpuInnerResourceTable {
        GpuInnerResourceTable {
            key_source: Handle::new(0),
            surfaces: HashMap::new(),
            glyph_map: HashMap::new(),
        }
    }

    fn register_surface(&mut self, surface: GpuSurface) -> SurfaceId {
        let key = self.key_source.next();
        self.surfaces.entry(key).or_insert(surface.clone());
        key
    }

    fn register_mask(&mut self, surface: GpuSurface) -> MaskId {
        let key = self.key_source.next();
        self.surfaces.entry(key).or_insert(surface.clone());
        key
    }

    fn register_glyph(&mut self, glyph_id: u32, surface: GpuSurface) {
        let mask_id = self.register_mask(surface);
        self.glyph_map.insert(glyph_id, mask_id);
    }

    fn get_surface(&self, surface_id: SurfaceId) -> GpuSurface {
        self.surfaces
            .get(&surface_id)
            .expect("Invalid SurfaceId provided to InnerResourceTable")
            .clone()
    }

    fn get_mask(&self, mask_id: MaskId) -> GpuSurface {
        self.surfaces
            .get(&mask_id)
            .expect("Invalid MaskId provided to InnerResourceTable")
            .clone()
    }

    fn get_mask_id_for_glyph(&self, glyph_id: u32) -> Option<MaskId> {
        self.glyph_map.get(&glyph_id).map(|mask_id| *mask_id)
    }
}

#[derive(Clone)]
pub(crate) struct GpuResourceTable {
    inner: Rc<RefCell<GpuInnerResourceTable>>,
}

impl GpuResourceTable {
    pub fn new() -> GpuResourceTable {
        GpuResourceTable {
            inner: Rc::new(RefCell::new(GpuInnerResourceTable::new())),
        }
    }

    pub fn register_surface(&self, surface: GpuSurface) -> SurfaceId {
        self.inner.borrow_mut().register_surface(surface)
    }

    pub fn register_mask(&self, surface: GpuSurface) -> MaskId {
        self.inner.borrow_mut().register_mask(surface)
    }

    pub fn register_glyph(&self, glyph_id: u32, surface: GpuSurface) {
        self.inner.borrow_mut().register_glyph(glyph_id, surface)
    }

    pub fn get_surface(&self, surface_id: SurfaceId) -> GpuSurface {
        self.inner.borrow().get_surface(surface_id)
    }

    pub fn get_mask(&self, mask_id: MaskId) -> GpuSurface {
        self.inner.borrow().get_mask(mask_id)
    }

    pub fn get_mask_id_for_glyph(&self, glyph_id: u32) -> Option<MaskId> {
        self.inner.borrow().get_mask_id_for_glyph(glyph_id)
    }
}
