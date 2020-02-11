#![allow(unused)]

extern crate vulkano;
extern crate vulkano_win;

use crate::newt_render::{RenderCommand, Renderer};
use png;
use std::cell::RefCell;
use std::fmt::{Display, Error, Formatter};
use std::io::Cursor;
use std::sync::Arc;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::{Device, DeviceCreationError, DeviceExtensions, Queue, QueuesIter};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass};
use vulkano::image::{Dimensions, ImmutableImage, SwapchainImage};
use vulkano::instance::{Instance, PhysicalDevice, QueueFamily};
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::swapchain::{
    AcquireError, ColorSpace, PresentMode, Surface, SurfaceTransform, Swapchain,
    SwapchainCreationError,
};
use vulkano::sync::{now, FlushError, GpuFuture};
use vulkano_win::VkSurfaceBuild;
use winit::{Event, EventsLoop, Window, WindowBuilder, WindowEvent};

#[derive(Default, Debug, Clone)]
struct Vertex {
    position: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position);

mod newt_render;

fn main() {
    let mut events_loop = EventsLoop::new();
    let mut renderer: Renderer = newt_render::Renderer::initialize(&events_loop)
        .ok()
        .unwrap();
    let mut force_recreate = false;

    let image = include_bytes!("image_img.png").to_vec();
    let decoder = png::Decoder::new(Cursor::new(image));
    let (info, mut reader) = decoder.read_info().unwrap();
    let image_dimensions = Dimensions::Dim2d {
        width: info.width,
        height: info.height,
    };
    let mut image_data = Vec::new();
    image_data.resize((info.height * info.width * 4) as usize, 0);
    reader.next_frame(&mut image_data).unwrap();
    let surface = renderer.load_image(image_data, info.height as usize, info.width as usize)
        .expect("load_image failed");

    loop {
        let mut frame = renderer
            .begin_frame(force_recreate)
            .expect("begin_frame failed");
        force_recreate = false;

        let mut commands: Vec<RenderCommand> = Vec::new();

        let mut x_offset = 0;
        let stride = 55;
        for x in 0..0 {
            let mut y_offset = 0;
            for y in 0..0 {
                commands.push(RenderCommand::Rectangle {
                    x: x_offset,
                    y: y_offset,
                    width: 50,
                    height: 50,
                });

                y_offset += stride;
            }
            x_offset += stride;
        }

        commands.push(RenderCommand::NewtSurface {
            x: 0,
            y: 0,
            width: 512,
            height: 512,
            surface: surface.clone()
        });

        frame.submit_commands(commands);
        renderer.present(frame);

        let mut done = false;
        events_loop.poll_events(|ev| match ev {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => done = true,
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => force_recreate = true,
            _ => (),
        });
        if done {
            return;
        }
    }
}
