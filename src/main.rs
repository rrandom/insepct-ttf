use iced::{
    button, executor, window, Align, Application, Button, Column, Command, Container, Element,
    Length, Row, Settings, Text,
};
use owned_ttf_parser::{AsFontRef, OwnedFont};
use std::path::PathBuf;

use dialog::RawFontInfo;

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

        if self.font.is_some() {
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
                        .view(&self.glyph_info, 400, 400)
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

    use super::glyph_info;

    const GLYPH_MARGIN: f32 = 5.0;

    #[derive(Default)]
    pub struct State {
        cache: Cache,
    }

    impl State {
        pub fn view<'a>(
            &'a mut self,
            glyph_info: &'a glyph_info::GlyphInfo,
            width: u16,
            height: u16,
        ) -> Element<'a, ()> {
            Canvas::new(GlyphCanvas {
                state: self,
                glyph_info,
            })
            .width(Length::Units(width))
            .height(Length::Units(height))
            .into()
        }

        pub fn request_redraw(&mut self) {
            self.cache.clear()
        }
    }

    struct GlyphCanvas<'a> {
        state: &'a mut State,
        glyph_info: &'a glyph_info::GlyphInfo,
    }

    impl<'a> canvas::Program<()> for GlyphCanvas<'a> {
        fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
            if let Some(bbox) = self.glyph_info.bbox {
                self.calc_scale(&bounds, &bbox);

                let g = self.state.cache.draw(bounds.size(), |frame| {

                });


                let rect = Rectangle::new(
                    Point::new(bbox.x_min.into(), bbox.y_min.into()),
                    Size::new(bbox.width().into(), bbox.height().into()),
                );

                let glyph = self.state.cache.draw(rect.size(), |frame| {
                    let center = frame.center();

                    let p = Path::new(|p| {
                        use super::glyph_info::DrawType::*;
                        for c in &self.glyph_info.outline.clone().take().unwrap().0 {
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

                    frame.fill(
                        &Path::rectangle(Point::ORIGIN, frame.size()),
                        Color::from_rgba(0.5, 0.5, 0.5, 0.5)
                    );

                });
                return vec![glyph];
            }
            vec![]
        }
    }

    impl<'a> GlyphCanvas<'a> {
        fn calc_scale(&self, bounds: &Rectangle, bbox: &owned_ttf_parser::Rect) {
            let glyph_width = bounds.width - GLYPH_MARGIN*2.0;
            let glyph_height = bounds.height - GLYPH_MARGIN*2.0;

            let glyph_scale = glyph_width/((bbox.x_max - bbox.x_min) as f32).min(glyph_height/(bbox.y_max- bbox.y_min) as f32);

            let glyph_baseline = GLYPH_MARGIN + glyph_height * (bbox.y_max / (bbox.y_max - bbox.y_min)) as f32;

            dbg!(&glyph_scale, &glyph_baseline);
        }
    }
}

mod glyph_info {

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

                // self.outline.as_ref().map(|f| dbg!(f));
            }

            c.into()
        }
    }
}
