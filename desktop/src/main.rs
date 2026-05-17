use iced;

mod app;
mod map;
mod sidebar;

fn main() -> iced::Result {
    iced::application(app::new, app::update, app::view).run()
}
