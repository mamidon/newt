use crate::backend::GpuSurface;
use crate::{Handle, SurfaceId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

struct InnerResourceTable {
    key_source: Handle,
    surfaces: HashMap<SurfaceId, GpuSurface>,
}

impl InnerResourceTable {
    fn new() -> InnerResourceTable {
        InnerResourceTable {
            key_source: Handle::new(0),
            surfaces: HashMap::new(),
        }
    }

    fn register_surface(&mut self, surface: GpuSurface) -> SurfaceId {
        let key = self.key_source.next();
        self.surfaces.entry(key).or_insert(surface.clone());
        key
    }

    fn get_surface(&self, surface_id: SurfaceId) -> GpuSurface {
        self.surfaces
            .get(&surface_id)
            .expect("Invalid SurfaceId provided to InnerResourceTable")
            .clone()
    }
}

#[derive(Clone)]
pub(crate) struct ResourceTable {
    inner: Rc<RefCell<InnerResourceTable>>,
}

impl ResourceTable {
    pub fn new() -> ResourceTable {
        ResourceTable {
            inner: Rc::new(RefCell::new(InnerResourceTable::new())),
        }
    }

    pub fn register_surface(&self, surface: GpuSurface) -> SurfaceId {
        self.inner.borrow_mut().register_surface(surface)
    }

    pub fn get_surface(&self, surface_id: SurfaceId) -> GpuSurface {
        self.inner.borrow().get_surface(surface_id)
    }
}
