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
use std::borrow::Borrow;
use std::collections::hash_map::Iter;
use std::ops::Mul;

pub struct TypeSet {
    font: Font,
    point_size: f32,
    font_units_to_pixels_scale: f32,
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

        let font_units_to_pixels_scale =
            (point_size * 96.0 / 72.0) / (font.metrics().units_per_em as f32);

        TypeSet {
            font,
            point_size,
            font_units_to_pixels_scale,
            faces,
        }
    }

    pub fn faces(&self) -> impl Iterator<Item = &TypeFace> {
        self.faces
            .values()
            .filter(|face| face.raster_size.area() > 0)
    }

    pub fn glyph_run(&self, text: &str) -> GlyphRun {
        let metrics = self.font.metrics();
        let ascent_pixels = (metrics.ascent * self.font_units_to_pixels_scale) as i64;
        let descent_pixels = (metrics.descent * self.font_units_to_pixels_scale) as i64;
        let linegap_pixels = (metrics.line_gap * self.font_units_to_pixels_scale) as i64;

        let mut glyphs: Vec<Glyph> = Vec::new();

        for c in text.chars() {
            if let Some(glyph_id) = self.font.glyph_for_char(c) {
                let type_face = self
                    .faces
                    .get(&glyph_id)
                    .expect("all glyphs in font are in the hashmap");

                let raster_bounds = type_face.raster_size;
                let advance = self
                    .font
                    .advance(glyph_id)
                    .expect("advance failed")
                    .mul(self.font_units_to_pixels_scale * (if c == '\t' { 4.0 } else { 1.0 }));

                let glyph_bounds = self
                    .font
                    .raster_bounds(
                        glyph_id,
                        self.point_size,
                        &FontTransform::identity(),
                        &Point2D::origin(),
                        HintingOptions::None,
                        RasterizationOptions::GrayscaleAa,
                    )
                    .expect("glyph_for_char failed");

                let size = Size2D::new(raster_bounds.width, raster_bounds.height);
                let units_per_em = self.font.metrics().units_per_em as f32;

                if size.area() > 0 {
                    glyphs.push(Glyph {
                        glyph_id,
                        offset: Point2D::new(
                            glyph_bounds.origin.x as i64,
                            glyph_bounds.origin.y as i64,
                        ),
                        size,
                        advance: Vector2D::new(advance.x as i64, advance.y as i64),
                        is_newline: c == '\n',
                        is_whitespace: c.is_whitespace(),
                    })
                }
            }
        }

        GlyphRun {
            ascent_pixels,
            descent_pixels,
            linegap_pixels,
            glyphs,
        }
    }
}

#[derive(Clone)]
pub struct TypeFace {
    glyph_id: u32,
    raster_size: Size2D<u32, UnknownUnit>,
    bytes: Vec<u8>,
}

impl TypeFace {
    fn new(glyph_id: u32, bounds: Size2D<u32, UnknownUnit>, bytes: Vec<u8>) -> TypeFace {
        TypeFace {
            glyph_id,
            raster_size: bounds,
            bytes,
        }
    }

    pub fn size(&self) -> Size2D<u32, UnknownUnit> {
        self.raster_size
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
                0 => 0,
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
    ascent_pixels: i64,
    descent_pixels: i64,
    linegap_pixels: i64,
    glyphs: Vec<Glyph>,
}

impl GlyphRun {
    pub fn position(&self, origin: Point2D<i64, UnknownUnit>) -> GlyphRun {
        let x_start = origin.x;
        let y_start = origin.y + self.ascent_pixels as i64;
        let pixels_per_line = self.ascent_pixels - self.descent_pixels + self.linegap_pixels;

        let glyphs: Vec<Glyph> = self
            .glyphs
            .iter()
            .scan(Vector2D::new(x_start, y_start), |cursor, glyph| {
                let next_glyph = Glyph {
                    offset: glyph.offset + *cursor,
                    size: glyph.size,
                    ..*glyph
                };

                if glyph.is_newline {
                    *cursor = Vector2D::new(x_start, cursor.y + pixels_per_line as i64);
                } else {
                    *cursor += glyph.advance;
                }

                Some(next_glyph)
            })
            .collect();

        GlyphRun { glyphs, ..*self }
    }

    pub fn glyphs(&self) -> impl Iterator<Item = &Glyph> {
        self.glyphs.iter().filter(|g| !g.is_whitespace)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Glyph {
    glyph_id: u32,
    offset: Point2D<i64, UnknownUnit>,
    size: Size2D<u32, UnknownUnit>,
    advance: Vector2D<i64, UnknownUnit>,
    is_newline: bool,
    is_whitespace: bool,
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
