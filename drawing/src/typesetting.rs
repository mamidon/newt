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

struct FontUnits;
pub struct Pixels;

pub struct TypeSet {
    font: Font,
    point_size: f32,
    font_units_to_pixels_scale: f32,
    faces: HashMap<u32, TypeFace>,
}

impl TypeSet {
    pub fn new(point_size: f32) -> TypeSet {
        let font = TypeSet::load_font();
        let font_units_to_pixels_scale =
            (point_size * 96.0 / 72.0) / (font.metrics().units_per_em as f32);

        let faces = TypeSet::build_faces(&font, point_size, font_units_to_pixels_scale);

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
        GlyphRun::build(self, text)
    }

    fn load_font() -> Font {
        SystemSource::new()
            .select_best_match(&[FamilyName::SansSerif], &Properties::new())
            .expect("select_best_match failed")
            .load()
            .expect("Font Handle load failed")
    }

    fn build_faces(
        font: &Font,
        point_size: f32,
        font_units_to_pixels_scale: f32,
    ) -> HashMap<u32, TypeFace> {
        let mut faces = HashMap::new();
        let glyph_count = font.glyph_count();

        for glyph_id in 0..glyph_count {
            let type_face = TypeFace::build(font, glyph_id, point_size, font_units_to_pixels_scale);

            faces.insert(glyph_id, type_face);
        }

        faces
    }
}

#[derive(Clone)]
pub struct TypeFace {
    glyph_id: u32,
    raster_offset: Point2D<i32, Pixels>,
    raster_size: Size2D<u32, Pixels>,
    raster_advance: Vector2D<i32, Pixels>,
    bytes: Vec<u8>,
}

impl TypeFace {
    fn build(
        font: &Font,
        glyph_id: u32,
        point_size: f32,
        font_units_to_pixels_scale: f32,
    ) -> TypeFace {
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

        let raster_size: Size2D<u32, Pixels> = Size2D::new(
            glyph_bounds.size.width as u32,
            glyph_bounds.size.height as u32,
        );
        let raster_offset = Point2D::new(glyph_bounds.origin.x, glyph_bounds.origin.y);
        let reverse_offset = raster_offset.mul(-1);

        let mut canvas = Canvas::new(&raster_size.cast_unit(), Format::A8);
        font.rasterize_glyph(
            &mut canvas,
            glyph_id,
            point_size,
            &FontTransform::identity(),
            &Point2D::new(reverse_offset.x as f32, reverse_offset.y as f32),
            HintingOptions::None,
            RasterizationOptions::GrayscaleAa,
        );

        let raster_advance = font
            .advance(glyph_id)
            .expect("advance failed")
            .mul(font_units_to_pixels_scale);

        TypeFace {
            glyph_id,
            bytes: canvas.pixels,
            raster_size,
            raster_offset,
            raster_advance: Vector2D::new(raster_advance.x as i32, raster_advance.y as i32),
        }
    }

    pub fn size(&self) -> Size2D<u32, Pixels> {
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
                0 => 0x00FF00FF,
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

    pub fn to_mask_bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }
}

#[derive(Debug)]
pub struct GlyphRun {
    glyphs: Vec<Glyph>,
}

impl GlyphRun {
    fn build(type_set: &TypeSet, text: &str) -> GlyphRun {
        let metrics = type_set.font.metrics();
        let raster_ascent = (metrics.ascent * type_set.font_units_to_pixels_scale) as i32;

        let pixels_per_line = {
            let raster_descent = (metrics.descent * type_set.font_units_to_pixels_scale) as i32;
            let raster_linegap = (metrics.line_gap * type_set.font_units_to_pixels_scale) as i32;

            raster_ascent - raster_descent + raster_linegap
        };

        let mut origin = Vector2D::new(0, raster_ascent);
        let mut glyphs: Vec<Glyph> = Vec::new();

        for c in text.chars() {
            let glyph_id = match type_set.font.glyph_for_char(c) {
                Some(glyph_id) => glyph_id,
                None => continue,
            };

            let type_face = type_set
                .faces
                .get(&glyph_id)
                .expect("all glyphs in font are in the hashmap");

            let advance_scale = if c == '\t' { 4 } else { 1 };
            let advance = type_face.raster_advance.mul(advance_scale);

            glyphs.push(Glyph {
                glyph_id,
                offset: type_face.raster_offset + origin,
                size: type_face.raster_size,
                is_whitespace: c.is_whitespace(),
            });

            origin = if c == '\n' {
                Vector2D::new(0, origin.y + pixels_per_line)
            } else {
                origin + advance
            };
        }

        GlyphRun { glyphs }
    }

    pub fn glyphs(&self) -> impl Iterator<Item = &Glyph> {
        self.glyphs.iter().filter(|g| !g.is_whitespace)
    }

    pub fn with_offset(&self, delta: Vector2D<i32, Pixels>) -> GlyphRun {
        let glyphs = self
            .glyphs
            .iter()
            .filter(|g| !g.is_whitespace)
            .map(|g| g.with_offset(delta))
            .collect();

        GlyphRun { glyphs }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Glyph {
    glyph_id: u32,
    offset: Point2D<i32, Pixels>,
    size: Size2D<u32, Pixels>,
    is_whitespace: bool,
}

impl Glyph {
    pub fn id(&self) -> u32 {
        self.glyph_id
    }

    pub fn offset(&self) -> Point2D<i32, Pixels> {
        self.offset
    }

    pub fn size(&self) -> Size2D<u32, Pixels> {
        self.size
    }

    pub fn with_offset(&self, delta: Vector2D<i32, Pixels>) -> Glyph {
        Glyph {
            offset: self.offset + delta,
            ..*self
        }
    }
}