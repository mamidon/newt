use crate::newt_render::Renderer;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use vulkano::format::Format;
use vulkano::image::{Dimensions, ImmutableImage};
use vulkano::sync::{now, GpuFuture};

pub trait OntoGpu<G> {
    fn onto_gpu(&self, renderer: &Renderer) -> G;
}

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct AttachmentHandle(usize);

pub struct AttachmentCollection<G> {
    next_handle: AttachmentHandle,
    attachments: HashMap<AttachmentHandle, G>,
}

pub struct HostSurface {
    bytes: Vec<u8>,
    width: u32,
    height: u32,
}

#[derive(Clone)]
pub struct GpuSurface {
    texture: Arc<ImmutableImage<Format>>,
}

impl AttachmentHandle {
    fn first_handle() -> AttachmentHandle {
        AttachmentHandle(0)
    }

    fn next_handle(&self) -> AttachmentHandle {
        AttachmentHandle(self.0 + 1)
    }
}

impl<G> AttachmentCollection<G> {
    pub fn new() -> AttachmentCollection<G> {
        AttachmentCollection {
            next_handle: AttachmentHandle::first_handle(),
            attachments: HashMap::new(),
        }
    }

    pub fn load(&mut self, gpu_attachment: G) -> AttachmentHandle {
        let attachment_handle = self.next_handle;
        self.next_handle = self.next_handle.next_handle();

        self.attachments.insert(attachment_handle, gpu_attachment);

        attachment_handle
    }

    pub fn retrieve(&self, handle: AttachmentHandle) -> Option<&G> {
        self.attachments.get(&handle)
    }
}

impl HostSurface {
    pub fn new(bytes: &Vec<u8>, width: u32, height: u32) -> HostSurface {
        HostSurface {
            bytes: bytes.iter().cloned().collect(),
            width,
            height,
        }
    }
}

impl OntoGpu<GpuSurface> for HostSurface {
    fn onto_gpu(&self, renderer: &Renderer) -> GpuSurface {
        let dimensions = Dimensions::Dim2d {
            width: self.width,
            height: self.height,
        };

        let (gpu_surface, loading_future) = ImmutableImage::from_iter(
            self.bytes.iter().cloned(),
            dimensions,
            Format::R8G8B8A8Srgb,
            renderer.graphics_queue.clone(),
        )
        .unwrap();

        loading_future.join(now(renderer.logical_device.clone()));

        GpuSurface {
            texture: gpu_surface,
        }
    }
}

impl GpuSurface {
    pub fn texture(&self) -> Arc<ImmutableImage<Format>> {
        self.texture.clone()
    }
}
