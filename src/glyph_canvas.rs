
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

          if let Some(bbox) = self.font.outline_glyph(*self.glyph_id, &mut b)
          {

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
                              MoveTo { x, y } => {
                                  p.move_to(Point::new(*x, *y))
                              }
                              LineTo { x, y } => {
                                  p.line_to(Point::new(*x, *y))
                              }
                              QuadTo { x1, y1, x, y } => p
                                  .quadratic_curve_to(
                                      Point::new(*x1, *y1),
                                      Point::new(*x, *y),
                                  ),
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

      fn curve_to(
          &mut self,
          x1: f32,
          y1: f32,
          x2: f32,
          y2: f32,
          x: f32,
          y: f32,
      ) {
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