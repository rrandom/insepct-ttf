use iced::{canvas, Point};
use owned_ttf_parser::{AsFontRef, OwnedFont};

// pub async fn parse_font(data: Vec<u8>) -> Option<owned_ttf_parser::OwnedFont> {
//   match async { return OwnedFont::from_vec(data, 0) }.await {
//       Some(f) => Some(f),
//       None => None,
//   }
// }

struct OutlineBuilder {
    inner: canvas::path::Builder,
}

impl owned_ttf_parser::OutlineBuilder for OutlineBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.inner.move_to(Point::new(x, y));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.inner.line_to(Point::new(x, y));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.inner
            .quadratic_curve_to(Point::new(x1, y1), Point::new(x, y));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.inner
            .bezier_curve_to(Point::new(x1, y1), Point::new(x2, y2), Point::new(x, y));
    }

    fn close(&mut self) {
        self.inner.close();
    }
}

pub fn get_bbox_outline(
    font: &OwnedFont,
    id: owned_ttf_parser::GlyphId,
) -> (Option<owned_ttf_parser::Rect>, canvas::Path) {
    let builder = canvas::path::Builder::new();

    let mut outline = OutlineBuilder { inner: builder };
    let bbox = font.as_font().outline_glyph(id, &mut outline);

    (bbox, outline.inner.build())
}
