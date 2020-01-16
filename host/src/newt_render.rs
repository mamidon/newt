use vulkano::instance::{Instance, PhysicalDevice};
use std::sync::Arc;
use vulkano::swapchain::{Surface, Swapchain, PresentMode, SurfaceTransform, SwapchainCreationError, AcquireError};
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use winit::{Window, WindowBuilder, EventsLoop};
use vulkano::device::{Queue, DeviceExtensions, Device};
use vulkano_win::VkSurfaceBuild;
use vulkano::framebuffer::{Framebuffer, Subpass, RenderPassAbstract, FramebufferAbstract, RenderPass, RenderPassDesc};
use vulkano::command_buffer::{DynamicState, AutoCommandBufferBuilder};
use vulkano::sync::{now, GpuFuture, FlushError};
use vulkano::image::SwapchainImage;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::vertex::{VertexSource, SingleBufferDefinition};
use vulkano::buffer::{CpuAccessibleBuffer, BufferUsage, BufferAccess};
use vulkano::descriptor::PipelineLayoutAbstract;

#[derive(Default, Debug, Clone)]
struct Vertex { position: [f32; 2] }
vulkano::impl_vertex!(Vertex, position);


type ErrorMessage = &'static str;

pub struct Renderer {
    instance: Arc<Instance>,
    surface: Arc<Surface<Window>>,
    logical_device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    recreate_swapchain: bool,
    swapchain: Arc<Swapchain<Window>>,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync + 'static>,
    dynamic_state: DynamicState,
    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>
}

pub enum RenderCommand {
    Rectangle { x: usize, y: usize, width: usize, height: usize }
}

impl Renderer {
    pub fn initialize(events_loop: &EventsLoop) -> Result<Renderer, ErrorMessage> {
        let instance = Instance::new(
            None,
            &vulkano_win::required_extensions(),
            None)
            .map_err(|_| "Failed to creat Vulkan Instance")?;

        let surface = WindowBuilder::new()
            .with_dimensions((512,512).into())
            .build_vk_surface(&events_loop, instance.clone())
            .unwrap();

        let physical_device = PhysicalDevice::enumerate(&instance)
            .next()
            .ok_or("Failed to find PhysicalDevice")?;

        let queue_family = physical_device.queue_families()
            .find(|&qf| qf.supports_graphics() && surface.is_supported(qf).unwrap_or(false))
            .ok_or("Failed to find supported QueueFamily")?;

        let required_extensions = DeviceExtensions { khr_swapchain: true, .. DeviceExtensions::none() };
        let (device, mut queues) = Device::new(
            physical_device,
            physical_device.supported_features(),
            &required_extensions,
            [(queue_family, 0.5)].iter().cloned(),
        ).map_err(|_| "Failed to create a Vulkan Logical Device")?;

        let graphics_queue = queues.next()
            .ok_or("Did not receive a graphics queue with the Vulkan Logical Device")?;

        let (mut swapchain, images) = {
            let capabilities = surface.capabilities(physical_device).unwrap();
            let usage = capabilities.supported_usage_flags;
            let alpha = capabilities.supported_composite_alpha.iter().next().unwrap();
            let format = capabilities.supported_formats[0].0;
            let initial_dimensions = if let Some(dimensions) = surface.window().get_inner_size() {
                let physical_dimensions: (u32, u32) = dimensions.to_physical(surface.window().get_hidpi_factor()).into();
                [physical_dimensions.0, physical_dimensions.1]
            } else {
                return Err("Failed to get inner size of the render target");
            };

            Swapchain::new(device.clone(), surface.clone(), capabilities.min_image_count,
                           format, initial_dimensions, 1, usage, &graphics_queue, SurfaceTransform::Identity, alpha,
                           PresentMode::Fifo, true, None)
                .unwrap()
        };

        mod vertex_shader {
            vulkano_shaders::shader!{
        ty: "vertex",
        src: r#"
#version 450

layout(location = 0) in vec2 position;
layout(location = 0) out vec2 uv;
void main() {
gl_Position = vec4(position, 0.0, 1.0);
uv = position + vec2(0.5);
}
        "#
    }
        }

        mod fragment_shader {
            vulkano_shaders::shader!{
        ty: "fragment",
        src: r#"
#version 450

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 f_color;

void main() {
if (abs(uv.x) < 0.05 && abs(uv.y) < 0.05) {
    f_color = vec4(1.0, 0.0, 0.0, 1.0);
} else {
    f_color = vec4(0.0, 1.0, 0.0, 1.0);
}
}
        "#
    }
        }

        let vs = vertex_shader::Shader::load(device.clone()).unwrap();
        let fs = fragment_shader::Shader::load(device.clone()).unwrap();

        let render_pass = Arc::new(vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.format(),
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        ).map_err(|_| "Failed to create the RenderPass")?);

