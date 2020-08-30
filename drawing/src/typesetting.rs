use std::collections::HashMap;

use euclid::{Point2D, Size2D, Vector2D};
use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::family_name::FamilyName;
use font_kit::hinting::HintingOptions;
use font_kit::loader::FontTransform;
use font_kit::loaders::directwrite::Font;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;

use font_kit::metrics::Metrics;
use std::ops::Mul;

pub struct Pixels;

#[derive(Clone)]
pub struct TypeSet {
    font: Font,
    font_metrics: Metrics,
    font_units_to_pixels_scale: f32,
    faces: HashMap<u32, TypeFace>,
}

impl TypeSet {
    pub fn new(point_size: f32) -> TypeSet {
        let font = TypeSet::load_font();
        let font_metrics = font.metrics();
        let font_units_to_pixels_scale =
            (point_size * 96.0 / 72.0) / (font_metrics.units_per_em as f32);

        let faces = TypeSet::build_faces(&font, point_size, font_units_to_pixels_scale);

        TypeSet {
            font,
            font_metrics,
            font_units_to_pixels_scale,
            faces,
        }
    }

    pub fn as_glyphs(&self, text: &str) -> Glyphs {
        let glyphs: Vec<Glyph> = text
            .chars()
            .map(|c| self.font.glyph_for_char(c))
            .map(|o| o.expect("A character code didn't map to a glyph id"))
            .map(|id| Glyph::new(id))
            .collect();

        Glyphs::new(glyphs)
    }

    pub fn as_typeset_glyphs(&self, text: &str) -> Vec<TypeSetGlyph> {
        self.as_glyphs(text)
            .glyphs()
            .map(|g| TypeSetGlyph::new(self, *g))
            .collect()
    }

    pub fn build_glyph_run(&self) -> GlyphRun {
        let raster_ascent = (self.font_metrics.ascent * self.font_units_to_pixels_scale) as i32;
        let pixels_per_line = {
            let raster_descent =
                (self.font_metrics.descent * self.font_units_to_pixels_scale) as i32;
            let raster_linegap =
                (self.font_metrics.line_gap * self.font_units_to_pixels_scale) as i32;

            raster_ascent - raster_descent + raster_linegap
        };

        let mut run = GlyphRun::new(pixels_per_line as u32);
        run
    }

    pub fn faces(&self) -> impl Iterator<Item = &TypeFace> {
        self.faces
            .values()
            .filter(|face| face.raster_size.area() > 0)
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
        )
        .expect("rasterize_glyph failed");

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

#[derive(Copy, Clone, Debug)]
pub struct Glyph {
    glyph_id: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct TypeSetGlyph {
    pub glyph_id: u32,
    pub bounds: Size2D<u32, Pixels>,
    pub baseline_offset: Point2D<i32, Pixels>,
    pub advance: Vector2D<i32, Pixels>,
}

#[derive(Debug)]
pub struct Glyphs {
    glyphs: Vec<Glyph>,
}

pub struct GlyphRun {
    line_height: u32,
    line_width: u32,
    glyphs: Vec<TypeSetGlyph>,
}

impl Glyph {
    pub fn new(glyph_id: u32) -> Glyph {
        Glyph { glyph_id }
    }

    pub fn id(&self) -> u32 {
        self.glyph_id
    }
}

impl TypeSetGlyph {
    pub fn new(type_set: &TypeSet, glyph: Glyph) -> TypeSetGlyph {
        let glyph_id = glyph.glyph_id;
        let face = type_set
            .faces
            .get(&glyph_id)
            .expect("All glyphs in font are in the faces hashmap");

        TypeSetGlyph {
            glyph_id,
            bounds: face.raster_size,
            baseline_offset: face.raster_offset,
            advance: face.raster_advance,
        }
    }
}

impl Glyphs {
    pub fn empty() -> Glyphs {
        Glyphs { glyphs: Vec::new() }
    }
    pub fn new(glyphs: Vec<Glyph>) -> Glyphs {
        Glyphs { glyphs }
    }

    pub fn glyphs(&self) -> impl Iterator<Item = &Glyph> {
        self.glyphs.iter()
    }
}

impl GlyphRun {
    pub fn new(line_height: u32) -> GlyphRun {
        GlyphRun {
            line_height,
            line_width: 0,
            glyphs: Vec::new(),
        }
    }

    pub fn append(&mut self, new_glyphs: &[TypeSetGlyph]) {
        for glyph in new_glyphs {
            self.line_width += glyph.bounds.width + glyph.advance.x as u32;
            self.glyphs.push(*glyph);
        }
    }

    pub fn line_width(&self) -> u32 {
        self.line_width
    }

    pub fn line_height(&self) -> u32 {
        self.line_height
    }

    pub fn glyphs(&self) -> impl Iterator<Item = &TypeSetGlyph> {
        self.glyphs.iter()
    }
}
