#![allow(unused)]

extern crate drawing;

use drawing::{Brush, Drawing, DrawingOptions, Extent, ShapeKind, SurfaceId};
use png;
use std::io::Cursor;

use crate::typesetting::{TypeFace, TypeSet};
use euclid::Point2D;
use std::collections::HashMap;
use winit::{Event, EventsLoop, Window, WindowBuilder, WindowEvent};

mod typesetting;

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

    let type_set = TypeSet::new(32.0);
    let mut type_face_textures: HashMap<u32, SurfaceId> = HashMap::new();

    for type_face in type_set.faces() {
        let texture_id = drawing
            .load_rgba_texture(
                type_face.size().width,
                type_face.size().height,
                type_face.to_rgba_bytes(255, 0, 0).as_slice(),
            )
            .expect("load_rgba_texture failed");
        type_face_textures.insert(type_face.glyph_id(), texture_id);
    }

    let mut glyph_run = type_set.glyph_run("Hello, World!");
    println!("Glyph run: {:#?}", glyph_run);

    glyph_run = glyph_run.position(Point2D::new(100, 100));

    let mut force_recreate = false;
    loop {
        let mut draw_list = drawing
            .create_draw_list()
            .expect("Failed to create_draw_list");

        for glyph in glyph_run.glyphs() {
            if let Some(surface_id) = type_face_textures.get(&glyph.id()) {
                draw_list.push_glyph(
                    *surface_id,
                    Extent::new(
                        glyph.offset().x,
                        glyph.offset().y,
                        glyph.size().width,
                        glyph.size().height,
                    ),
                );
            } else {
                println!("failed to render glyph_id: {:?}", glyph);
            }
        }
        //draw_list.push_glyph(texture_id, Extent::new(x_offset, y_offset, 16, 16));

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
