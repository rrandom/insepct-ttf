use iced::{
    button, executor, Align, Application, Button, Column, Command, Container, Element,
    HorizontalAlignment, Length, Row, Settings, Text,
};
use owned_ttf_parser::{AsFontRef, OwnedFont};
use std::path::PathBuf;

mod dialog;
mod glyph_canvas;

use dialog::RawFontInfo;

pub fn main() {
    GlyphViewer::run(Settings::default())
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
    // Parsed(Option<owned_ttf_parser::OwnedFont>),
    Empty,
    Next,
}

pub async fn parse_font(data: Vec<u8>) -> Option<owned_ttf_parser::OwnedFont> {
    match async { return OwnedFont::from_vec(data, 0) }.await {
        Some(f) => Some(f),
        None => None,
    }
}

#[derive(Default)]
struct GlyphViewer {
    font_path: Option<PathBuf>,
    button_state: button::State,
    glyph: glyph_canvas::State,
    font: Option<OwnedFont>,
    button: button::State,
    glyph_id: owned_ttf_parser::GlyphId,
}

impl Application for GlyphViewer {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (GlyphViewer, Command<Self::Message>) {
        let app = Default::default();
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
            },
            Message::Next => {
                self.glyph_id.0 = self.glyph_id.0 + 1;
                self.glyph.request_redraw();
            }
            _ => {},
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        let open_btn = Button::new(&mut self.button_state, Text::new("open"))
            .on_press(Message::OpenFilePressed);

        let project_label = Text::new("Font Path: ")
            .width(Length::Shrink)
            .size(32)
            .color([0.5, 0.5, 0.5])
            .horizontal_alignment(HorizontalAlignment::Left);

        let path = self
            .font_path
            .as_ref()
            .and_then(|v| v.to_str())
            .unwrap_or_default();

        let font_name: String = self
            .font
            .as_ref()
            .and_then(|f| f.as_font().family_name())
            .unwrap_or_default();

        let row = Row::new()
            .spacing(20)
            .push(open_btn)
            .push(project_label)
            .push(Text::new(path));

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

        let info_row = Row::new()
            .push(Text::new(format!("Font name: {} ;", font_name)))
            .push(Text::new(number_of_glyphs));

        let content = Column::new()
            .max_width(800)
            .spacing(20)
            .push(row)
            .push(info_row);

        let container = Container::new(content).width(Length::Fill).center_x();

        let mut r = Column::new()
            .padding(0)
            .spacing(0)
            .align_items(Align::Center)
            .push(container);

            if let Some(f) = &self.font {
                let btn = Button::new(&mut self.button, Text::new("Next"))
                .padding(8)
                .on_press(Message::Next);

                r = r
                .push(Text::new(format!("{}", &self.glyph_id.0)))
                .push(btn)
                .push(
                    self.glyph
                        .view(f.as_font(), &self.glyph_id)
                        .map(|_| Message::Empty),
                );
            }

            r.into()

    }
}
