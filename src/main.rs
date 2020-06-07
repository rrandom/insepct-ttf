use iced::{
    button, executor, Application, Button, Command, Container, Element, HorizontalAlignment,
    Length, Row, Settings, Text,
};
use std::path::PathBuf;

pub fn main() {
    App::run(Settings::default())
}

mod dialog;

#[derive(Debug, Clone)]
pub enum LoadError {
    FileError,
    FormatError,
}

#[derive(Debug, Clone)]
enum Message {
    OpenFilePressed,
    Loaded(Result<PathBuf, LoadError>),
}

struct App {
    font_path: Option<PathBuf>,
    button_state: button::State,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (App, Command<Self::Message>) {
        let app = App {
            font_path: None,
            button_state: button::State::new(),
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
                dbg!(&r);
                if let Some(v) = r.ok().take() {
                    self.font_path = Some(v);
                }
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


        let path = self.font_path.as_ref().and_then(|v| v.to_str()).unwrap_or("");

        let row = Row::new().spacing(20).push(open_btn).push(project_label).push(Text::new(path));

        row.into()
    }
}
