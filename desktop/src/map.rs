use corelib::tile_service::{GeometryType, MbTiles, TileWithoutData, decode_tile, extract_geometries};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Task, Theme};
use iced::widget::{canvas, container, canvas::Path};

pub struct Map {
    pub db: MbTiles,
    pub geometries: Vec<GeometryType>,
    pub extent: f32,
}

impl Map {
    pub fn get_geometries(&self) -> Vec<GeometryType> {
        let raw = self.db.get_tile(TileWithoutData { z: 14, x: 8117, y: 8495 }).unwrap();
        let tile = decode_tile(raw);
        let geometries = extract_geometries(tile);

        for g in &geometries {
            if let GeometryType::Point(pts) = g {
                println!("points: {:?}", pts);
            }
        }

        geometries
    }

    pub fn draw_points(&self, frame: &mut canvas::Frame<Renderer>, points: &[(i32, i32)], bounds: iced::Rectangle) {
        for &point in points {
            let x = point.0 as f32 / self.extent * bounds.width;
            let y = point.1 as f32 / self.extent * bounds.height;

            let pt = Point::new(x, y);
            let circle = Path::circle(pt, 2.0);

            frame.fill(&circle, Color::BLACK);
        }
    }
}

impl<Message> canvas::Program<Message> for Map {
    type State = ();

    fn draw(
            &self,
            _state: &Self::State,
            renderer: &Renderer,
            _theme: &Theme,
            bounds: iced::Rectangle,
            _cursor: iced::advanced::mouse::Cursor,
        ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        for geometry in self.geometries.iter() {
            match geometry {
                GeometryType::Point(points) => {
                    self.draw_points(&mut frame, points, bounds);
                }
                _ => (),
            }
        }

        vec![frame.into_geometry()]
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Pan,
    Zoom,
}

pub fn update(state: &mut Map, msg: Message) -> Task<Message> {
    match msg {
        Message::Pan => Task::none(),
        Message::Zoom => Task::none(),
    }
}

pub fn view(state: &Map) -> Element<'_, Message> {
    container(canvas(state))
        .height(Length::Fill)
        .width(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(Color::WHITE)),
            ..Default::default()
        })
        .into()
}
