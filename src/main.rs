use iced::{
    button, executor, Application, Button, Column, Command, Container, Element,
    HorizontalAlignment, Length, Row, Settings, Text,
};
use std::path::PathBuf;

mod dialog;

use dialog::RawFontInfo;

pub fn main() {
    App::run(Settings::default())
}

#[derive(Debug, Clone)]
pub enum LoadError {
    FileError,
    FormatError,
}

#[derive(Debug, Clone)]
pub struct FontInfo {
    family_name: String,
    number_of_glyphs: u16,
}

#[derive(Debug, Clone)]
enum Message {
    OpenFilePressed,
    Loaded(Result<RawFontInfo, LoadError>),
    Parsed(Option<FontInfo>),
}

pub async fn parse_font(data: Vec<u8>) -> Option<FontInfo> {
    match async { return ttf_parser::Font::from_data(&data[..], 0) }.await {
        Some(f) => Some(FontInfo {
            family_name: f.family_name().unwrap(),
            number_of_glyphs: f.number_of_glyphs(),
        }),
        None => None,
    }
}

struct App {
    font_path: Option<PathBuf>,
    button_state: button::State,
    font_info: Option<FontInfo>,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (App, Command<Self::Message>) {
        let app = App {
            font_path: None,
            button_state: button::State::new(),
            font_info: None,
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
                // dbg!(&r);
                if let Some(RawFontInfo { path, data }) = r.ok().take() {
                    self.font_path = Some(path);
                    return Command::perform(parse_font(data), Message::Parsed);
                }
            }
            Message::Parsed(f) => {
                let font = match f {
                    Some(f) => f,
                    None => {
                        eprint!("Error: failed to open a font.");
                        std::process::exit(1);
                    }
                };
                // dbg!(&font);
                self.font_info = Some(font);
            }
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

        let font_name = self
            .font_info
            .as_ref()
            .map(|f| f.family_name.as_str())
            .unwrap_or_default();

        let row = Row::new()
            .spacing(20)
            .push(open_btn)
            .push(project_label)
            .push(Text::new(path));

        let info_row = Row::new()
            .push(Text::new(format!("Font name: {} ;", font_name)))
            .push(Text::new(
                self.font_info
                    .as_ref()
                    .map(|f| format!("number of glyphs: {}", f.number_of_glyphs.to_string()))
                    .unwrap_or_default(),
            ));

        let content = Column::new()
            .max_width(800)
            .spacing(20)
            .push(row)
            .push(info_row);

        Container::new(content)
            .width(Length::Fill)
            .center_x()
            .into()
    }
}
