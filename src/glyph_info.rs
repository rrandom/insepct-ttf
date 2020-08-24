use iced::Element;

#[derive(Default)]
pub struct GlyphInfo {
    pub bbox: Option<owned_ttf_parser::Rect>,
    pub id: owned_ttf_parser::GlyphId,
    pub outline: Option<iced::canvas::Path>,
}

impl GlyphInfo {
    pub fn next(&mut self) {
        self.id.0 += 1;
    }

    pub fn prev(&mut self) {
        self.id.0 -= 1;
    }

    pub fn view(&self) -> Element<()> {
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
