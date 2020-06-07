use std::path::PathBuf;
use iced::{executor, Application, Command, Element, Settings, Text, Row, Length,HorizontalAlignment};

pub fn main() {
    App::run(Settings::default())
}

#[derive(Debug)]
enum Message {
    OpenFilePressed
}

struct App {
    font_path: Option<PathBuf>,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (App, Command<Self::Message>) {
        (App { font_path: None }, Command::none())
    }

    fn title(&self) -> String {
        String::from("TTF-Inspector")
    }

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        let project_label = Text::new("Font Path: ")
        .width(Length::Shrink)
        .size(32)
        .color([0.5, 0.5, 0.5])
        .horizontal_alignment(HorizontalAlignment::Left);

        project_label.into()
    }

}