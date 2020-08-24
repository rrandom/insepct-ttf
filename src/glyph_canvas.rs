use iced::{
    canvas::{self, Cache, Canvas, Cursor, Geometry, Path},
    Color, Element, Length, Point, Rectangle, Vector,
};

use super::glyph_info;

const GLYPH_MARGIN: f32 = 5.0;
const FONT_SIZE: f32 = 128.0;

#[derive(Default)]
pub struct State {
    cache: Cache,
}

impl State {
    pub fn view<'a>(
        &'a mut self,
        font: &'a owned_ttf_parser::Font,
        glyph_info: &'a glyph_info::GlyphInfo,
        width: u16,
        height: u16,
    ) -> Element<'a, ()> {
        Canvas::new(GlyphCanvas {
            state: self,
            font,
            glyph_info,
        })
        .width(Length::Units(width))
        .height(Length::Units(height))
        .into()
    }

    pub fn request_redraw(&mut self) {
        self.cache.clear()
    }
}

struct GlyphCanvas<'a> {
    state: &'a mut State,
    font: &'a owned_ttf_parser::Font<'a>,
    glyph_info: &'a glyph_info::GlyphInfo,
}

impl<'a> canvas::Program<()> for GlyphCanvas<'a> {
    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        if let Some(bbox) = self.glyph_info.bbox {
            let units_per_em = self
                .font
                .units_per_em()
                .ok_or("invalid units per em")
                .unwrap();

            let (glyph_scale, glyph_baseline, glyph_size) = self.calc_scale(&bounds, &bbox);

            let get_ypx = |yunits: f32| glyph_baseline - yunits * glyph_scale;

            let scale = FONT_SIZE / units_per_em as f32;
            let glyph = self.state.cache.draw(bounds.size(), |frame| {
                let iced::Size { width, height } = bounds.size();
                frame.fill_rectangle(
                    Point::ORIGIN,
                    bounds.size(),
                    Color::from_rgba(0.1, 0.1, 0.1, 0.1),
                );
                frame.fill_rectangle(
                    Point::new(width / 2.0, 0.0),
                    iced::Size::new(1.0, height),
                    Color::BLACK,
                );

                frame.fill_rectangle(
                    Point::new(0.0, height / 2.0),
                    iced::Size::new(width, 1.0),
                    Color::BLACK,
                );

                let path = self.get_path();
                let cell_size = bounds.height.min(bounds.width);
                let bbox_w = (bbox.x_max - bbox.x_min) as f32 * scale;
                let dx = (cell_size - bbox_w) / 2.0;
                let y = cell_size + self.font.descender() as f32 * scale;

                dbg!(self.font.ascender(), self.font.descender());

                // frame.translate(Vector::new(-width / 2.0, -height / 2.0));
                // frame.scale(glyph_scale, glyph_scale);

                frame.translate(Vector::new(dx as f32, y as f32));
                frame.scale(scale, scale);
                frame.scale(1.0, -1.0);
                frame.fill(&path, Color::BLACK);

                self.h_line("Baseline", 0.0, frame);
                self.h_line("Ascender", self.font.ascender() as f32, frame);
                self.h_line("Descender", self.font.descender() as f32, frame);
            });
            return vec![glyph];
        }
        vec![]
    }
}

impl<'a> GlyphCanvas<'a> {
    fn get_path(&self) -> Path {
        self.glyph_info.outline.clone().take().unwrap()
    }

    fn h_line(&self, content: &str, y: f32, frame: &mut iced::canvas::Frame) {
        let text = iced::canvas::Text {
            content: content.into(),
            position: iced::Point { x: 2.0, y: y + 3.0 },
            ..Default::default()
        };

        frame.fill_text(text);
        frame.fill_rectangle(
            iced::Point::new(80.0, y),
            iced::Size::new(1000.0, 2.0),
            Color::from_rgb(0.8, 0.0, 0.0),
        );
    }

    fn calc_scale(&self, bounds: &Rectangle, bbox: &owned_ttf_parser::Rect) -> (f32, f32, f32) {
        let glyph_width = bounds.width - GLYPH_MARGIN * 2.0;
        let glyph_height = bounds.height - GLYPH_MARGIN * 2.0;

        let glyph_scale = glyph_width
            / ((bbox.x_max - bbox.x_min) as f32)
                .min(glyph_height / (bbox.y_max - bbox.y_min) as f32);

        let units_per_em = self
            .font
            .units_per_em()
            .ok_or("invalid units per em")
            .unwrap() as f32;

        let glyph_size = glyph_scale * units_per_em;

        let glyph_baseline =
            GLYPH_MARGIN + glyph_height * (bbox.y_max / (bbox.y_max - bbox.y_min)) as f32;

        dbg!(&glyph_scale, &glyph_baseline);

        (glyph_scale, glyph_baseline, glyph_size)
    }
}
