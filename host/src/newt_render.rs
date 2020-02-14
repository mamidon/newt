use std::sync::Arc;
use vulkano::buffer::{BufferAccess, BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBuffer, DynamicState};
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor::{DescriptorSet, PipelineLayoutAbstract};
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::format::{ClearValue, Format};
use vulkano::framebuffer::{
    Framebuffer, FramebufferAbstract, RenderPass, RenderPassAbstract, RenderPassDesc, Subpass,
};
use vulkano::image::{Dimensions, ImmutableImage, StorageImage, SwapchainImage};
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::vertex::{SingleBufferDefinition, VertexSource};
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::swapchain::{
    AcquireError, PresentMode, Surface, SurfaceTransform, Swapchain, SwapchainAcquireFuture,
    SwapchainCreationError,
};
use vulkano::sync::{now, FlushError, GpuFuture};
use vulkano_win::VkSurfaceBuild;
use winit::{EventsLoop, Window, WindowBuilder};

pub mod attachments;
mod pipelines;

use crate::newt_render::attachments::{
    AttachmentCollection, AttachmentHandle, GpuSurface, HostSurface, OntoGpu,
};
use crate::newt_render::pipelines::boxes::BoxPipeline;
use crate::newt_render::pipelines::glyphs::GlyphPipeline;
use crate::newt_render::pipelines::CommandBufferWritingInfo;
use std::hash::Hash;
use vulkano::sampler::{Filter, Sampler};

type ErrorMessage = &'static str;

pub struct Renderer {
    surface_attachments: AttachmentCollection<GpuSurface>,
    instance: Arc<Instance>,
    surface: Arc<Surface<Window>>,
    logical_device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    recreate_swapchain: bool,
    swapchain: Arc<Swapchain<Window>>,
    box_pipeline: BoxPipeline,
    glyph_pipeline: GlyphPipeline,
}

pub struct RendererFrame {
    submitted_commands: Vec<RenderCommand>,
    submission_future: Option<Box<dyn GpuFuture>>,
}

#[derive(Clone)]
pub enum RenderCommand {
    Rectangle {
        x: isize,
        y: isize,
        width: usize,
        height: usize,
    },
    NewtSurface {
        x: isize,
        y: isize,
        width: usize,
        height: usize,
        surface: AttachmentHandle,
    },
}

impl RendererFrame {
    pub fn initialize() -> RendererFrame {
        RendererFrame {
            submitted_commands: vec![],
            submission_future: None,
        }
    }

    pub fn submit_commands<C: IntoIterator<Item = RenderCommand>>(&mut self, commands: C) {
        for command in commands {
            self.submitted_commands.push(command);
        }
    }
}

impl Renderer {
    pub fn initialize(events_loop: &EventsLoop) -> Result<Renderer, ErrorMessage> {
        let instance = Instance::new(None, &vulkano_win::required_extensions(), None)
            .map_err(|_| "Failed to creat Vulkan Instance")?;

        let surface = WindowBuilder::new()
            .with_dimensions((512, 512).into())
            .build_vk_surface(&events_loop, instance.clone())
            .unwrap();

        let physical_device = PhysicalDevice::enumerate(&instance)
            .next()
            .ok_or("Failed to find PhysicalDevice")?;

        let queue_family = physical_device
            .queue_families()
            .find(|&qf| qf.supports_graphics() && surface.is_supported(qf).unwrap_or(false))
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

        let (mut swapchain, images) = {
            let capabilities = surface.capabilities(physical_device).unwrap();
            let usage = capabilities.supported_usage_flags;
            let alpha = capabilities
                .supported_composite_alpha
                .iter()
                .next()
                .unwrap();
            let format = capabilities.supported_formats[0].0;
            let initial_dimensions = if let Some(dimensions) = surface.window().get_inner_size() {
                let physical_dimensions: (u32, u32) = dimensions
                    .to_physical(surface.window().get_hidpi_factor())
                    .into();
                [physical_dimensions.0, physical_dimensions.1]
            } else {
                return Err("Failed to get inner size of the render target");
            };

            Swapchain::new(
                device.clone(),
                surface.clone(),
                capabilities.min_image_count,
                format,
                initial_dimensions,
                1,
                usage,
                &graphics_queue,
                SurfaceTransform::Identity,
                alpha,
                PresentMode::Fifo,
                true,
                None,
            )
            .unwrap()
        };

        let mut recreate_swapchain = false;

        Ok(Renderer {
            surface_attachments: AttachmentCollection::new(),
            instance,
            surface: surface.clone(),
            logical_device: device.clone(),
            graphics_queue,
            recreate_swapchain,
            swapchain: swapchain.clone(),
            box_pipeline: BoxPipeline::initialize(&device, swapchain.format(), &images)?,
            glyph_pipeline: GlyphPipeline::initialize(&device, swapchain.format(), &images)?,
        })
    }

