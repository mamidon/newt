#![allow(unused)]

extern crate drawing;

use drawing::{Brush, Drawing, DrawingOptions, Extent, MaskId, ShapeKind, SurfaceId};
use png;
use std::io::{BufRead, BufReader, Cursor};

use crate::typesetting::{GlyphRun, Pixels, TypeFace, TypeSet};
use euclid::{Point2D, Size2D, Vector2D};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::ops::Add;
use winit::{Event, EventsLoop, KeyboardInput, VirtualKeyCode, Window, WindowBuilder, WindowEvent};

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
    let mut type_face_textures: HashMap<u32, MaskId> = HashMap::new();

    let mut glyph_run: GlyphRun = type_set.glyph_run(&file_content);
    let used_glyph_ids: HashSet<u32> = glyph_run.glyphs().map(|glyph| glyph.id()).collect();

    for type_face in type_set.faces() {
        if !used_glyph_ids.contains(&type_face.glyph_id()) {
            continue;
        }

        let texture_id = drawing
            .load_mask_texture(
                type_face.size().width,
                type_face.size().height,
                type_face.to_mask_bytes().as_slice(),
            )
            .expect("load_rgba_texture failed");
        type_face_textures.insert(type_face.glyph_id(), texture_id);
    }

    let mut offset: Vector2D<i32, Pixels> = Vector2D::zero();
    let mut force_recreate = false;
    loop {
        glyph_run = glyph_run.with_offset(offset);
        offset = Vector2D::zero();

        let mut draw_list = drawing
            .create_draw_list()
            .expect("Failed to create_draw_list");

        for glyph in glyph_run.glyphs() {
            if let Some(mask_id) = type_face_textures.get(&glyph.id()) {
                draw_list.push_mask(
                    *mask_id,
                    Brush {
                        foreground: 0x000000FF,
                        background: 0x00000000,
                    },
                    Extent::new(
                        glyph.offset().x as i64,
                        glyph.offset().y as i64,
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
                    VirtualKeyCode::Up => offset += Vector2D::new(0, -10),
                    VirtualKeyCode::Down => offset += Vector2D::new(0, 10),
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
