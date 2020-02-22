use crate::backend::shape_pipeline::ShapeVertex;
use crate::{Color, DrawCommand, DrawList, DrawingOptions, DrawingResult, ShapeKind};
use std::sync::Arc;
use std::time::Duration;
use vulkano::buffer::{BufferAccess, BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder, DynamicState};
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract};
use vulkano::instance::{Instance, PhysicalDevice, QueueFamily};
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::swapchain::{
    AcquireError, PresentMode, Surface, SurfaceTransform, Swapchain, SwapchainAcquireFuture,
    SwapchainCreationError,
};
use vulkano::sync::GpuFuture;
use vulkano_win::VkSurfaceBuild;
use winit::{EventsLoop, Window, WindowBuilder};

pub(crate) struct Gpu {
    options: DrawingOptions,
    device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    surface: Arc<Surface<Window>>,
    swapchain: Arc<Swapchain<Window>>,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    frame_buffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
    shape_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
}

pub(crate) struct GpuFrame {
    dynamic_state: DynamicState,
    command_buffer_builder: AutoCommandBufferBuilder,
    shape_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    swapchain_acquisition: SwapchainAcquireFuture<Window>,
    target_index: usize,
    target_dimensions: [u32; 2],
}

pub(crate) struct SealedGpuFrame {
    commands: AutoCommandBuffer,
    swapchain_acquisition: SwapchainAcquireFuture<Window>,
    target_index: usize,
}

impl Gpu {
    pub fn initialize(event_loop: &EventsLoop, options: DrawingOptions) -> DrawingResult<Gpu> {
        let instance = Instance::new(None, &vulkano_win::required_extensions(), None)
            .map_err(|_| "Failed to create Vulkan instance")?;
        let physical_device = PhysicalDevice::enumerate(&instance)
            .next()
            .ok_or("Failed to find a PhysicalDevice")?;
        let surface = WindowBuilder::new()
            .with_dimensions((options.width as u32, options.height as u32).into())
            .build_vk_surface(event_loop, instance.clone())
            .expect("Failed to build_vk_surface");
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

        let (swapchain, images) = {
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

        let format = swapchain.format();
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

        let frame_buffers: Vec<_> = images
            .iter()
            .map(|image| {
                Arc::new(
                    Framebuffer::start(render_pass.clone())
                        .add(image.clone())
                        .expect("Failed to add image to FrameBuffer")
                        .build()
                        .expect("Failed to build FrameBuffer"),
                ) as Arc<dyn FramebufferAbstract + Send + Sync>
            })
            .collect();

        let shape_pipeline = shape_pipeline::create_pipeline(device.clone(), render_pass.clone());

        Ok(Gpu {
            options,
            device,
            graphics_queue,
            surface,
            swapchain,
            render_pass,
            frame_buffers,
            shape_pipeline,
        })
    }

    pub fn begin_frame(&mut self, mut force_recreate: bool) -> GpuFrame {
        while force_recreate {
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
            let frame_buffers: Vec<_> = new_images
                .iter()
                .map(|image| {
                    Arc::new(
                        Framebuffer::start(self.render_pass.clone())
                            .add(image.clone())
                            .expect("Failed to add image to FrameBuffer")
                            .build()
                            .expect("Failed to build FrameBuffer"),
                    ) as Arc<dyn FramebufferAbstract + Send + Sync>
                })
                .collect();
            self.frame_buffers = frame_buffers;
            force_recreate = false;
        }

        let (image_index, acquire_future) =
            match vulkano::swapchain::acquire_next_image(self.swapchain.clone(), None) {
                Ok(tuple) => tuple,
                Err(AcquireError::OutOfDate) => {
                    unimplemented!("Not handling swapchain rebuilding yet");
                }
                Err(error) => panic!("{:?}", error),
            };

        let hi_dpi_factor = self.surface.window().get_hidpi_factor() as f32;
        let viewports = Viewport {
            origin: [0.0, 0.0],
            dimensions: [
                self.options.width as f32 * hi_dpi_factor,
                self.options.height as f32 * hi_dpi_factor,
            ],
            depth_range: 0.0..1.0,
        };

        let dynamic_state = DynamicState {
            line_width: None,
            viewports: Some(vec![viewports]),
            reference: None,
            compare_mask: None,
            scissors: None,
            write_mask: None,
        };

        GpuFrame::new(
            self.device.clone(),
            self.graphics_queue.family(),
            self.frame_buffers[image_index].clone(),
            [self.options.width as u32, self.options.height as u32],
            image_index,
            dynamic_state,
            self.shape_pipeline.clone(),
            acquire_future,
        )
    }

    pub fn end_frame(&mut self, sealed_gpu_frame: SealedGpuFrame) {
        sealed_gpu_frame
            .swapchain_acquisition
            .then_execute(self.graphics_queue.clone(), sealed_gpu_frame.commands)
            .expect("Failed then_execute")
            .then_swapchain_present(
                self.graphics_queue.clone(),
                self.swapchain.clone(),
                sealed_gpu_frame.target_index,
            )
            .then_signal_fence_and_flush()
            .expect("Failed to then_signal_fence_and_flush")
            .wait(Some(Duration::from_millis(5000)))
            .expect("Failed to wait");
    }
}

impl GpuFrame {
    pub fn new(
        device: Arc<Device>,
        queue_family: QueueFamily,
        target: Arc<dyn FramebufferAbstract + Send + Sync>,
        target_dimensions: [u32; 2],
        target_index: usize,
        dynamic_state: DynamicState,
        shape_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        swapchain_acquisition: SwapchainAcquireFuture<Window>,
    ) -> GpuFrame {
        let command_buffer_builder =
            AutoCommandBufferBuilder::primary_one_time_submit(device, queue_family)
                .expect("Failed to create command buffer")
                .begin_render_pass(target, false, vec![[0.0, 0.0, 1.0, 1.0].into()])
                .expect("Failed to begin render pass");

        GpuFrame {
            command_buffer_builder,
            dynamic_state,
            shape_pipeline,
            swapchain_acquisition,
            target_index,
            target_dimensions,
        }
    }

