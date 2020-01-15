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

    loop {
        renderer.begin_frame(force_recreate);
        force_recreate = false;

        let commands = vec![RenderCommand::Rectangle { x: 10, y: 10, width: 100, height: 100 }];
        renderer.submit_commands(commands);

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
