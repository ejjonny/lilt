use iced::widget::{horizontal_space, vertical_space, Button, Column, Row, Text};
use iced::window::frames;
use iced::Task;
use iced::{Element, Length};
use lilt::Animated;
use lilt::Easing;
use std::time::Instant;

pub fn main() -> iced::Result {
    iced::application("Iced Minimal", Example::update, Example::view)
        .subscription(Example::subscription)
        .run()
}

struct Example {
    animated_toggle: Animated<f32, Instant>,
}

#[derive(Debug, Clone, Copy)]
enum AppMessage {
    Animate,
    Tick,
}

impl Default for Example {
    fn default() -> Self {
        Self::new()
    }
}

impl Example {
    fn new() -> Self {
        Self {
            animated_toggle: Animated::new(3.).duration(1000.).easing(Easing::Linear),
        }
    }

    fn subscription(&self) -> iced::Subscription<AppMessage> {
        let now = std::time::Instant::now();
        if self.animated_toggle.in_progress(now) {
            frames().map(|_| AppMessage::Tick)
        } else {
            iced::Subscription::none()
        }
    }

    fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        let now = std::time::Instant::now();
        match message {
            AppMessage::Animate => self.animated_toggle.transition(
                if self.animated_toggle.value >= 5. {
                    3.
                } else {
                    self.animated_toggle.value + 1.
                },
                now,
            ),
            AppMessage::Tick => (),
        }
        Task::none()
    }

    fn view(&self) -> Element<AppMessage> {
        let now = std::time::Instant::now();
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
                .width(self.animated_toggle.animate(|a| a * 100., now)),
            )
            .push(vertical_space())
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
