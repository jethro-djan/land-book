use iced::widget::{container, row};
use iced::{Element, Task, Theme};

use crate::map;
use crate::sidebar;
use corelib::tile_service::GeometryType;
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
    let db = MbTiles::open_local(
        concat!(env!("CARGO_MANIFEST_DIR"), "/data/kumasi_tiles.mbtiles")
    ).expect("Failed to open .mbtiles database");

    let mut map_state = map::Map {
        db,
        extent: 4096.0,
        geometries: vec![],
    };

    map_state.geometries = map_state.get_geometries();

    App {
        sidebar_state: sidebar::Sidebar {
            width: 240.0,
            is_minimised: false,
        },
        map_state,
    }
}

pub fn update(_state: &mut App, msg: Message) -> Task<Message> {
    match msg {
        Message::Sidebar(msg) => Task::none(),
        Message::Map(msg) => Task::none(),
    }
}

pub fn view(state: &App,) -> Element<'_, Message> {
    container(row![
        sidebar::view(&state.sidebar_state).map(Message::Sidebar), 
        map::view(&state.map_state).map(Message::Map),
    ])
    .style(|theme: &Theme| container::Style {
        background: Some(theme.extended_palette().background.weakest.color.into()),
        ..Default::default()
    })
    .into()
}
