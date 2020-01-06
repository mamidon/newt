#![allow(unused)]

extern crate vulkano;
extern crate vulkano_win;

use vulkano::instance::{Instance, PhysicalDevice, QueueFamily};
use winit::{EventsLoop, WindowBuilder, Window, Event, WindowEvent};
use vulkano_win::VkSurfaceBuild;
use vulkano::device::{DeviceExtensions, Device, DeviceCreationError, QueuesIter, Queue};
use std::sync::Arc;
use vulkano::swapchain::{Surface, Swapchain, SurfaceTransform, PresentMode, ColorSpace, AcquireError, SwapchainCreationError};
use std::fmt::{Display, Formatter, Error};
use std::cell::RefCell;
use vulkano::buffer::{CpuAccessibleBuffer, BufferUsage};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::framebuffer::{Framebuffer, Subpass, RenderPassAbstract, FramebufferAbstract};
use vulkano::command_buffer::{DynamicState, AutoCommandBufferBuilder};
use vulkano::image::SwapchainImage;
use vulkano::pipeline::viewport::Viewport;
use vulkano::sync::{now, GpuFuture, FlushError};

#[derive(Default, Debug, Clone)]
struct Vertex { position: [f32; 2] }
vulkano::impl_vertex!(Vertex, position);

fn main() {
    let instance = Instance::new(
        None,
        &vulkano_win::required_extensions(),
        None)
        .unwrap();

    let mut events_loop = EventsLoop::new();
    let surface = WindowBuilder::new()
        .with_dimensions((512,512).into())
        .build_vk_surface(&events_loop, instance.clone())
        .unwrap();

    let physical_device =
        PhysicalDevice::enumerate(&instance)
            .next()
            .unwrap();

    let queue_family = physical_device.queue_families()
        .find(|&qf| qf.supports_graphics() && surface.is_supported(qf).unwrap_or(false))
        .unwrap();

    let required_extensions = DeviceExtensions { khr_swapchain: true, .. DeviceExtensions::none() };
    let (device, mut queues) = Device::new(
        physical_device,
        physical_device.supported_features(),
        &required_extensions,
        [(queue_family, 0.5)].iter().cloned())
        .unwrap();
    let queue = queues.next().unwrap();

    let (mut swapchain, images) = {
        let capabilities = surface.capabilities(physical_device).unwrap();
        let usage = capabilities.supported_usage_flags;
        let alpha = capabilities.supported_composite_alpha.iter().next().unwrap();
        let format = capabilities.supported_formats[0].0;
        let initial_dimensions = if let Some(dimensions) = surface.window().get_inner_size() {
            let physical_dimensions: (u32, u32) = dimensions.to_physical(surface.window().get_hidpi_factor()).into();
            [physical_dimensions.0, physical_dimensions.1]
        } else {
            return;
        };

        Swapchain::new(device.clone(), surface.clone(), capabilities.min_image_count,
            format, initial_dimensions, 1, usage, &queue, SurfaceTransform::Identity, alpha,
        PresentMode::Fifo, true, None)
            .unwrap()
    };

    let vertex_buffer = {

        CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), [
            Vertex { position: [-1.0, -1.0] },
            Vertex { position: [0.0, -1.0] },
            Vertex { position: [0.0, 0.0] },

            Vertex { position: [0.0, 0.0] },
            Vertex { position: [-1.0, 0.0] },
            Vertex { position: [-1.0, -1.0] },
            ].iter().cloned()).unwrap()
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
    if (uv.x * uv.x + uv.y * uv.y < 0.05) {
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
    ).unwrap());

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
    let mut previous_frame = Box::new(now(device.clone())) as Box<dyn GpuFuture>;

    loop {
        previous_frame.cleanup_finished();

        if recreate_swapchain {
            // Get the new dimensions of the window.
            let dimensions = if let Some(dimensions) = surface.window().get_inner_size() {
                let dimensions: (u32, u32) = dimensions.to_physical(surface.window().get_hidpi_factor()).into();
                [dimensions.0, dimensions.1]
            } else {
                return;
            };

            let (new_swapchain, new_images) = match swapchain.recreate_with_dimension(dimensions) {
                Ok(r) => r,
                // This error tends to happen when the user is manually resizing the window.
                // Simply restarting the loop is the easiest way to fix this issue.
                Err(SwapchainCreationError::UnsupportedDimensions) => continue,
                Err(err) => panic!("{:?}", err)
            };

            swapchain = new_swapchain;
            // Because framebuffers contains an Arc on the old swapchain, we need to
            // recreate framebuffers as well.
            framebuffers = window_size_dependent_setup(&new_images, render_pass.clone(), &mut dynamic_state);

            recreate_swapchain = false;
        }

        let (image_index, future) = match vulkano::swapchain::acquire_next_image(swapchain.clone(), None) {
            Ok(tuple) => tuple,
            Err(AcquireError::OutOfDate) => {
                recreate_swapchain = true;
                continue;
            },
            Err(error) => panic!("{:?}", error)
        };

        let clear_values = vec!([0.0, 0.0, 1.0, 1.0].into());

        let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(
            device.clone(),
            queue_family.clone())
            .unwrap()
            .begin_render_pass(framebuffers[image_index].clone(), false, clear_values)
            .unwrap()
            .draw(pipeline.clone(), &dynamic_state, vertex_buffer.clone(), (), ())
            .unwrap()
            .end_render_pass()
            .unwrap()
            .build()
            .unwrap();

        let future = previous_frame.join(future)
            .then_execute(queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(queue.clone(), swapchain.clone(), image_index)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                // This wait is required when using NVIDIA or running on macOS. See https://github.com/vulkano-rs/vulkano/issues/1247
                future.wait(None).unwrap();
                previous_frame = Box::new(future) as Box<_>;
            }
            Err(FlushError::OutOfDate) => {
                recreate_swapchain = true;
                previous_frame = Box::new(now(device.clone())) as Box<_>;
            }
            Err(e) => {
                println!("{:?}", e);
                previous_frame = Box::new(now(device.clone())) as Box<_>;
            }
        }

        let mut done = false;
        events_loop.poll_events(|ev| {
            match ev {
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => done = true,
                Event::WindowEvent { event: WindowEvent::Resized(_), .. } => recreate_swapchain = true,
                _ => ()
            }
        });
        if done { return; }
    }
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
