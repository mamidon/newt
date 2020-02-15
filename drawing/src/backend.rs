use crate::{DrawingOptions, DrawingResult, TextureGreyScale, TextureRGBA};
use std::sync::Arc;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::image::Dimensions::Dim2d;
use vulkano::image::{ImageViewAccess, StorageImage};
use vulkano::instance::{Instance, PhysicalDevice, QueueFamily};

pub(crate) struct Gpu {
    options: DrawingOptions,
    instance: Arc<Instance>,
    device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    target_buffer: Arc<dyn ImageViewAccess + Send + Sync>,
}

pub(crate) struct GpuFrame {
    command_buffer_builder: AutoCommandBufferBuilder,
}

impl Gpu {
    pub fn initialize(options: DrawingOptions) -> DrawingResult<Gpu> {
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

        let format = vulkano::format::Format::R8G8B8A8Srgb;
        let render_pass = Arc::new(
            vulkano::single_pass_renderpass!(
                device.clone(),
                attachments: {
                    target: {
                        load: Clear,
                        store: Store,
                        format: format,
                        samples: 1,
                    }
                },
                pass: {
                    color: [target],
                    depth_stencil: {}
                }
            )
            .expect("Failed to create top level RenderPass"),
        );

        let target_buffer = StorageImage::new(
            device.clone(),
            Dim2d {
                width: options.width as u32,
                height: options.height as u32,
            },
            format,
            (&[graphics_queue.family()]).iter().cloned(),
        )
        .expect("Failed to create rasterization target buffer");

        Ok(Gpu {
            options,
            instance,
            device,
            graphics_queue,
            render_pass,
            target_buffer,
        })
    }

    pub fn begin_rasterizing(&self) -> GpuFrame {
        GpuFrame::new(
            self.device.clone(),
            self.graphics_queue.family(),
            self.target_buffer.clone(),
        )
    }

    pub fn load_texture_rgba(&self, texture: &TextureRGBA) -> DrawingResult<()> {
        unimplemented!()
    }

    pub fn load_texture_greyscale(&self, texture: &TextureGreyScale) -> DrawingResult<()> {
        unimplemented!()
    }
}

impl GpuFrame {
    pub fn new(
        device: Arc<Device>,
        queue_family: QueueFamily,
        target: Arc<dyn ImageViewAccess + Sync + Send>,
    ) -> GpuFrame {
        let command_buffer_builder =
            AutoCommandBufferBuilder::primary_one_time_submit(device, queue_family)
                .expect("Failed to create command buffer")
                .begin_render_pass(target, false, vec![[0.0, 0.0, 1.0, 1.0].into()])
                .expect("Failed to begin render pass");

        GpuFrame {
            command_buffer_builder,
        }
    }
}
