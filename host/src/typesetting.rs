use std::collections::HashMap;

use euclid::{Point2D, Size2D, UnknownUnit};
use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::family_name::FamilyName;
use font_kit::hinting::HintingOptions;
use font_kit::loader::FontTransform;
use font_kit::loaders::directwrite::Font;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use std::collections::hash_map::Iter;

pub struct TypeSet {
    font: Font,
    point_size: f32,
    faces: HashMap<char, TypeFace>,
}

impl<'a> TypeSet {
    pub fn new(point_size: f32) -> TypeSet {
        let font = SystemSource::new()
            .select_best_match(&[FamilyName::SansSerif], &Properties::new())
            .expect("select_best_match failed")
            .load()
            .expect("Font Handle load failed");

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

    pub fn characters(&self) -> impl Iterator<Item = &char> {
        self.faces
            .iter()
            .filter(|kvp| kvp.1.bounds().area().gt(&0))
            .map(|kvp| kvp.0)
    }

    pub fn get_face(&self, code_point: char) -> Option<&TypeFace> {
        self.faces.get(&code_point)
    }
}

#[derive(Clone)]
pub struct TypeFace {
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

    pub fn bounds(&self) -> Size2D<u32, UnknownUnit> {
        self.bounds
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
