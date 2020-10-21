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
use std::cmp::max;
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
            .map(|id| id.expect("A character code didn't map to a glyph id"))
            .map(|id| Glyph { id })
            .collect();

        Glyphs { glyphs }
    }

    pub fn analyze_glyphs(&self, glyphs: &Glyphs) -> Vec<GlyphAnalysis> {
        glyphs
            .glyphs()
            .map(|g| GlyphAnalysis::new(self, *g))
            .collect()
    }

    pub fn build_glyph_run(&self) -> GlyphRun {
        GlyphRun::new(self.clone())
    }

    pub fn faces(&self) -> impl Iterator<Item = &TypeFace> {
        self.faces
            .values()
            .filter(|face| face.raster_size.area() > 0)
    }

    pub fn ascent(&self) -> i32 {
        (self.font_metrics.ascent * self.font_units_to_pixels_scale) as i32
    }

    pub fn descent(&self) -> i32 {
        (self.font_metrics.descent * self.font_units_to_pixels_scale) as i32
    }

    pub fn line_gap(&self) -> i32 {
        (self.font_metrics.line_gap * self.font_units_to_pixels_scale) as i32
    }

    pub fn line_height(&self) -> i32 {
        self.ascent() - self.descent() + self.line_gap()
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
    id: u32,
}

impl Glyph {
    pub fn id(&self) -> u32 {
        self.id
    }
}

#[derive(Copy, Clone, Debug)]
pub struct GlyphAnalysis {
    pub bounds: Size2D<u32, Pixels>,
    pub baseline_offset: Point2D<i32, Pixels>,
    pub advance: Vector2D<i32, Pixels>,
}

#[derive(Debug)]
pub struct Glyphs {
    glyphs: Vec<Glyph>,
}

#[derive(Clone)]
pub struct GlyphRun {
    type_set: TypeSet,
    line_width: u32,
    height: i32,
    offset: Vector2D<i32, Pixels>,
    glyphs: Vec<(Glyph, GlyphAnalysis)>,
}

impl GlyphAnalysis {
    pub fn new(type_set: &TypeSet, glyph: Glyph) -> GlyphAnalysis {
        let face = type_set
            .faces
            .get(&glyph.id)
            .expect("All glyphs in font are in the faces hashmap");

        GlyphAnalysis {
            bounds: face.raster_size,
            baseline_offset: face.raster_offset,
            advance: face.raster_advance,
        }
    }

    pub fn with_offset(&self, x: i32, y: i32) -> GlyphAnalysis {
        GlyphAnalysis {
            baseline_offset: self.baseline_offset + Vector2D::new(x, y),
            ..*self
        }
    }
}

impl Glyphs {
    pub fn glyphs(&self) -> impl Iterator<Item = &Glyph> {
        self.glyphs.iter()
    }
}

impl GlyphRun {
    pub fn new(type_set: TypeSet) -> GlyphRun {
        let ascent = type_set.ascent();

        GlyphRun {
            type_set,
            line_width: 0,
            height: 0,
            offset: Vector2D::new(0, -ascent),
            glyphs: Vec::new(),
        }
    }

    pub fn append_text(&mut self, text: &str) {
        let glyphs = self.type_set.as_glyphs(text);
        let analyses = self.type_set.analyze_glyphs(&glyphs);

        if self.height == 0 {
            self.height = self.type_set.line_height();
        }

        for (glyph, analysis) in glyphs.glyphs().zip(analyses) {
            self.glyphs.push((
                *glyph,
                analysis.with_offset(self.line_width as i32, analysis.baseline_offset.y),
            ));
            self.line_width += analysis.advance.x as u32;
        }
    }

    pub fn set_line_offset(&mut self, line: i32) {
        self.offset = Vector2D::new(
            0,
            -self.type_set.ascent() - line * self.type_set.line_height(),
        )
    }

    pub fn width(&self) -> u32 {
        self.line_width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn glyphs(&self) -> Vec<(Glyph, GlyphAnalysis)> {
        self.glyphs
            .iter()
            .map(|tuple| (tuple.0, tuple.1.with_offset(self.offset.x, self.offset.y)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::typesetting::{GlyphRun, TypeSet};

    #[test]
    fn can_load_singular_font() {
        // TODO support selecting a font.. so ghetto!
        let type_set = TypeSet::new(12.0);

        assert_ne!(0, type_set.ascent());
        assert_ne!(0, type_set.descent());
        assert_ne!(0, type_set.line_height());
    }

    #[test]
    fn can_construct_glyphs_from_strings() {
        let type_set = TypeSet::new(12.0);

        let first = type_set.as_glyphs("hello");
        let second = type_set.as_glyphs(", world");

        assert_eq!(5, first.glyphs.len());
        assert_eq!(7, second.glyphs.len());
    }

    #[test]
    fn glyphrun_starts_out_empty() {
        let type_set = TypeSet::new(12.0);

        let glyph_run = GlyphRun::new(type_set.clone());

        assert_eq!(0, glyph_run.width());
        assert_eq!(0, glyph_run.height());
    }

    #[test]
    fn glyphrun_append_text_concatenates_basic_strings() {
        let type_set = TypeSet::new(12.0);

        let mut glyph_run = GlyphRun::new(type_set.clone());

        let first_glyphs = type_set.as_glyphs("hello");
        let first_analysis = type_set.analyze_glyphs(&first_glyphs);

        let second_glyphs = type_set.as_glyphs("world");
        let second_analysis = type_set.analyze_glyphs(&second_glyphs);

        let expected_width: u32 = first_analysis
            .iter()
            .chain(&second_analysis)
            .map(|a| dbg!(a.advance.x) as u32)
            .sum();

        glyph_run.append_text("hello");
        glyph_run.append_text("world");

        assert_eq!(expected_width, glyph_run.width());
    }

    #[test]
    fn glyphrun_append_text_ignores_newlines_concatenates_basic_strings() {
        let type_set = TypeSet::new(12.0);

        let first_glyphs = type_set.as_glyphs("hello\n");
        let first_analysis = type_set.analyze_glyphs(&first_glyphs);

        let second_glyphs = type_set.as_glyphs("\nworld");
        let second_analysis = type_set.analyze_glyphs(&second_glyphs);

        let expected_width: u32 = first_analysis
            .iter()
            .chain(&second_analysis)
            .map(|a| dbg!(a.advance.x) as u32)
            .sum();

        let mut glyph_run = GlyphRun::new(type_set.clone());

        glyph_run.append_text("hello\n");
        glyph_run.append_text("\nworld");

        assert_eq!(expected_width, glyph_run.width());
        assert_eq!(type_set.line_height(), glyph_run.height());
    }
}
