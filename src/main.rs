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
mod overview;
mod utils;

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
    Overview(overview::OverviewMessage),
}

struct GlyphViewer {
    font_path: Option<PathBuf>,
    button_state: button::State,
    overview: overview::State,
    glyph: glyph_canvas::State,
    font: Option<OwnedFont>,
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
            overview: Default::default(),
            glyph: Default::default(),
            glyph_info: Default::default(),
            font: None,
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
            Message::Overview(overview::OverviewMessage::ClickGlyph(id)) => {
                self.glyph_info.id = owned_ttf_parser::GlyphId(id as u16);
                self.update_glyph();
                self.glyph.request_redraw();
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
            let overview_row = Row::new().height(Length::Units(400)).push(
                self.overview
                    .view(&font.as_font(), 602, 602)
                    .map(Message::Overview),
            );

            let row = Row::new()
                .height(Length::Fill)
                .max_width(800)
                .push(
                    self.glyph
                        .view(&font.as_font(), &self.glyph_info, 400, 400)
                        .map(|_| Message::Empty),
                )
                .push(self.glyph_info.view().map(|_| Message::Empty));

            // r = r.push(next_btn).push(prev_btn).push(row)
            r = r.push(overview_row).push(row);
        }

        r.into()
    }
}

impl GlyphViewer {
    fn update_glyph(&mut self) {
        if let Some(font) = &self.font {
            let (bbox, outline) = utils::get_bbox_outline(font.as_font(), self.glyph_info.id);
            self.glyph_info.bbox = bbox;
            self.glyph_info.outline = Some(outline);
        }
    }
}
