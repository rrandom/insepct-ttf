use iced::{
    canvas::{self, Cache, Canvas, Cursor, Event, Frame, Geometry, Stroke, Text},
    mouse, Color, Element, Length, Point, Rectangle, Vector,
};

#[derive(Default)]
pub struct State {
    cache: Cache,
}
#[derive(Debug, Clone)]
pub enum OverviewMessage {
    ClickGlyph(i8),
}

const ROWS: i8 = 5;
const COLUMNS: i8 = 10;
const CELL_SIZE: f32 = 60.0;

impl State {
    pub fn view<'a>(
        &'a mut self,
        font: &'a owned_ttf_parser::Font,
        width: u16,
        height: u16,
    ) -> Element<'a, OverviewMessage> {
        Canvas::new(OverviewCanvas { state: self, font })
            .width(Length::Units(width))
            .height(Length::Units(height))
            .into()
    }

    pub fn request_redraw(&mut self) {
        self.cache.clear()
    }
}

struct OverviewCanvas<'a> {
    state: &'a mut State,
    font: &'a owned_ttf_parser::Font<'a>,
}

impl<'a> canvas::Program<OverviewMessage> for OverviewCanvas<'a> {
    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let items = self.state.cache.draw(bounds.size(), |frame| {
            self.draw_grid(frame);

            let n = ((ROWS * COLUMNS) as u16).min(self.font.number_of_glyphs());
            let mut _row = 0;
            let mut column = 0;

            let scale = CELL_SIZE / self.font.height() as f32;
            frame.scale(scale, scale);
            frame.scale(1.0, -1.0);

            for id in 0..n {
                let id_txt = Text {
                    content: format!("{}", id),
                    size: 12.0,
                    color: Color::from_rgb(0.2, 0.2, 0.2),
                    position: Point::new(3.0 / scale, -(CELL_SIZE - 13.0) / scale),
                    ..Default::default()
                };

                frame.fill_text(id_txt);

                let (bbox, outline) =
                    super::utils::get_bbox_outline(self.font, owned_ttf_parser::GlyphId(id));

                if let Some(bbox) = bbox {
                    let bbox_w = (bbox.x_max - bbox.x_min) as f32;

                    let dx = (CELL_SIZE / scale - bbox_w as f32) / 2.0;
                    let dy = CELL_SIZE / scale + self.font.descender() as f32;

                    frame.translate(Vector::new(dx, -dy));
                    frame.fill(&outline, Color::BLACK);
                    frame.translate(Vector::new(-dx, dy));
                }

                column += 1;
                if column == COLUMNS {
                    column = 0;
                    _row += 1;
                    frame.translate(Vector::new(
                        -self.font.height() as f32 * (COLUMNS - 1) as f32,
                        -self.font.height() as f32,
                    ));
                } else {
                    frame.translate(Vector::new(self.font.height() as f32, 0.0));
                }
            }
        });
        vec![items]
    }

    fn update(
        &mut self,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Option<OverviewMessage> {
        let cursor_position = cursor.position_in(&bounds)?;

        match event {
            Event::Mouse(mouse_event) => {
                match mouse_event {
                    mouse::Event::ButtonPressed(mouse::Button::Left) => {
                        let row = cursor_position.y / CELL_SIZE;
                        let col = cursor_position.x / CELL_SIZE;
        
                        let id = row as i8 * COLUMNS + col as i8;
                        dbg!(cursor_position, id);
        
                        return Some(OverviewMessage::ClickGlyph(id));
                    }

                    _=> ()
                }
            }
        }

        None
    }
}

impl<'a> OverviewCanvas<'a> {
    fn draw_grid(&self, frame: &mut Frame) {
        let width = COLUMNS as f32 * CELL_SIZE;
        let height = ROWS as f32 * CELL_SIZE;
        let mut pb = iced::canvas::path::Builder::new();

        let mut x = 1.0;
        for _ in 0..=COLUMNS {
            pb.move_to(Point::new(x, 0.0));
            pb.line_to(Point::new(x, height));
            x += CELL_SIZE;
        }

        let mut y = 1.0;
        for _ in 0..=ROWS {
            pb.move_to(Point::new(0.0, y));
            pb.line_to(Point::new(width, y));
            y += CELL_SIZE;
        }

        let path = pb.build();

        frame.stroke(&path, Stroke::default());
    }
}
