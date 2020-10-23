#![allow(unused)]

extern crate drawing;

use drawing::{
    Brush, DrawList, Drawing, DrawingOptions, Extent, Handle, MaskId, ShapeKind, SurfaceId,
};
use png;
use std::io::{BufRead, BufReader, Cursor};

use crate::layout::{
    Dimensions, LayoutItem, LayoutNode, Pixels, RenderItemKind, RenderNode, RenderSpace,
};

mod layout;

use drawing::typesetting::TypeSet;
use euclid::{Point2D, Size2D, Vector2D};
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::ops::Add;
use std::path::Path;
use winit::{Event, EventsLoop, KeyboardInput, VirtualKeyCode, Window, WindowBuilder, WindowEvent};

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

    let type_set = TypeSet::new(12.0);

    let text = std::env::args()
        .nth(1)
        .map_or("Hello, world!".to_string(), |path| {
            std::fs::read_to_string(Path::new(&path)).expect("Failed to open file")
        });

    let glyph_ids: Vec<u32> = type_set.as_glyphs(&text).glyphs().map(|g| g.id()).collect();
    let mut glyph_to_masks: HashMap<u32, Handle> = HashMap::new();

    for face in type_set
        .faces()
        .filter(|f| glyph_ids.contains(&f.glyph_id()))
    {
        let mask_id = drawing
            .load_mask_texture(face.size().width, face.size().height, face.as_a8_bytes())
            .unwrap();

        glyph_to_masks.insert(face.glyph_id(), mask_id);
    }

    let mut force_recreate = false;

    let brush_a = Brush {
        foreground: 0xFF0000FF,
        background: 0x00FF00FF,
    };
    let brush_b = Brush {
        foreground: 0x00FF00FF,
        background: 0x00FF00FF,
    };
    let brush_c = Brush {
        foreground: 0x0000FFFF,
        background: 0x00FF00FF,
    };
    let brush_d = Brush {
        foreground: 0x00FFFFFF,
        background: 0x00FF00FF,
    };
    let dimensions = Dimensions::new(150, 150);
    let layout_root = LayoutNode::new_stack(vec![
        LayoutNode::new_text(&text, type_set.clone()),
        LayoutNode::new_shape(ShapeKind::Rectangle, brush_a, dimensions),
        LayoutNode::new_shape(ShapeKind::Rectangle, brush_b, dimensions),
        LayoutNode::new_stack(vec![
            LayoutNode::new_shape(ShapeKind::Ellipse, brush_d, dimensions),
            LayoutNode::new_shape(ShapeKind::Ellipse, brush_d, dimensions),
        ]),
        LayoutNode::new_shape(ShapeKind::Rectangle, brush_c, dimensions),
    ]);
    let render_root = layout_root.layout(Some(1024), Some(1024));
    loop {
        force_recreate = false;

        let mut draw_list = drawing.create_draw_list().unwrap();
        for node in render_root.iter() {
            let extent = Extent::new(
                node.position().x,
                node.position().y,
                node.dimensions().width as u32,
                node.dimensions().height as u32,
            );

            match &node.kind().borrow() {
                RenderItemKind::Shape {
                    kind,
                    dimensions,
                    brush,
                } => draw_list.push_shape(*kind, *brush, extent),
                RenderItemKind::Stack { .. } => {}
                RenderItemKind::Box { .. } => {}
                RenderItemKind::Text { lines, type_set } => {
                    let layout_offset = node.position();
                    for (index, line) in lines.iter().enumerate() {
                        let base_line = line.base_line();

                        for glyph in line.glyphs() {
                            let x = layout_offset.x
                                + base_line.x as i64
                                + glyph.1.baseline_offset.x as i64;
                            let y = layout_offset.y
                                + (-base_line.y as i64)
                                + glyph.1.baseline_offset.y as i64;

                            if glyph_to_masks.contains_key(&glyph.0.id()) {
                                draw_list.push_mask(
                                    *glyph_to_masks.get(&glyph.0.id()).unwrap(),
                                    Brush {
                                        foreground: 0x000000FF,
                                        background: 0x00000000,
                                    },
                                    Extent::new(x, y, glyph.1.bounds.width, glyph.1.bounds.height),
                                );
                            }
                        }
                    }
                }
            }
        }

        drawing.submit_draw_list(&draw_list, false);
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
