use corelib::tile_service::{GeometryType, MbTiles, decode_tile, extract_geometries};
use iced::widget::{Action, canvas, canvas::Path, column, container, slider};
use iced::{Color, Element, Length, Point, Renderer, Task, Theme, Vector, mouse};

#[derive(Clone, Debug)]
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
    pub offset: Vector,
    pub center_tile: (u32, u32),
    pub loaded_center: (u32, u32),
    pub window_radius: u32,
    pub visible_tile_count: f32,
    pub loading: bool,
    pub cached_tile_pixel_size: std::cell::Cell<f32>,
}

#[derive(Default)]
pub struct MapCanvasState {
    pub dragging: Option<Point>,
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
        let tile_dx = tile_x as i64 - min_tx as i64;
        let tile_dy = tile_y as i64 - min_ty as i64;

        for &(lx, ly) in points {
            let gx = tile_dx as f32 * tile_pixel_size + (lx as f32 / self.extent * tile_pixel_size);

            let gy = tile_dy as f32 * tile_pixel_size + (ly as f32 / self.extent * tile_pixel_size);

            let pt = Point::new(gx, gy);

            let circle = Path::circle(pt, 2.0);
            frame.fill(&circle, Color::BLACK);
        }
    }

    pub fn draw_lines(
        &self,
        frame: &mut canvas::Frame<Renderer>,
        lines: &[Vec<(i32, i32)>],
        tile_x: u32,
        tile_y: u32,
        min_tx: u32,
        min_ty: u32,
        tile_pixel_size: f32,
    ) {
        let tile_dx = tile_x as i64 - min_tx as i64;
        let tile_dy = tile_y as i64 - min_ty as i64;

        for line in lines {
            let mut builder = canvas::path::Builder::new();
            let mut points = line.iter();

            if let Some(&first) = points.next() {
                let gx = tile_dx as f32 * tile_pixel_size
                    + (first.0 as f32 / self.extent * tile_pixel_size);

                let gy = tile_dy as f32 * tile_pixel_size
                    + (first.1 as f32 / self.extent * tile_pixel_size);

                builder.move_to(Point::new(gx, gy));

                for &point in points {
                    let gx = tile_dx as f32 * tile_pixel_size
                        + (point.0 as f32 / self.extent * tile_pixel_size);

                    let gy = tile_dy as f32 * tile_pixel_size
                        + (point.1 as f32 / self.extent * tile_pixel_size);

                    builder.line_to(Point::new(gx, gy));
                }
            }

            let path = builder.build();
            frame.stroke(
                &path,
                canvas::Stroke::default()
                    .with_color(Color::BLACK)
                    .with_width(1.0),
            );
        }
    }

    pub fn draw_polygons(
        &self,
        frame: &mut canvas::Frame<Renderer>,
        rings: &[Vec<(i32, i32)>],
        tile_x: u32,
        tile_y: u32,
        min_tx: u32,
        min_ty: u32,
        tile_pixel_size: f32,
    ) {
        let tile_dx = tile_x as i64 - min_tx as i64;
        let tile_dy = tile_y as i64 - min_ty as i64;

        for ring in rings {
            let mut builder = canvas::path::Builder::new();
            let mut points = ring.iter();

            if let Some(&first) = points.next() {
                let gx = tile_dx as f32 * tile_pixel_size
                    + (first.0 as f32 / self.extent * tile_pixel_size);

                let gy = tile_dy as f32 * tile_pixel_size
                    + (first.1 as f32 / self.extent * tile_pixel_size);

                builder.move_to(Point::new(gx, gy));

                for &point in points {
                    let gx = tile_dx as f32 * tile_pixel_size
                        + (point.0 as f32 / self.extent * tile_pixel_size);

                    let gy = tile_dy as f32 * tile_pixel_size
                        + (point.1 as f32 / self.extent * tile_pixel_size);

                    builder.line_to(Point::new(gx, gy));
                }

                builder.close();
            }

            let path = builder.build();
            frame.fill(&path, Color::WHITE);
            frame.stroke(
                &path,
                canvas::Stroke::default()
                    .with_color(Color::BLACK)
                    .with_width(1.0),
            );
        }
    }
}

impl canvas::Program<Message> for Map {
    type State = MapCanvasState;

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        if self.tiles.is_empty() {
            return vec![frame.into_geometry()];
        }

        let min_tx = self.center_tile.0.saturating_sub(self.window_radius);
        let min_ty = self.center_tile.1.saturating_sub(self.window_radius);

        let tile_count_x = (self.window_radius * 2 + 1) as f32;
        let tile_count_y = (self.window_radius * 2 + 1) as f32;

        let tile_pixel_size = (bounds.width / tile_count_x).min(bounds.height / tile_count_y);
        self.cached_tile_pixel_size.set(tile_pixel_size);

