use iced::{
    button, executor, window, Align, Application, Button, Column, Command, Container, Element,
    Length, Row, Settings, Text,
};
use owned_ttf_parser::{AsFontRef, OwnedFont};
use std::path::PathBuf;

use dialog::RawFontInfo;

mod dialog;
mod glyph_canvas;
mod glyph_info;

pub fn main() {
    GlyphViewer::run(Settings {
        antialiasing: false,
        window: window::Settings {
            size: (800, 800),
            ..window::Settings::default()
        },
        ..Settings::default()
    });
}

#[derive(Debug, Clone)]
pub enum LoadError {
    FileError,
    FormatError,
}

#[derive(Debug, Clone)]
enum Message {
    OpenFilePressed,
    Loaded(Result<RawFontInfo, LoadError>),
    Empty,
    Next,
    Prev,
}

pub async fn parse_font(data: Vec<u8>) -> Option<owned_ttf_parser::OwnedFont> {
    match async { return OwnedFont::from_vec(data, 0) }.await {
        Some(f) => Some(f),
        None => None,
    }
}

struct GlyphViewer {
    font_path: Option<PathBuf>,
    button_state: button::State,
    glyph: glyph_canvas::State,
    font: Option<OwnedFont>,
    next_button: button::State,
    prev_button: button::State,
    glyph_info: glyph_info::GlyphInfo,
}

impl Application for GlyphViewer {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (GlyphViewer, Command<Self::Message>) {
        let app = GlyphViewer {
            font_path: None,
            button_state: Default::default(),
            glyph: Default::default(),
            glyph_info: Default::default(),
            font: None,
            next_button: Default::default(),
            prev_button: Default::default(),
        };
        (app, Command::none())
    }

    fn title(&self) -> String {
        String::from("TTF-Inspector")
    }

    fn update(&mut self, msg: Self::Message) -> Command<Self::Message> {
        match msg {
            Message::OpenFilePressed => return Command::perform(dialog::open(), Message::Loaded),
            Message::Loaded(r) => {
                if let Some(RawFontInfo { path, data }) = r.ok().take() {
                    self.font_path = Some(path);
                    self.font = OwnedFont::from_vec(data, 0);
                }
            }
            Message::Next => {
                if let Some(font) = &self.font {
                    let total = font.as_font().number_of_glyphs();
                    if self.glyph_info.id.0 < total {
                        self.glyph_info.next();
                        self.update_glyph();
                        self.glyph.request_redraw();
                    }
                }
            }
            Message::Prev => {
                if let Some(_font) = &self.font {
                    if self.glyph_info.id.0 > 0 {
                        self.glyph_info.prev();
                        self.update_glyph();
                        self.glyph.request_redraw();
                    }
                }
            }

            _ => {}
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        let open_btn = Button::new(&mut self.button_state, Text::new("open"))
            .on_press(Message::OpenFilePressed);

        let project_label = Text::new("Font Path: ").width(Length::Shrink);

        let path_txt = self
            .font_path
            .as_ref()
            .and_then(|v| v.to_str())
            .unwrap_or_default();

        let font_name: String = self
            .font
            .as_ref()
            .and_then(|f| f.as_font().family_name())
            .unwrap_or_default();

        let font_path_row = Row::new()
            .spacing(20)
            .push(open_btn)
            .push(project_label)
            .push(Text::new(path_txt))
            .align_items(Align::Center);

        let number_of_glyphs = self
            .font
            .as_ref()
            .map(|f| {
                format!(
                    "number of glyphs: {}",
                    f.as_font().number_of_glyphs().to_string()
                )
            })
            .unwrap_or_default();

        let font_info_row = Row::new()
            .push(Text::new(format!("Font name: {} ; ", font_name)))
            .push(Text::new(number_of_glyphs));

        let content = Column::new()
            .max_width(800)
            .spacing(20)
            .push(font_path_row)
            .push(font_info_row);

        let container = Container::new(content).width(Length::Fill).center_x();

        let mut r = Column::new()
            .padding(20)
            .spacing(10)
            .max_width(800)
            .align_items(Align::Center)
            .push(container);

        if let Some(font) = &self.font {
            let next_btn = Button::new(&mut self.next_button, Text::new("Next"))
                .padding(8)
                .on_press(Message::Next);

            let prev_btn = Button::new(&mut self.prev_button, Text::new("Prev"))
                .padding(8)
                .on_press(Message::Prev);

            let row = Row::new()
                .height(Length::Fill)
                .max_width(800)
                .push(
                    self.glyph
                        .view(&font.as_font(), &self.glyph_info, 400, 400)
                        .map(|_| Message::Empty),
                )
                .push(self.glyph_info.view().map(|_| Message::Empty));

            r = r.push(next_btn).push(prev_btn).push(row)
        }

        r.into()
    }
}

impl GlyphViewer {
    fn update_glyph(&mut self) {
        if let Some(font) = &self.font {
            let mut b = glyph_info::Outline(Vec::new());
            self.glyph_info.bbox = font.as_font().outline_glyph(self.glyph_info.id, &mut b);
            self.glyph_info.outline = Some(b);
        }
    }
}
