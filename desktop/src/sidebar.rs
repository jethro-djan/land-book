use iced::widget::{column, container, text};
use iced::{Element, Length, Task, Theme};

pub struct Sidebar {
    pub width: f32,
    pub is_minimised: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Minimise,
}

pub fn update(state: &mut Sidebar, msg: Message) -> Task<Message> {
    match msg {
        Message::Minimise => Task::none(),
    }
}

pub fn view(state: &Sidebar) -> Element<'_, Message> {
    let sidebar_content = column![text!("Side menu")];

    container(sidebar_content)
        .height(Length::Fill)
        .width(Length::Fixed(state.width))
        .style(|theme: &Theme| container::Style {
            background: Some(theme.extended_palette().background.weakest.color.into()),
            ..Default::default()
        })
        .into()
}
