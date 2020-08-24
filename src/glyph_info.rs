use iced::Element;

#[derive(Clone, Debug)]
pub struct Outline(pub Vec<DrawType>);

#[derive(Clone, Debug)]
pub enum DrawType {
    MoveTo {
        x: f32,
        y: f32,
    },
    LineTo {
        x: f32,
        y: f32,
    },
    QuadTo {
        x1: f32,
        y1: f32,
        x: f32,
        y: f32,
    },
    CurveTo {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x: f32,
        y: f32,
    },
    Close,
}

impl owned_ttf_parser::OutlineBuilder for Outline {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.push(DrawType::MoveTo { x, y });
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.0.push(DrawType::LineTo { x, y });
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.push(DrawType::QuadTo { x1, y1, x, y });
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.0.push(DrawType::CurveTo {
            x1,
            y1,
            x2,
            y2,
            x,
            y,
        });
    }

    fn close(&mut self) {
        self.0.push(DrawType::Close);
    }
}

#[derive(Default)]
pub struct GlyphInfo {
    pub bbox: Option<owned_ttf_parser::Rect>,
    pub id: owned_ttf_parser::GlyphId,
    pub outline: Option<Outline>,
}

impl GlyphInfo {
    pub fn next(&mut self) {
        self.id.0 = self.id.0 + 1;
    }

    pub fn prev(&mut self) {
        self.id.0 = self.id.0 - 1;
    }

    pub fn view<'a>(&'a self) -> Element<'a, ()> {
        let mut c = iced::Column::new();

        if let Some(rect) = self.bbox {
            c = c
                .push(iced::Text::new(format!("Index {}", self.id.0)))
                .push(iced::Text::new(format!("xMin {}", &rect.x_min)))
                .push(iced::Text::new(format!("xMax {}", &rect.x_max)))
                .push(iced::Text::new(format!("yMin {}", &rect.y_min)))
                .push(iced::Text::new(format!("yMax {}", &rect.y_max)));
        }

        c.into()
    }
}
