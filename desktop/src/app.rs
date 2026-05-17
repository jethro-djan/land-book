use iced::widget::{container, row};
use iced::{Element, Task, Theme, Vector};

use crate::map;
use crate::sidebar;
use corelib::tile_service::MbTiles;

pub struct App {
    pub sidebar_state: sidebar::Sidebar,
    pub map_state: map::Map,
}

#[derive(Debug, Clone)]
pub enum Message {
    Sidebar(sidebar::Message),
    Map(map::Message),
}

pub fn new() -> App {
    let db = MbTiles::open_local(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/kumasi_tiles.mbtiles"
    ))
    .expect("Failed to open .mbtiles database");

    let zoom = 14;
    let window_radius = 3u32;
    let center_tile = map::center_existing_tile_at_zoom(&db, zoom)
        .expect("No tiles found at starting point");

    let tiles = map::load_window(&db, zoom, center_tile, window_radius);

    let map_state = map::Map {
        db,
        zoom: 14,
        extent: 4096.0,
        tiles,
        offset: Vector::new(0.0, 0.0),
        center_tile: center_tile,
        window_radius,
        cached_tile_pixel_size: std::cell::Cell::new(0.0),
    };

    // map_state.tiles = map::load_window(&map_state.db, map_state.zoom, center_tile, window_radius);

    App {
        sidebar_state: sidebar::Sidebar {
            width: 240.0,
            is_minimised: false,
        },
        map_state,
    }
}

pub fn update(state: &mut App, msg: Message) -> Task<Message> {
    match msg {
        Message::Sidebar(msg) => Task::none(),
        Message::Map(msg) => map::update(&mut state.map_state, msg).map(Message::Map),
    }
}

pub fn view(state: &App) -> Element<'_, Message> {
    // container(row![
    //     sidebar::view(&state.sidebar_state).map(Message::Sidebar),
    //     map::view(&state.map_state).map(Message::Map),
    // ])
    container(map::view(&state.map_state).map(Message::Map))
        .style(|theme: &Theme| container::Style {
            background: Some(theme.extended_palette().background.weakest.color.into()),
            ..Default::default()
        })
        .into()
}
