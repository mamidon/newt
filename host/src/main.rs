#![allow(unused)]

extern crate drawing;

use drawing::{Brush, Drawing, DrawingOptions, Extent, MaskId, ShapeKind, SurfaceId};
use png;
use std::io::{BufRead, BufReader, Cursor};

use crate::layout::{
    Dimensions, LayoutItem, LayoutSpace, ShapeLeaf, TextLeaf, VerticalStackContainer,
    WindowContainer,
};

use euclid::{Point2D, Size2D, Vector2D};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::ops::Add;
use winit::{Event, EventsLoop, KeyboardInput, VirtualKeyCode, Window, WindowBuilder, WindowEvent};

mod layout;

fn main() {
    let mut events_loop = EventsLoop::new();
    let mut drawing: Drawing = Drawing::initialize(
        &events_loop,
        DrawingOptions {
            width: 1024,
            height: 1024,
        },
    )
    .expect("Failed to initialize Drawing");

    let mut force_recreate = false;
    let mut root = LayoutItem::container(WindowContainer::new(1024, 1024));
    let mut stack = LayoutItem::container(VerticalStackContainer::new());
    let brush = Brush {
        foreground: 0xFF0000FF,
        background: 0x00FF00FF,
    };
    let dimensions = Dimensions::new(150, 150);
    let shape1 = LayoutItem::leaf(ShapeLeaf::new(ShapeKind::Rectangle, brush, dimensions));
    let shape2 = LayoutItem::leaf(ShapeLeaf::new(ShapeKind::Ellipse, brush, dimensions));
    let text1 = LayoutItem::leaf(TextLeaf::new("Help\nHelp\nHello", &drawing.type_set));

    stack.attach(shape1);
    stack.attach(text1);
    stack.attach(shape2);
    root.attach(stack);
    let outcome = root.layout(&LayoutSpace::new(Some(100), Some(100)));

    loop {
        drawing.submit_draw_list(&outcome.draw_list, force_recreate);
        force_recreate = false;

        let mut done = false;
        events_loop.poll_events(|polled_event| {
            let window_event = if let Event::WindowEvent { event, .. } = polled_event {
                event
            } else {
                return;
            };

            match window_event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(virtual_keycode),
                            ..
                        },
                    ..
                } => match virtual_keycode {
                    VirtualKeyCode::Escape => done = true,
                    _ => (),
                },
                WindowEvent::CloseRequested => {
                    done = true;
                }
                WindowEvent::Resized(_) => {
                    force_recreate = true;
                }
                _ => (),
            }
        });

        if done {
            return;
        }
    }
}
