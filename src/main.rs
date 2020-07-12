use iced::{
    button, executor, Align, Application, Button, Column, Command, Container, Element,
    HorizontalAlignment, Length, Row, Settings, Text,
};
use owned_ttf_parser::{AsFontRef, OwnedFont};
use std::path::PathBuf;

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
            }
            Message::Next => {
                self.glyph_id.0 = self.glyph_id.0 + 1;
                self.glyph.request_redraw();
            }
            _ => {}
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

mod dialog {
    use nfd::Response;
    use std::io;
    use std::path::{Path, PathBuf};

    #[derive(Clone, Debug)]
    pub struct RawFontInfo {
        pub path: PathBuf,
        pub data: Vec<u8>,
    }

    pub async fn open_dialog() -> Result<PathBuf, io::Error> {
        let result: nfd::Response =
            match async { return nfd::open_file_dialog(Some("ttf"), None) }.await {
                Ok(result) => result,
                Err(e) => {
                    dbg!(&e);
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Unable to unwrap data from new file dialog",
                    ));
                }
            };

        let file_string: String = match result {
            Response::Okay(file_path) => file_path,
            Response::OkayMultiple(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Multiple files returned when one was expected",
                ))
            }
            Response::Cancel => {
                return Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    "User cancelled file open",
                ))
            }
        };

        let mut result: PathBuf = PathBuf::new();
        result.push(Path::new(&file_string));

        if result.exists() {
            Ok(result)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "File does not exist",
            ))
        }
    }

    pub async fn open() -> Result<RawFontInfo, super::LoadError> {
        use super::LoadError;

        let path = match open_dialog().await {
            Ok(path) => path,
            Err(error) => {
                println!("{:?}", error);
                return Err(LoadError::FileError);
            }
        };

        let font_data = async_std::fs::read(path.as_path()).await.unwrap();

        Ok(RawFontInfo {
            path,
            data: font_data,
        })
    }
}

mod glyph_canvas {

    use iced::{
        canvas::{self, Cache, Canvas, Cursor, Geometry, Path},
        Color, Element, Length, Point, Rectangle, Size, Vector,
    };

    #[derive(Default)]
    pub struct State {
        cache: Cache,
    }

    impl State {
        pub fn view<'a>(
            &'a mut self,
            font: &'a owned_ttf_parser::Font<'a>,
            glyph_id: &'a owned_ttf_parser::GlyphId,
        ) -> Element<'a, ()> {
            Canvas::new(GlyphCanvas {
                state: self,
                font,
                glyph_id,
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        }

        pub fn request_redraw(&mut self) {
            self.cache.clear()
        }
    }

    struct GlyphCanvas<'a> {
        state: &'a mut State,
        font: &'a owned_ttf_parser::Font<'a>,
        glyph_id: &'a owned_ttf_parser::GlyphId,
    }

    impl<'a> canvas::Program<()> for GlyphCanvas<'a> {
        fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
            let mut b = Builder(Vec::new());

            if let Some(bbox) = self.font.outline_glyph(*self.glyph_id, &mut b) {
                let rect = Rectangle::new(
                    Point::new(bbox.x_min.into(), bbox.y_min.into()),
                    Size::new(bbox.width().into(), bbox.height().into()),
                );

                let glyph = self.state.cache.draw(rect.size(), |frame| {
                    let center = frame.center();

                    let p = Path::new(|p| {
                        use DrawType::*;
                        for c in &b.0 {
                            match c {
                                MoveTo { x, y } => p.move_to(Point::new(*x, *y)),
                                LineTo { x, y } => p.line_to(Point::new(*x, *y)),
                                QuadTo { x1, y1, x, y } => {
                                    p.quadratic_curve_to(Point::new(*x1, *y1), Point::new(*x, *y))
                                }
                                CurveTo {
                                    x1,
                                    y1,
                                    x2,
                                    y2,
                                    x,
                                    y,
                                } => {
                                    p.bezier_curve_to(
                                        Point::new(*x1, *y1),
                                        Point::new(*x2, *y2),
                                        Point::new(*x, *y),
                                    );
                                }
                                Close => {
                                    p.close();
                                }
                            }
                        }
                    });

                    let size = bounds.size();

                    let y_scale: f32 = size.height / bbox.height() as f32;
                    let x_scale: f32 = size.width / bbox.width() as f32;

                    frame.translate(Vector::new(center.x, center.y));
                    frame.scale(y_scale.min(x_scale));
                    frame.rotate(std::f32::consts::PI);
                    frame.translate(Vector::new(
                        -1.0 * bbox.x_min as f32,
                        -1.0 * bbox.y_min as f32,
                    ));
                    frame.fill(&p, Color::from_rgb8(0x0, 0x0, 0x0));
                });
                return vec![glyph];
            }
            vec![]
        }
    }

    struct Builder(Vec<DrawType>);

    enum DrawType {
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

    impl owned_ttf_parser::OutlineBuilder for Builder {
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
}
