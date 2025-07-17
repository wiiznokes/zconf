use cosmic::{
    Core, Element,
    app::{Settings, Task},
    iced::{Length, alignment},
    widget::{button, row, text},
};
use serde::{Deserialize, Serialize};
use zconf::ConfigManager;

fn main() {
    cosmic::app::run::<AppState>(Settings::default(), ()).unwrap();
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct Config {
    active: bool,
}

struct AppState {
    core: Core,
    config: ConfigManager<Config, zconf::Toml>,
}

#[derive(Debug, Clone)]
enum AppMsg {
    Update,
}

impl cosmic::Application for AppState {
    type Message = AppMsg;
    type Executor = cosmic::executor::Default;
    type Flags = ();
    const APP_ID: &'static str = "identifier";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(core: cosmic::Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let app_state = AppState {
            core,
            config: ConfigManager::new("config.toml"),
        };
        (app_state, Task::none())
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            AppMsg::Update => {
                self.config.update(|config| {
                    config.active = !config.active;
                });
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<Self::Message> {
        row()
            .width(Length::Fill)
            .align_y(alignment::Vertical::Center)
            .push(button::text("Click me").on_press(AppMsg::Update))
            .push(text(format!("{:?}", self.config.data())))
            .into()
    }
}
