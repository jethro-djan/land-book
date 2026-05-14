use iced;

mod app;

fn main() -> iced::Result {
    iced::application(app::new, app::update, app::view).run()
}
