use cosmic::{
    Core, Element,
    app::{Settings, Task},
    iced::{
        Length, Subscription, alignment,
        futures::{
            SinkExt, Stream, StreamExt,
            channel::mpsc::{self},
            executor::block_on,
            future::pending,
        },
    },
    widget::{button, row, text},
};
use serde::{Deserialize, Serialize};
use zconf::ConfigManager;

#[macro_use]
extern crate log;

fn main() {
    env_logger::init();

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
    ConfigWasUpdated,
    Listening(mpsc::Sender<ConfigSubEvent>),
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
            AppMsg::ConfigWasUpdated => {
                if let Err(e) = self.config.reload() {
                    error!("{e}");
                }
            }
            AppMsg::Listening(mut sender) => {
                println!("listening");

                if let Err(e) = self.config.watch(move || {
                    let _ = block_on(sender.send(ConfigSubEvent::Main));
                }) {
                    error!("{e}")
                }
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

    fn subscription(&self) -> Subscription<Self::Message> {
        cosmic::iced::Subscription::run(stream)
    }
}

enum ConfigSubEvent {
    Main,
}

fn stream() -> impl Stream<Item = AppMsg> {
    cosmic::iced::stream::channel(100, move |mut output| async move {
        let (tx, mut rx) = mpsc::channel::<ConfigSubEvent>(100);

        let _ = output.send(AppMsg::Listening(tx)).await;

        loop {
            match rx.next().await {
                Some(e) => match e {
                    ConfigSubEvent::Main => {
                        let _ = output.send(AppMsg::ConfigWasUpdated).await;
                    }
                },
                None => pending().await,
            }
        }
    })
}
