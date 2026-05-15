use iced::border::width;
use iced::widget::{Canvas, Container, canvas, column, container, row, text};
use iced::{Element, Length, Renderer, Task, Theme, mouse};

pub struct Map {
    pub tile_index: TileIndex,
    pub size: iced::Size,
    pub sidebar_state: Sidebar,
}

pub struct Sidebar {
    pub width: f32,
    pub is_minimised: bool,
}

impl<Message> canvas::Program<Message> for Map {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        vec![frame.into_geometry()]
    }
}

pub struct TileIndex {
    pub z: f32,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone)]
pub enum Message {
    Zoom,
    Pan,
}

pub fn new() -> Map {
    Map {
        tile_index: TileIndex {
            z: 1.0,
            x: 2.0,
            y: 2.0,
        },
        size: iced::Size::new(800.0, 920.0),
        sidebar_state: Sidebar {
            width: 240.0,
            is_minimised: false,
        },
    }
}

pub fn update(state: &mut Map, msg: Message) -> Task<Message> {
    match msg {
        Message::Pan => Task::none(),
        Message::Zoom => Task::none(),
    }
}

pub fn view(state: &Map) -> Element<'_, Message> {
    container(row![view_sidebar(&state.sidebar_state), canvas(new()),])
        .style(|theme: &Theme| container::Style {
            background: Some(theme.extended_palette().background.weakest.color.into()),
            ..Default::default()
        })
        .into()
}

fn view_sidebar<'a>(sidebar_state: &'a Sidebar) -> Container<'a, Message> {
    let sidebar_content = column![text!("Side menu")];

    container(sidebar_content)
        .height(Length::Fill)
        .width(Length::Fixed(sidebar_state.width))
}
