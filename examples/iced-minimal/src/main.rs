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
    animated_toggle: Animated<bool, Instant>,
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
            animated_toggle: Animated::new(false)
                .duration(100.)
                .easing(Easing::EaseOut)
                .asymmetric_duration(5000.),
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
            AppMessage::Animate => self
                .animated_toggle
                .transition(!self.animated_toggle.value, now),
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
                .width(self.animated_toggle.animate(100., 500., now)),
            )
            .push(vertical_space())
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
