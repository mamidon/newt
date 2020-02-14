use crate::{DrawingResult, TextureGreyScale, TextureRGBA};
use std::sync::Arc;
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::instance::{Instance, PhysicalDevice};

pub(crate) struct Gpu {
    instance: Arc<Instance>,
    device: Arc<Device>,
    graphics_queue: Arc<Queue>,
}
impl Gpu {
    pub fn initialize() -> DrawingResult<Gpu> {
        let instance = Instance::new(None, &vulkano_win::required_extensions(), None)
            .map_err(|_| "Failed to create Vulkan instance")?;
        let physical_device = PhysicalDevice::enumerate(&instance)
            .next()
            .ok_or("Failed to find a PhysicalDevice")?;
        let queue_family = physical_device
            .queue_families()
            .find(|&qf| qf.supports_graphics())
            .ok_or("Failed to find supported QueueFamily")?;

        let required_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };
        let (device, mut queues) = Device::new(
            physical_device,
            physical_device.supported_features(),
            &required_extensions,
            [(queue_family, 0.5)].iter().cloned(),
        )
        .map_err(|_| "Failed to create a Vulkan Logical Device")?;
        let graphics_queue = queues
            .next()
            .ok_or("Did not receive a graphics queue with the Vulkan Logical Device")?;

        Ok(Gpu {
            instance,
            device,
            graphics_queue,
        })
    }

    pub fn load_texture_rgba(&self, texture: &TextureRGBA) -> DrawingResult<()> {
        unimplemented!()
    }

    pub fn load_texture_greyscale(&self, texture: &TextureGreyScale) -> DrawingResult<()> {
        unimplemented!()
    }
}
