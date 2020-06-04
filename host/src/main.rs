#![allow(unused)]

extern crate drawing;

use drawing::{Brush, Drawing, DrawingOptions, Extent, ShapeKind, SurfaceId};
use png;
use std::io::{BufRead, BufReader, Cursor};

use crate::typesetting::{GlyphRun, TypeFace, TypeSet};
use euclid::{Point2D, Size2D};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::ops::Add;
use winit::{Event, EventsLoop, Window, WindowBuilder, WindowEvent};

mod typesetting;

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
    let arguments: Vec<String> = std::env::args().collect();
    let file_path = arguments.get(1).expect("File path not provided");
    let file = File::open(file_path).expect("File not found");
    let file_lines: Vec<String> = BufReader::new(file).lines().map(|l| l.unwrap()).collect();
    let file_content = file_lines.join("\n");

    let type_set = TypeSet::new(12.0);
    let mut type_face_textures: HashMap<u32, SurfaceId> = HashMap::new();

    let mut glyph_run: GlyphRun = type_set.glyph_run(&file_content);
    let used_glyph_ids: HashSet<u32> = glyph_run.glyphs().map(|glyph| glyph.id()).collect();

    for type_face in type_set.faces() {
        if !used_glyph_ids.contains(&type_face.glyph_id()) {
            continue;
        }

        let texture_id = drawing
            .load_rgba_texture(
                type_face.size().width,
                type_face.size().height,
                type_face.to_rgba_bytes(0, 0, 0).as_slice(),
            )
            .expect("load_rgba_texture failed");
        type_face_textures.insert(type_face.glyph_id(), texture_id);
    }

    let mut force_recreate = false;
    loop {
        let mut line_start = Point2D::new(0, 0);
        let mut draw_list = drawing
            .create_draw_list()
            .expect("Failed to create_draw_list");

        for glyph in glyph_run.position(line_start).glyphs() {
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
