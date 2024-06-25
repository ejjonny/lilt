use iced::widget::{container, horizontal_space, Button, Column, Row, Space, Text};
use iced::window::frames;
use iced::{executor, Color};
use iced::{Application, Command, Element, Length, Settings, Theme};
use lilt::Animated;
use lilt::Easing;
use std::time::Instant;

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

struct Example {
    bars: Vec<Animated<bool, Instant>>,
}

#[derive(Debug, Clone, Copy)]
enum AppMessage {
    Animate,
    Tick,
}

impl Application for Example {
    type Executor = executor::Default;
    type Message = AppMessage;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<AppMessage>) {
        let left: Vec<Animated<bool, Instant>> = (0..10)
            .map(|i| {
                Animated::new(false)
                    .duration(800.)
                    .easing(Easing::EaseInOutBack)
                    .delay(i as f32 * 30.)
            })
            .rev()
            .collect();
        let right: Vec<Animated<bool, Instant>> = (0..10)
            .map(|i| {
                Animated::new(false)
                    .duration(800.)
                    .easing(Easing::EaseOutBounce)
                    .delay(i as f32 * 30.)
            })
            .collect();
        (
            Self {
                bars: [left, right].concat(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Demo")
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        frames().map(|_| AppMessage::Tick)
    }

    fn update(&mut self, message: AppMessage) -> Command<AppMessage> {
        let now = std::time::Instant::now();
        match message {
            AppMessage::Animate => {
                for bar in &mut self.bars {
                    bar.transition(!bar.value, now)
                }
            }
            AppMessage::Tick => (),
        }
        Command::none()
    }

    fn view(&self) -> Element<AppMessage> {
        let time = std::time::Instant::now();

        Column::new()
            .push(
                Button::new(
                    Row::new()
                        .push(horizontal_space())
                        .push(Text::new("Animate!"))
                        .push(horizontal_space()),
                )
                .on_press(AppMessage::Animate),
            )
            .push({
                let mut bars = Row::new();
                for bar in self.bars.iter() {
                    bars = bars.push(
                        container(Space::new(Length::Fill, bar.animate(10., 300., time)))
                            .style(custom_style()),
                    );
                }
                bars
            })
            .into()
    }
}

fn custom_style() -> iced::theme::Container {
    iced::theme::Container::Custom(Box::new(CustomContainer::new()))
}

pub struct CustomContainer;
impl CustomContainer {
    fn new() -> Self {
        CustomContainer {}
    }
}
impl container::StyleSheet for CustomContainer {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        let mut a = iced::widget::container::Appearance::default();
        a.background = Some(iced::Background::Color(Color::from_rgb8(30, 0, 0)));
        a.border = iced::Border::with_radius(10.);
        return a;
    }
}
