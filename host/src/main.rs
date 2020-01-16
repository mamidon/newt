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
use crate::newt_render::{Renderer, RenderCommand};

#[derive(Default, Debug, Clone)]
struct Vertex { position: [f32; 2] }
vulkano::impl_vertex!(Vertex, position);

mod newt_render;

fn main() {
    let mut events_loop = EventsLoop::new();
    let mut renderer: Renderer = newt_render::Renderer::initialize(&events_loop).ok().unwrap();
    let mut force_recreate = false;
    let mut previous_frame_future: Option<Box<dyn GpuFuture>> = None;

    loop {
        match &mut previous_frame_future {
            Some(future) => future.cleanup_finished(),
            _ => {}
        }

        renderer.begin_frame(force_recreate);
        force_recreate = false;

        let mut commands: Vec<RenderCommand> = Vec::new();

        let mut x_offset = -512;
        for x in 0..10 {
            let stride = 55;

            let mut y_offset = -512;
            for y in 0..10 {

                commands.push(RenderCommand::Rectangle { x: x_offset, y: y_offset, width: 50, height: 50 });

                y_offset += stride;
            }
            x_offset += stride;
        }

        previous_frame_future = Some(renderer.submit_commands(previous_frame_future, commands));
        renderer.present();

        let mut done = false;
        events_loop.poll_events(|ev| {
            match ev {
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => done = true,
                Event::WindowEvent { event: WindowEvent::Resized(_), .. } => force_recreate = true,
                _ => ()
            }
        });
        if done { return; }
    }
}
