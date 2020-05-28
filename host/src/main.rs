#![allow(unused)]

extern crate drawing;

use drawing::{Brush, Drawing, DrawingOptions, Extent, ShapeKind, SurfaceId};
use png;
use std::io::Cursor;

use crate::typesetting::{TypeFace, TypeSet};
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
    let mut type_face_textures: HashMap<char, (SurfaceId, TypeFace)> = HashMap::new();
    let type_face = type_set.get_face('q').unwrap();

    for (&character, type_face) in type_set.get_faces() {
        let texture_id = drawing
            .load_rgba_texture(
                type_face.bounds().width,
                type_face.bounds().height,
                type_face.to_rgba_bytes(255, 0, 0).as_slice(),
            )
            .expect("");
        type_face_textures.insert(character, (texture_id, type_face.clone()));
    }
    let mut force_recreate = false;

    loop {
        let mut draw_list = drawing
            .create_draw_list()
            .expect("Failed to create_draw_list");

        draw_list.push_shape(
            ShapeKind::Ellipse,
            Brush {
                foreground: 0xFF0000FF,
                background: 0x00FF00FF,
            },
            Extent::new(100, 100, 50, 50),
        );
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
