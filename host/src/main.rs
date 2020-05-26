#![allow(unused)]

extern crate drawing;

use drawing::{Brush, Drawing, DrawingOptions, Extent, ShapeKind};
use png;
use std::io::Cursor;

use euclid::{Point2D, Size2D, UnknownUnit};
use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::family_name::FamilyName;
use font_kit::hinting::HintingOptions;
use font_kit::loader::FontTransform;
use font_kit::loaders::directwrite::Font;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use std::collections::HashMap;
use winit::{Event, EventsLoop, Window, WindowBuilder, WindowEvent};

struct TypeSet {
    font: Font,
    point_size: f32,
    faces: HashMap<char, TypeFace>,
}

impl TypeSet {
    pub fn new(font: Font, point_size: f32) -> TypeSet {
        let mut faces: HashMap<char, TypeFace> = HashMap::new();

        for code_index in 0..255u8 {
            let code_point: char = code_index.into();
            if let Some(glyph_id) = font.glyph_for_char(code_point) {
                let glyph_bounds = font
                    .raster_bounds(
                        glyph_id,
                        point_size,
                        &FontTransform::identity(),
                        &Point2D::origin(),
                        HintingOptions::None,
                        RasterizationOptions::GrayscaleAa,
                    )
                    .expect("glyph_for_char failed");

                let glyph_size = Size2D::new(
                    glyph_bounds.size.width as u32,
                    glyph_bounds.size.height as u32,
                );

                let mut canvas = Canvas::new(&glyph_size, Format::A8);
                font.rasterize_glyph(
                    &mut canvas,
                    glyph_id,
                    point_size,
                    &FontTransform::identity(),
                    &Point2D::new(
                        -(glyph_bounds.origin.x as f32),
                        -(glyph_bounds.origin.y as f32),
                    ),
                    HintingOptions::None,
                    RasterizationOptions::GrayscaleAa,
                );
                faces.insert(
                    code_point,
                    TypeFace::new(glyph_id, glyph_size, canvas.pixels),
                );
            }
        }

        TypeSet {
            font,
            point_size,
            faces,
        }
    }

    pub fn get_face(&self, code_point: char) -> Option<&TypeFace> {
        self.faces.get(&code_point)
    }
}

struct TypeFace {
    glyph_id: u32,
    bounds: Size2D<u32, UnknownUnit>,
    bytes: Vec<u8>,
}

impl TypeFace {
    pub fn new(glyph_id: u32, bounds: Size2D<u32, UnknownUnit>, bytes: Vec<u8>) -> TypeFace {
        TypeFace {
            glyph_id,
            bounds,
            bytes,
        }
    }

    pub fn as_a8_bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    pub fn to_rgba_bytes(&self, r: u8, g: u8, b: u8) -> Vec<u8> {
        let rgb: u32 = (r as u32) << 24 | (g as u32) << 16 | (b as u32) << 8;

        let rgba_bytes: Vec<[u8; 4]> = self
            .bytes
            .iter()
            .map(|byte| match byte {
                0 => (255 << 16) | 255,
                &x => rgb | (x as u32),
            })
            .map(|pixel| pixel.to_be_bytes())
            .collect();

        rgba_bytes
            .iter()
            .flat_map(|rba| rba.iter())
            .cloned()
            .collect()
    }
}

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

    let type_set = TypeSet::new(font, 32.0);
    let type_face = type_set.get_face('q').unwrap();

    let texture_id = drawing
        .load_rgba_texture(
            type_face.bounds.width,
            type_face.bounds.height,
            type_face.to_rgba_bytes(255, 0, 0).as_slice(),
        )
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