        frame.translate(self.offset);

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
                    GeometryType::LineString(lines) => {
                        self.draw_lines(
                            &mut frame,
                            lines,
                            tile.tile_x,
                            tile.tile_y,
                            min_tx,
                            min_ty,
                            tile_pixel_size,
                        );
                    }
                    GeometryType::Polygon(rings) => {
                        self.draw_polygons(
                            &mut frame,
                            rings,
                            tile.tile_x,
                            tile.tile_y,
                            min_tx,
                            min_ty,
                            tile_pixel_size,
                        );
                    }
                }
            }
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut MapCanvasState,
        event: &iced::Event,
        bounds: iced::Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        let cursor_position = cursor.position_in(bounds)?;

        match event {
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                state.dragging = Some(cursor_position);
                None
            }
            iced::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(origin) = state.dragging {
                    let delta =
                        Vector::new(cursor_position.x - origin.x, cursor_position.y - origin.y);
                    state.dragging = Some(cursor_position);
                    Some(Action::publish(Message::Panned(delta)))
                } else {
                    None
                }
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.dragging = None;
                None
            }
            _ => None,
        }
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        _bounds: iced::Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> iced::advanced::mouse::Interaction {
        if state.dragging.is_some() {
            mouse::Interaction::Grabbing
        } else {
            mouse::Interaction::Grab
        }
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
                Some(TileGeometries {
                    tile_x: tx,
                    tile_y: ty,
                    geometries,
                })
            }
        })
        .collect()
}

pub fn load_window(
    db: &MbTiles,
    zoom: u32,
    center: (u32, u32),
    radius: u32,
) -> Vec<TileGeometries> {
    let (cx, cy) = center;
    let x_min = cx.saturating_sub(radius);
    let x_max = cx + radius;
    let y_min = cy.saturating_sub(radius);
    let y_max = cy + radius;

    db.get_tiles_in_range(zoom, x_min, x_max, y_min, y_max)
        .unwrap()
        .iter()
        .filter_map(|tile| {
            let (tx, ty) = (tile.x, tile.y);
            let decoded = decode_tile(&tile);
            let geometries = extract_geometries(decoded);
            if geometries.is_empty() {
                None
            } else {
                Some(TileGeometries {
                    tile_x: tx,
                    tile_y: ty,
                    geometries,
                })
            }
        })
        .collect()
}

pub fn center_existing_tile_at_zoom(db: &MbTiles, zoom: u32) -> Option<(u32, u32)> {
    let tiles = db.get_tiles_at_zoom(zoom).ok()?;

    let min_x = tiles.iter().map(|t| t.x).min()?;
    let max_x = tiles.iter().map(|t| t.x).max()?;
    let min_y = tiles.iter().map(|t| t.y).min()?;
    let max_y = tiles.iter().map(|t| t.y).max()?;

    let target_x = min_x + (max_x - min_x) / 2;
    let target_y = min_y + (max_y - min_y) / 2;

    tiles
        .iter()
        .min_by_key(|t| {
            let dx = t.x as i64 - target_x as i64;
            let dy = t.y as i64 - target_y as i64;
            dx * dx + dy * dy
        })
        .map(|t| (t.x, t.y))
}

pub async fn load_window_async(
    db: MbTiles,
    zoom: u32,
    center: (u32, u32),
    radius: u32,
) -> Vec<TileGeometries> {
    load_window(&db, zoom, center, radius)
}

#[derive(Debug, Clone)]
pub enum Message {
    ZoomLevelChanged(u32),
    Panned(Vector),
    TilesLoaded(Vec<TileGeometries>),
}

pub fn update(state: &mut Map, msg: Message) -> Task<Message> {
    match msg {
        Message::ZoomLevelChanged(zoom) => {
            state.zoom = zoom;
            // state.tiles = get_all_geometries(&state.db, zoom);

            state.offset = Vector::new(0.0, 0.0);

            state.center_tile =
                center_existing_tile_at_zoom(&state.db, zoom).unwrap_or(state.center_tile);

            Task::perform(
                load_window_async(
                    state.db.clone(),
                    state.zoom,
                    state.center_tile,
                    state.window_radius,
                ),
                Message::TilesLoaded,
            )
        }
        Message::Panned(delta) => {
            state.offset.x += delta.x;
            state.offset.y += delta.y;

            let tile_px = state.cached_tile_pixel_size.get();
            if tile_px == 0.0 {
                return Task::none();
            }

            // Convert accumulated offset into whole tile shifts
            let shift_x = (state.offset.x / tile_px).floor() as i64;
            let shift_y = (state.offset.y / tile_px).floor() as i64;

            if shift_x != 0 || shift_y != 0 {
                state.offset.x -= shift_x as f32 * tile_px;
                state.offset.y -= shift_y as f32 * tile_px;

                state.center_tile.0 = (state.center_tile.0 as i64 - shift_x).max(0) as u32;
                state.center_tile.1 = (state.center_tile.1 as i64 - shift_y).max(0) as u32;

                let dx = state.center_tile.0 as i64 - state.loaded_center.0 as i64;
                let dy = state.center_tile.1 as i64 - state.loaded_center.1 as i64;
                let threshold = (state.window_radius / 2).max(1) as i64;

                if (dx.abs() > threshold || dy.abs() > threshold) && !state.loading {
                    state.loading = true;
                    state.loaded_center = state.center_tile;

                    return Task::perform(
                        load_window_async(
                            state.db.clone(),
                            state.zoom,
                            state.center_tile,
                            state.window_radius,
                        ),
                        Message::TilesLoaded,
                    );

                }
            }

            Task::none()
        }
        Message::TilesLoaded(tiles) => {
            state.tiles = tiles;
            state.loading = false;
            Task::none()
        }
    }
}

pub fn view(state: &Map) -> Element<'_, Message> {
    container(column![
        canvas(state).width(Length::Fill).height(Length::Fill),
        slider(1..=14, state.zoom, Message::ZoomLevelChanged).step(1 as u32),
    ])
    .center(Length::Fill)
    .padding(30.0)
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(Color::WHITE)),
        ..Default::default()
    })
    .into()
}
