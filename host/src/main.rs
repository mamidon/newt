#![allow(unused)]

extern crate drawing;

use drawing::{Brush, Drawing, DrawingOptions, Extent, ShapeKind};
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

    let font = SystemSource::new()
        .select_best_match(&[FamilyName::SansSerif], &Properties::new())
        .expect("select_best_match failed")
        .load()
        .expect("Font Handle load failed");
    let glyph_id = font.glyph_for_char('A').expect("glyph_for_char failed");
    let mut canvas = Canvas::new(&Size2D::new(16, 16), Format::A8);
    font.rasterize_glyph(
        &mut canvas,
        glyph_id,
        16.0,
        &FontTransform::identity(),
        &Point2D::new(0.0, 16.0),
        HintingOptions::None,
        RasterizationOptions::GrayscaleAa,
    )
    .expect("rasterize_glyph failed");
    let rgba_bytes: Vec<[u8; 4]> = canvas
        .pixels
        .iter()
        .map(|byte| match byte {
            0 => 0u32,
            x => {
                let x32 = *x as u32;
                (x32 << 16) | (x32)
            }
        })
        .map(|pixel| pixel.to_be_bytes())
        .collect();
    let foo: Vec<u8> = rgba_bytes
        .iter()
        .flat_map(|rba| rba.iter())
        .cloned()
        .collect();
    let texture_id = drawing
        .load_rgba_texture(info.width, info.height, image_data.as_slice())
        .expect("");
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
                        ShapeKind::Ellipse,
                        Brush {
                            foreground: 0xFF0000FF,
                            background: 0x00FF00FF,
                        },
                        Extent::new(x_offset, y_offset, 50, 50),
                    );
                } else {
                    /*draw_list.push_shape(
                        ShapeKind::Ellipse,
                        Brush {
                            foreground: 0x00FF00FF,
                            background: 0x00FF0000,
                        },
                        Extent::new(x_offset, y_offset, 50, 50),
                    );*/
                    draw_list.push_glyph(texture_id, Extent::new(x_offset, y_offset, 16, 16));
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
