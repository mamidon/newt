#![allow(unused)]

extern crate vulkano;
extern crate vulkano_win;

use vulkano::instance::{Instance, PhysicalDevice, QueueFamily};
use winit::{EventsLoop, WindowBuilder, Window};
use vulkano_win::VkSurfaceBuild;
use vulkano::device::{DeviceExtensions, Device, DeviceCreationError, QueuesIter};
use std::sync::Arc;
use vulkano::swapchain::Surface;
use std::fmt::{Display, Formatter, Error};

struct VulkanState<'instance> {
    instance: &'instance Arc<Instance>,
    surface: Arc<Surface<Window>>,
    physical_devices: Option<Vec<PhysicalDevice<'instance>>>,
    selected_physical_device: Option<PhysicalDevice<'instance>>,
}

impl<'instance> VulkanState<'instance> {
    pub fn new(instance: &'instance Arc<Instance>, surface: &Arc<Surface<Window>>) -> VulkanState<'instance> {
        VulkanState {
            instance,
            surface: surface.clone(),
            physical_devices: None,
            selected_physical_device: None
        }
    }
}

struct VulkanStateBuilder<'instance> {
    state: VulkanState<'instance>,
    device_selector: Option<fn(&PhysicalDevice) -> bool>,
    queue_family_selector: Option<fn(&QueueFamily, &Arc<Surface<Window>>) -> bool>,
    logical_device: Option<Arc<Device>>
}

impl<'instance> VulkanStateBuilder<'instance> {
    pub fn new(state: VulkanState<'instance>) -> VulkanStateBuilder {
        VulkanStateBuilder {
            state,
            device_selector: None,
            queue_family_selector: None,
            logical_device: None
        }
    }

    pub fn set_device_selector(&mut self, predicate: fn(&PhysicalDevice) -> bool)
    {
        self.device_selector = Some(predicate);
    }

    pub fn set_queue_family_selector(&mut self, predicate: fn(&QueueFamily, &Arc<Surface<Window>>) -> bool)
    {
        self.queue_family_selector = Some(predicate);
    }

    fn get_devices(&self) -> Vec<PhysicalDevice> {
        if let Some(devices) = self.state.physical_devices.as_ref() {
            return devices.clone();
        }

        let devices: Vec<_> = PhysicalDevice::enumerate(&self.state.instance).collect();
        self.state.physical_devices
            .replace(devices);

        self.state.physical_devices
            .expect("Given an Instance, polling the devices shouldn't fail")
            .clone()
    }

    fn get_device_selector(&self) -> &fn(&PhysicalDevice) -> bool {
        match self.device_selector.as_ref() {
            Some(selector) => selector,
            None => unimplemented!()
        }
    }

    fn get_queue_family_selector(&self) -> &fn(&QueueFamily, &Arc<Surface<Window>>) -> bool {
        match self.queue_family_selector.as_ref() {
            Some(selector) => selector,
            None => unimplemented!()
        }
    }

    fn get_selected_device(&self) -> Result<PhysicalDevice, String> {
        let selector = self.get_device_selector();

        self.get_devices()
            .into_iter()
            .find(|device| selector(device))
            .map(|device| device)
            .ok_or("PhysicalDevice not found".to_string())
    }

    fn get_queue_families(&self) -> Result<Vec<QueueFamily>, String> {
        let selected_device = self.get_selected_device()?;

        Ok(selected_device.queue_families().collect())
    }

    fn get_selected_queue_family(&self) -> Result<QueueFamily, String> {
        let queue_family_selector = self.get_queue_family_selector();
        let selected_queue_family = self.get_queue_families()?
            .into_iter()
            .find(|qf| queue_family_selector(qf, &self.state.surface));

        selected_queue_family
            .ok_or("Failed to select a QueueFamily".to_string())
    }

    pub fn get_logical_device(&self) -> Result<(Arc<Device>, QueuesIter), String>{
        let device = self.get_selected_device()?;
        let queue_family = self.get_selected_queue_family()?;

        let required_extensions = DeviceExtensions { khr_swapchain: true, .. DeviceExtensions::none() };
        let device_result = Device::new(device,
                    device.supported_features(),
                    &required_extensions,
                    [(queue_family, 0.5)].iter().cloned());

        device_result.ok()
            .ok_or("Failed to create a logical Device".to_string())
    }
}

fn main() {
    let instance = Instance::new(
        None,
        &vulkano_win::required_extensions(),
        None)
        .unwrap();

    let events_loop = EventsLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&events_loop, instance.clone())
        .unwrap();

    let mut state = VulkanState::new(&instance, &surface);
    let mut builder = VulkanStateBuilder::new(state);

    builder.set_device_selector(|_| true);
    builder.set_queue_family_selector(|family, surface|
        family.supports_graphics() && surface.is_supported(*family).unwrap_or(false)
    );
    let (logical_device, mut queues) = builder.get_logical_device().unwrap();
    let logical_queues: Vec<_> = queues.collect();

    println!("{:?}", logical_device);
    println!("{:?}", logical_queues);
}
