use corelib::tile_service::{GeometryType, MbTiles, TileWithoutData, decode_tile, extract_geometries};
use iced::{Color, Element, Length, Point, Renderer, Task, Theme};
use iced::widget::{canvas, container, canvas::Path, slider, column};

pub struct TileGeometries {
    pub tile_x: u32,
    pub tile_y: u32,
    pub geometries: Vec<GeometryType>,
}

pub struct Map {
    pub db: MbTiles,
    pub tiles: Vec<TileGeometries>,
    pub extent: f32,
    pub zoom: u32,
}

impl Map {

    pub fn draw_points(
        &self, 
        frame: &mut canvas::Frame<Renderer>, 
        points: &[(i32, i32)], 
        tile_x: u32,
        tile_y: u32,
        min_tx: u32,
        min_ty: u32,
        tile_pixel_size: f32,
    ) {
        for &(lx, ly) in points {
            let gx = (tile_x - min_tx) as f32 * tile_pixel_size
                + (lx as f32 / self.extent * tile_pixel_size);
            let gy = (tile_y - min_ty) as f32 * tile_pixel_size
                + (ly as f32 / self.extent * tile_pixel_size);

            let pt = Point::new(gx, gy);

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

        if self.tiles.is_empty() {
            return vec![frame.into_geometry()]
        }

        let min_tx = self.tiles.iter().map(|t| t.tile_x).min().unwrap();
        let max_tx = self.tiles.iter().map(|t| t.tile_x).max().unwrap();
        let min_ty = self.tiles.iter().map(|t| t.tile_y).min().unwrap();
        let max_ty = self.tiles.iter().map(|t| t.tile_y).max().unwrap();

        let tile_count_x = (max_tx - min_tx + 1) as f32;
        let tile_count_y = (max_ty - min_ty + 1) as f32;

        let tile_pixel_size = (bounds.width / tile_count_x)
            .min(bounds.height / tile_count_y);

        for tile in &self.tiles {
            for geometry in &tile.geometries {
                match geometry {
                    GeometryType::Point(points) => {
                        self.draw_points(
                            &mut frame, 
                            points, 
                            tile.tile_x,
                            tile.tile_y,
                            min_tx,
                            min_ty,
                            tile_pixel_size,
                        );
                    }
                    _ => (),
                }
            }
        }

        vec![frame.into_geometry()]
    }
}


pub fn get_all_geometries(db: &MbTiles, zoom: u32) -> Vec<TileGeometries> {
    db.get_tiles_at_zoom(zoom)
        .unwrap()
        .iter()
        .filter_map(|tile| {
            let (tx, ty) = (tile.x, tile.y);
            let decoded = decode_tile(&tile);
            let geometries = extract_geometries(decoded);
            if geometries.is_empty() {
                None 
            } else {
                Some(TileGeometries { tile_x: tx, tile_y: ty, geometries })
            }
        })
        .collect()
}

#[derive(Debug, Clone)]
pub enum Message {
    ZoomLevelChanged(u32),
    Pan,
    Zoom,
}

pub fn update(state: &mut Map, msg: Message) -> Task<Message> {
    match msg {
        Message::ZoomLevelChanged(zoom) => {
            state.zoom = zoom;
            state.tiles = get_all_geometries(&state.db, zoom);
            Task::none()
        }
        Message::Pan => Task::none(),
        Message::Zoom => Task::none(),
    }
}

pub fn view(state: &Map) -> Element<'_, Message> {
    container(
        column![
            canvas(state).width(Length::Fill).height(Length::Fill),
            slider(1..=14, state.zoom, Message::ZoomLevelChanged).step(1 as u32),
        ]
    )
        .center(Length::Fill)
        .padding(30.0)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(Color::WHITE)),
            ..Default::default()
        })
        .into()
}