    pub fn build_command_buffer(mut self, draw_list: &DrawList) -> DrawingResult<SealedGpuFrame> {
        let mut iterator = draw_list.commands.iter();
        let target_width = self.target_dimensions[0] as f32;
        let target_height = self.target_dimensions[1] as f32;

        loop {
            if let Some(head) = iterator.next() {
                match head {
                    DrawCommand::Shape {
                        brush,
                        extent,
                        kind,
                    } => {
                        let left = extent.x as f32 / target_width * 2.0 - 1.0;
                        let right =
                            (extent.x + extent.width as i64) as f32 / target_width * 2.0 - 1.0;
                        let top = extent.y as f32 / target_height * 2.0 - 1.0;
                        let bottom =
                            (extent.y + extent.height as i64) as f32 / target_height * 2.0 - 1.0;
                        let kind_input = match kind {
                            ShapeKind::Rectangle => 1,
                            ShapeKind::Ellipse => 2,
                            ShapeKind::Line => 3,
                        };

                        let vertices: Vec<ShapeVertex> = vec![
                            ShapeVertex {
                                position: [left, top],
                                uv_input: [-1.0, -1.0],
                                kind_input,
                                foreground_input: self.to_color(brush.foreground),
                                background_input: self.to_color(brush.background),
                            },
                            ShapeVertex {
                                position: [right, top],
                                uv_input: [1.0, -1.0],
                                kind_input,
                                foreground_input: self.to_color(brush.foreground),
                                background_input: self.to_color(brush.background),
                            },
                            ShapeVertex {
                                position: [left, bottom],
                                uv_input: [-1.0, 1.0],
                                kind_input,
                                foreground_input: self.to_color(brush.foreground),
                                background_input: self.to_color(brush.background),
                            },
                            ShapeVertex {
                                position: [right, top],
                                uv_input: [1.0, -1.0],
                                kind_input,
                                foreground_input: self.to_color(brush.foreground),
                                background_input: self.to_color(brush.background),
                            },
                            ShapeVertex {
                                position: [right, bottom],
                                uv_input: [1.0, 1.0],
                                kind_input,
                                foreground_input: self.to_color(brush.foreground),
                                background_input: self.to_color(brush.background),
                            },
                            ShapeVertex {
                                position: [left, bottom],
                                uv_input: [-1.0, 1.0],
                                kind_input,
                                foreground_input: self.to_color(brush.foreground),
                                background_input: self.to_color(brush.background),
                            },
                        ];

                        let vertex_buffer: Vec<Arc<dyn BufferAccess + Send + Sync + 'static>> = {
                            vec![CpuAccessibleBuffer::from_iter(
                                self.shape_pipeline.device().clone(),
                                BufferUsage::all(),
                                vertices.into_iter(),
                            )
                            .unwrap()]
                        };

                        self.command_buffer_builder = self
                            .command_buffer_builder
                            .draw(
                                self.shape_pipeline.clone(),
                                &self.dynamic_state,
                                vertex_buffer,
                                (),
                                (),
                            )
                            .expect("Failed to draw shapes");
                    }
                    _ => {
                        println!("Bailed");
                        break;
                    }
                }
            } else {
                break;
            }
        }

        let command_buffer = self
            .command_buffer_builder
            .end_render_pass()
            .expect("Failed to end_render_pass")
            .build()
            .unwrap();

        Ok(SealedGpuFrame::new(
            command_buffer,
            self.swapchain_acquisition,
            self.target_index,
        ))
    }

    fn to_color(&self, color: Color) -> [f32; 4] {
        let red = ((color & 0xFF000000) >> 24) as f32 / 255.0;
        let green = ((color & 0x00FF0000) >> 16) as f32 / 255.0;
        let blue = ((color & 0x0000FF00) >> 8) as f32 / 255.0;
        let alpha = (color & 0x000000FF) as f32 / 255.0;
        return [red, green, blue, alpha];
    }
}

impl SealedGpuFrame {
    pub fn new(
        commands: AutoCommandBuffer,
        swapchain_acquisition: SwapchainAcquireFuture<Window>,
        target_index: usize,
    ) -> Self {
        SealedGpuFrame {
            commands,
            swapchain_acquisition,
            target_index,
        }
    }
}

mod shape_pipeline;
