#![allow(unused)]

extern crate drawing;

use drawing::{Brush, DrawCommand, Drawing, DrawingOptions, Extent, ShapeKind};
use png;
use std::io::Cursor;

use winit::{Event, EventsLoop, Window, WindowBuilder, WindowEvent};

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
    let mut image_data = Vec::new();
    image_data.resize((info.height * info.width * 4) as usize, 0);
    reader.next_frame(&mut image_data).unwrap();
    let mut force_recreate = false;

    loop {
        let mut draw_list = drawing
            .create_draw_list()
            .expect("Failed to create_draw_list");

        let mut x_offset = 0;
        let stride = 55;
        for x in 0..10 {
            let mut y_offset = 0;
            for y in 0..10 {
                if x % 2 == 0 && y % 2 == 0 {
                    draw_list.push_shape(
                        ShapeKind::Rectangle,
                        Brush {
                            foreground: 0xFF0000FF,
                            background: 0x00FF00FF,
                        },
                        Extent::new(x_offset, y_offset, 50, 50),
                    );
                } else {
                    draw_list.push_shape(
                        ShapeKind::Ellipse,
                        Brush {
                            foreground: 0x00FF00FF,
                            background: 0x00FF0000,
                        },
                        Extent::new(x_offset, y_offset, 50, 50),
                    );
                }

                y_offset += stride;
            }
            x_offset += stride;
        }

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