        let pipeline = Arc::new(GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vertex>()
            .vertex_shader(vs.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs.main_entry_point(), ())
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(device.clone())
            .unwrap());

        let mut dynamic_state = DynamicState {
            line_width: None,
            viewports: None,
            reference: None,
            compare_mask: None,
            scissors: None,
            write_mask: None
        };

        let mut framebuffers = window_size_dependent_setup(&images, render_pass.clone(), &mut dynamic_state);
        let mut recreate_swapchain = false;

        Ok(Renderer {
            instance,
            surface: surface.clone(),
            logical_device: device.clone(),
            graphics_queue,
            recreate_swapchain,
            swapchain,
            render_pass,
            dynamic_state,
            framebuffers,
            pipeline
        })
    }

    pub fn begin_frame(&mut self, force_recreate: bool) {
        self.recreate_swapchain = self.recreate_swapchain || force_recreate;

        while self.recreate_swapchain {
            // Get the new dimensions of the window.
            let dimensions = if let Some(dimensions) = self.surface.window().get_inner_size() {
                let dimensions: (u32, u32) = dimensions.to_physical(self.surface.window().get_hidpi_factor()).into();
                [dimensions.0, dimensions.1]
            } else {
                return;
            };

            let (new_swapchain, new_images) = match self.swapchain.recreate_with_dimension(dimensions) {
                Ok(r) => r,
                // This error tends to happen when the user is manually resizing the window.
                // Simply restarting the loop is the easiest way to fix this issue.
                Err(SwapchainCreationError::UnsupportedDimensions) => continue,
                Err(err) => panic!("{:?}", err)
            };

            self.swapchain = new_swapchain;
            // Because framebuffers contains an Arc on the old swapchain, we need to
            // recreate framebuffers as well.
            self.framebuffers = window_size_dependent_setup(&new_images, self.render_pass.clone(), &mut self.dynamic_state);

            self.recreate_swapchain = false;
        }
    }

    pub fn submit_commands<C: IntoIterator<Item=RenderCommand>>(&mut self, previous_frame: Option<Box<dyn GpuFuture>>, commands: C) -> Box<dyn GpuFuture> {
        let previous_frame_future = previous_frame.unwrap_or(Box::new(now(self.logical_device.clone())) as Box<dyn GpuFuture>);

        let (image_index, future) = match vulkano::swapchain::acquire_next_image(self.swapchain.clone(), None) {
            Ok(tuple) => tuple,
            Err(AcquireError::OutOfDate) => {
                self.recreate_swapchain = true;
                return previous_frame_future;
            },
            Err(error) => panic!("{:?}", error)
        };

        let clear_values = vec!([0.0, 0.0, 1.0, 1.0].into());
        let vertex_buffer: Vec<Arc<dyn BufferAccess + Send + Sync + 'static>> = {
            vec![CpuAccessibleBuffer::from_iter(self.logical_device.clone(), BufferUsage::all(), [
                Vertex { position: [-0.5, -0.25] },
                Vertex { position: [0.0, 0.5] },
                Vertex { position: [0.25, -0.1] }
            ].iter().cloned()).unwrap()]
        };

        let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(
            self.logical_device.clone(),
            self.graphics_queue.family())
            .unwrap()
            .begin_render_pass(self.framebuffers[image_index].clone(), false, clear_values)
            .unwrap()
            .draw(self.pipeline.clone(), &self.dynamic_state, vertex_buffer.clone(), (), ())
            .unwrap()
            .end_render_pass()
            .unwrap()
            .build()
            .unwrap();

        let future = previous_frame_future.join(future)
            .then_execute(self.graphics_queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(self.graphics_queue.clone(), self.swapchain.clone(), image_index)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                // This wait is required when using NVIDIA or running on macOS. See https://github.com/vulkano-rs/vulkano/issues/1247
                future.wait(None).unwrap();
                Box::new(future) as Box<_>
            }
            Err(FlushError::OutOfDate) => {
                self.recreate_swapchain = true;
                Box::new(now(self.logical_device.clone())) as Box<_>
            }
            Err(e) => {
                println!("{:?}", e);
                Box::new(now(self.logical_device.clone())) as Box<_>
            }
        }
    }

    pub fn present(&mut self) {}
}


fn window_size_dependent_setup(images: &[Arc<SwapchainImage<Window>>],
                               render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
                               dynamic_state: &mut DynamicState) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
    let dimensions = images[0].dimensions();

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0 .. 1.0,
    };
    dynamic_state.viewports = Some(vec!(viewport));

    images.iter().map(|image| {
        Arc::new(
            Framebuffer::start(render_pass.clone())
                .add(image.clone()).unwrap()
                .build().unwrap()
        ) as Arc<dyn FramebufferAbstract + Send + Sync>
    }).collect::<Vec<_>>()
}