    pub fn load_surface<S: Into<HostSurface>>(&mut self, surface: S) -> AttachmentHandle {
        let host_surface: HostSurface = surface.into();
        let gpu_surface = host_surface.onto_gpu(self);
        self.surface_attachments.load(gpu_surface)
    }

    fn get_gpu_surface(&self, handle: AttachmentHandle) -> GpuSurface {
        self.surface_attachments.retrieve(handle).unwrap().clone()
    }

    pub fn begin_frame(&mut self, force_recreate: bool) -> Result<RendererFrame, &'static str> {
        self.recreate_swapchain = self.recreate_swapchain || force_recreate;

        while self.recreate_swapchain {
            // Get the new dimensions of the window.
            let dimensions: (u32, u32) = self
                .surface
                .window()
                .get_inner_size()
                .expect("get_inner_size failed")
                .to_physical(self.surface.window().get_hidpi_factor())
                .into();

            let (new_swapchain, new_images) = match self
                .swapchain
                .recreate_with_dimension([dimensions.0, dimensions.1])
            {
                Ok(r) => r,
                // This error tends to happen when the user is manually resizing the window.
                // Simply restarting the loop is the easiest way to fix this issue.
                Err(SwapchainCreationError::UnsupportedDimensions) => continue,
                Err(err) => panic!("{:?}", err),
            };

            self.swapchain = new_swapchain;
            // Because framebuffers contains an Arc on the old swapchain, we need to
            // recreate framebuffers as well.
            self.box_pipeline.reinitialize(&new_images);
            self.glyph_pipeline.reinitialize(&new_images);

            self.recreate_swapchain = false;
        }

        Ok(RendererFrame::initialize())
    }

    pub fn present(&mut self, frame: RendererFrame) {
        let (image_index, acquire_future) =
            match vulkano::swapchain::acquire_next_image(self.swapchain.clone(), None) {
                Ok(tuple) => tuple,
                Err(AcquireError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return;
                }
                Err(error) => panic!("{:?}", error),
            };

        let logical_size = self.surface.window().get_inner_size().unwrap();
        let box_pipeline_writing_info = CommandBufferWritingInfo::initialize(
            frame.submitted_commands.iter(),
            self,
            image_index,
            logical_size.width,
            logical_size.height,
        );
        let box_pipeline_attachments = self
            .box_pipeline
            .create_attachments(&box_pipeline_writing_info);

        let mut command_buffer_builder = AutoCommandBufferBuilder::primary_one_time_submit(
            self.logical_device.clone(),
            self.graphics_queue.family(),
        )
        .unwrap()
        .begin_render_pass(
            self.box_pipeline.framebuffers[image_index].clone(),
            false,
            vec![[0.0, 0.0, 1.0, 1.0].into()],
        )
        .expect("begin_render_pass failed");

        command_buffer_builder = {
            self.box_pipeline
                .write_to_command_buffer(
                    &box_pipeline_writing_info,
                    command_buffer_builder,
                    box_pipeline_attachments,
                )
                .expect("write_to_command_buffer failed")
        };

        command_buffer_builder = {
            let glyph_pipeline_writing_info = CommandBufferWritingInfo::initialize(
                frame.submitted_commands.iter(),
                self,
                image_index,
                logical_size.width,
                logical_size.height,
            );
            let glyph_pipeline_attachments = self
                .glyph_pipeline
                .create_attachments(&glyph_pipeline_writing_info);

            self.glyph_pipeline
                .write_to_command_buffer(
                    &glyph_pipeline_writing_info,
                    command_buffer_builder,
                    glyph_pipeline_attachments,
                )
                .expect("write_to_command_buffer failed")
        };

        command_buffer_builder = command_buffer_builder
            .end_render_pass()
            .expect("end_render_pass failed");

        let command_buffer = command_buffer_builder.build().unwrap();

        let present_future = acquire_future
            .then_execute(self.graphics_queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                self.graphics_queue.clone(),
                self.swapchain.clone(),
                image_index,
            )
            .then_signal_fence_and_flush();

        match present_future {
            Ok(mut future) => {
                // This wait is required when using NVIDIA or running on macOS. See https://github.com/vulkano-rs/vulkano/issues/1247
                future.wait(None).unwrap();
                future.cleanup_finished();
            }
            Err(FlushError::OutOfDate) => {
                self.recreate_swapchain = true;
            }
            Err(e) => panic!("{}", e),
        }
    }
}
