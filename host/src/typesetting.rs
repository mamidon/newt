use std::collections::HashMap;

use euclid::{Point2D, Rect, Size2D, UnknownUnit, Vector2D};
use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::family_name::FamilyName;
use font_kit::hinting::HintingOptions;
use font_kit::loader::{FontTransform, Loader};
use font_kit::loaders::directwrite::Font;
use font_kit::metrics::Metrics;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use std::collections::hash_map::Iter;

pub struct TypeSet {
    font: Font,
    point_size: f32,
    faces: HashMap<u32, TypeFace>,
}

impl<'a> TypeSet {
    pub fn new(point_size: f32) -> TypeSet {
        let font = SystemSource::new()
            .select_best_match(&[FamilyName::SansSerif], &Properties::new())
            .expect("select_best_match failed")
            .load()
            .expect("Font Handle load failed");

        let mut faces: HashMap<u32, TypeFace> = HashMap::new();
        let glyph_count = font.glyph_count();
        for glyph_id in 0..glyph_count {
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
            let origin = font.origin(glyph_id).expect("font.origin failed");
            let advance = font.advance(glyph_id).expect("font.advance failed");

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

            faces.insert(glyph_id, TypeFace::new(glyph_id, glyph_size, canvas.pixels));
        }

        TypeSet {
            font,
            point_size,
            faces,
        }
    }

    pub fn faces(&self) -> impl Iterator<Item = &TypeFace> {
        self.faces.values().filter(|face| face.bounds.area() > 0)
    }

    pub fn glyph_run(&self, text: &str) -> GlyphRun {
        let mut glyphs: Vec<Glyph> = Vec::new();

        for c in text.chars() {
            if let Some(glyph_id) = self.font.glyph_for_char(c) {
                let type_face = self
                    .faces
                    .get(&glyph_id)
                    .expect("all glyphs in font are in the hashmap");

                let bounds = type_face.bounds;
                let advance = self.font.advance(glyph_id).expect("advance failed");
                let offset: Point2D<i64, UnknownUnit> = Point2D::origin();
                let size = Size2D::new(bounds.width, bounds.height);
                let units_per_em = self.font.metrics().units_per_em as f32;
                println!("units_per_em: {}", units_per_em);
                self.font.typographic_bounds(0).unwrap().size
                if size.area() > 0 {
                    glyphs.push(Glyph {
                        glyph_id,
                        offset,
                        size,
                        advance: Vector2D::new(
                            (advance.x / units_per_em) as i64,
                            (advance.y / units_per_em) as i64,
                        ),
                    })
                }
            }
        }

        GlyphRun {
            metrics: self.font.metrics(),
            glyphs,
        }
    }
}

#[derive(Clone)]
pub struct TypeFace {
    glyph_id: u32,
    bounds: Size2D<u32, UnknownUnit>,
    bytes: Vec<u8>,
}

impl TypeFace {
    fn new(glyph_id: u32, bounds: Size2D<u32, UnknownUnit>, bytes: Vec<u8>) -> TypeFace {
        TypeFace {
            glyph_id,
            bounds,
            bytes,
        }
    }

    pub fn size(&self) -> Size2D<u32, UnknownUnit> {
        self.bounds
    }

    pub fn glyph_id(&self) -> u32 {
        self.glyph_id
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

#[derive(Debug)]
pub struct GlyphRun {
    metrics: Metrics,
    glyphs: Vec<Glyph>,
}

impl GlyphRun {
    pub fn position(&self, origin: Point2D<i64, UnknownUnit>) -> GlyphRun {
        let glyphs: Vec<Glyph> = self
            .glyphs
            .iter()
            .scan(origin.to_vector(), |cursor, glyph| {
                let next_glyph = Glyph {
                    offset: glyph.offset + *cursor,
                    size: glyph.size,
                    ..*glyph
                };

                *cursor += glyph.advance;

                Some(next_glyph)
            })
            .collect();

        GlyphRun { glyphs, ..*self }
    }

    pub fn glyphs(&self) -> impl Iterator<Item = &Glyph> {
        self.glyphs.iter()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Glyph {
    glyph_id: u32,
    offset: Point2D<i64, UnknownUnit>,
    size: Size2D<u32, UnknownUnit>,
    advance: Vector2D<i64, UnknownUnit>,
}

impl Glyph {
    pub fn id(&self) -> u32 {
        self.glyph_id
    }

    pub fn offset(&self) -> Point2D<i64, UnknownUnit> {
        self.offset
    }

    pub fn size(&self) -> Size2D<u32, UnknownUnit> {
        self.size
    }
}
