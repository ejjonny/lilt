use animator::animated::Animated;
use animator::animated::Timing;
use animator::traits::AnimationTime;
use iced::executor;
use iced::widget::{horizontal_space, vertical_space, Button, Column, Row, Text};
use iced::window::frames;
use iced::{Application, Command, Element, Length, Settings, Theme};
use std::marker::PhantomData;

pub fn main() -> iced::Result {
    Example::<std::time::Instant>::run(Settings::with_flags(AppFlags {
        now: Box::new(|| Box::new(|| std::time::Instant::now())),
    }))
}

struct Example<Time>
where
    Time: AnimationTime,
{
    animated_toggle: Animated<bool, Time>,
    now: Box<dyn Fn() -> Box<dyn Fn() -> Time>>,
    _phantom: PhantomData<Time>,
}

#[derive(Debug, Clone, Copy)]
enum AppMessage {
    Animate,
    Tick,
}

struct AppFlags<Time>
where
    Time: AnimationTime,
{
    now: Box<dyn Fn() -> Box<dyn Fn() -> Time>>,
}

impl<Time> Application for Example<Time>
where
    Time: AnimationTime + 'static,
{
    type Message = AppMessage;
    type Flags = AppFlags<Time>;
    type Executor = executor::Default;
    type Theme = Theme;

    fn new(flags: Self::Flags) -> (Self, Command<AppMessage>) {
        (
            Self {
                animated_toggle: Animated::new(false, 300., Timing::EaseOut),
                now: flags.now,
                _phantom: PhantomData,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Animator")
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        let now = (self.now)()();
        if self.animated_toggle.in_progress(now) {
            frames().map(|_| AppMessage::Tick)
        } else {
            iced::Subscription::none()
        }
    }

    fn update(&mut self, message: AppMessage) -> Command<AppMessage> {
        let now = (self.now)()();
        match message {
            AppMessage::Animate => self
                .animated_toggle
                .transition(!self.animated_toggle.value, now),
            AppMessage::Tick => (),
        }
        Command::none()
    }

    fn view(&self) -> Element<AppMessage> {
        let now = (self.now)()();
        Column::new()
            .align_items(iced::Alignment::Center)
            .push(vertical_space())
            .push(
                Button::new(
                    Row::new()
                        .push(horizontal_space())
                        .push(Text::new("Animate!"))
                        .push(horizontal_space()),
                )
                .on_press(AppMessage::Animate)
                .width(self.animated_toggle.interpolate(100., 500., now)),
            )
            .push(vertical_space())
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

// impl Interpolable for Color {
//     fn interpolated(self, other: Self, ratio: f32) -> Self {
//         if ratio >= 1.0 {
//             return other;
//         } else if ratio > 0.0 {
//             return Color::new(
//                 self.r.interpolated(other.r, ratio),
//                 self.g.interpolated(other.g, ratio),
//                 self.b.interpolated(other.b, ratio),
//                 self.a.interpolated(other.a, ratio),
//             );
//         } else {
//             return self;
//         }
//     }
// }
