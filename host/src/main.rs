#![allow(unused)]

extern crate vulkano;
extern crate vulkano_win;

use vulkano::instance::{Instance, PhysicalDevice, QueueFamily};
use winit::{EventsLoop, WindowBuilder, Window};
use vulkano_win::VkSurfaceBuild;
use vulkano::device::{DeviceExtensions, Device, DeviceCreationError};
use std::sync::Arc;
use vulkano::swapchain::Surface;
use std::fmt::{Display, Formatter, Error};

struct VulkanStateBuilder<'instance> {
    instance: Arc<Instance>,
    devices: Vec<PhysicalDevice<'instance>>,
    queue_families: Option<Vec<QueueFamily<'instance>>>,
    selected_device: Option<PhysicalDevice<'instance>>,
    selected_queue_family: Option<QueueFamily<'instance>>,
    surface: Option<Arc<Surface<Window>>>,
    logical_device: Option<Arc<Device>>
}

impl<'instance> VulkanStateBuilder<'instance> {
    pub fn new(instance: &'instance Arc<Instance>) -> VulkanStateBuilder<'instance> {
        VulkanStateBuilder {
            instance: instance.clone(),
            devices: PhysicalDevice::enumerate(instance).collect(),
            queue_families: None,
            surface: None,
            selected_device: None,
            selected_queue_family: None,
            logical_device: None
        }
    }

    pub fn pick_device<P>(&mut self, predicate: P)
        where P: Fn(&PhysicalDevice) -> bool
    {
        self.selected_device = self.devices.iter()
            .find(|device| predicate(device))
            .map(|device| device.clone());

        if let Some(device) = self.selected_device {
            self.queue_families = Some(device.queue_families().collect());
        }
    }

    pub fn pick_queue_family<P>(&mut self, predicate: P)
        where P: Fn(&QueueFamily) -> bool
    {
        self.selected_queue_family = self.queue_families
            .as_ref()
            .and_then(|families| families.iter().find(|family| predicate(family)))
            .map(|family| family.clone());
    }

    pub fn initialize_logical_device(&mut self) -> Result<(), DeviceCreationError>{
        let device = self.selected_device
            .or_else(self.pick_device(|d| true))
            .expect("Couldn't find a physical device!");
        let queue_family = self.selected_queue_family
            .or_else(self.pick_queue_family(|qf| true))
            .expect("Couldn't find a queue family!");

        let required_extensions = DeviceExtensions { khr_swapchain: true, .. DeviceExtensions::none() };
        self.logical_device = Some(Device::new(device,
                    device.supported_features(),
                    &required_extensions,
                    [(queue_family, 0.5)].iter().cloned())?.0);

        Ok(())
    }
}

impl<'instance> Display for VulkanStateBuilder<'instance> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        writeln!(f, "Instance: {:?}", self.instance)?;
        writeln!(f)?;

        writeln!(f, "Loaded Extensions: {:?}", self.instance.loaded_extensions())?;

        if self.devices.is_empty() {
            writeln!(f, "Devices: None")?;
        } else {
            writeln!(f, "Devices:")?;
            for device in &self.devices {
                writeln!(f, "\t{:?}", device)?;
            }
        }

        writeln!(f, "Selected Device: {:?}", self.selected_device);
        writeln!(f)?;

        match &self.queue_families {
            Some(queue_families) => {
                writeln!(f, "Queue Families:");
                for queue_family in self.queue_families.as_ref().unwrap_or(&vec![]).iter() {
                    let id = queue_family.id();
                    let graphics = queue_family.supports_graphics();
                    let compute = queue_family.supports_compute();
                    let surface = /* TODO */
                    writeln!(f, "\tId: {}  Graphics?: {}  Compute?: {}  Window?: {}", queue_family)?;
                }
            }
            None => {
                writeln!(f, "Queue Families: None")?;
            }
        }

        writeln!(f, "Selected Queue Family: {:?}", self.selected_queue_family.map(|qf| qf.id()));
        writeln!(f)?;

        Ok(())
    }
}

fn main() {
    let instance = Instance::new(
        None,
        &vulkano_win::required_extensions(),
        None).unwrap();
    let mut builder = VulkanStateBuilder::new(&instance);

    let events_loop = EventsLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&events_loop, instance.clone())
        .unwrap();

    builder.pick_device(|_| true);
    builder.pick_queue_family(|family| {
        println!("queue: {} {}", family.supports_graphics(), surface.is_supported(*family).unwrap_or(false));

        family.supports_graphics() && surface.is_supported(*family).unwrap_or(false)
    });
    builder.initialize_logical_device();

    println!("{}", builder);
}
