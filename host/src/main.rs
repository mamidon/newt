#![allow(unused)]

extern crate drawing;
extern crate vulkano;
extern crate vulkano_win;

use crate::newt_render::attachments::HostSurface;
use crate::newt_render::{RenderCommand, Renderer};
use drawing::{Brush, DrawCommand, Drawing, DrawingOptions, Extent, ShapeKind};
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
    let mut drawing: Drawing = Drawing::initialize(
        &events_loop,
        DrawingOptions {
            width: 512,
            height: 512,
        },
    )
    .expect("Failed to initialize Drawing");

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
    let mut force_recreate = false;

    loop {
        let mut draw_list = drawing
            .create_draw_list()
            .expect("Failed to create_draw_list");

        draw_list.push(DrawCommand::Shape {
            kind: ShapeKind::Rectangle,
            extent: Extent::new(100, 100, 10, 10),
            brush: Brush {
                foreground: 0xFF0000FF,
                background: 0x00FF00FF,
            },
        });

        let sealed_draw_list = drawing
            .seal_draw_list(draw_list, force_recreate)
            .expect("Failed to seal draw list");
        force_recreate = false;

        drawing.submit_sealed_draw_list(sealed_draw_list);

        let mut done = false;
        events_loop.poll_events(|ev| match ev {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => done = true,
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                force_recreate = true;
            }
            _ => (),
        });
        if done {
            return;
        }
    }
}
