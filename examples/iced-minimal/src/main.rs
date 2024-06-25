use iced::widget::{horizontal_space, vertical_space, Button, Column, Row, Text};
use iced::window::frames;
use iced::{executor, Color};
use iced::{Application, Command, Element, Length, Settings, Theme};
use lilt::Easing;
use lilt::{Animated, Interpolable};
use std::time::Instant;

pub fn main() {}

// pub fn main() -> iced::Result {
//     Example::run(Settings::default())
// }

// struct Example {
//     animated_toggle: Animated<bool, Instant>,
// }

// #[derive(Debug, Clone, Copy)]
// enum AppMessage {
//     Animate,
//     Tick,
// }

// impl Application for Example {
//     type Executor = executor::Default;
//     type Message = AppMessage;
//     type Theme = Theme;
//     type Flags = ();

//     fn new(_flags: Self::Flags) -> (Self, Command<AppMessage>) {
//         (
//             Self {
//                 animated_toggle: Animated::new(false, 300., Easing::EaseOut, 0.),
//             },
//             Command::none(),
//         )
//     }

//     fn title(&self) -> String {
//         String::from("Animator")
//     }

//     fn subscription(&self) -> iced::Subscription<Self::Message> {
//         let now = std::time::Instant::now();
//         if self.animated_toggle.in_progress(now) {
//             frames().map(|_| AppMessage::Tick)
//         } else {
//             iced::Subscription::none()
//         }
//     }

//     fn update(&mut self, message: AppMessage) -> Command<AppMessage> {
//         let now = std::time::Instant::now();
//         match message {
//             AppMessage::Animate => self
//                 .animated_toggle
//                 .transition(!self.animated_toggle.value, now),
//             AppMessage::Tick => (),
//         }
//         Command::none()
//     }

//     fn view(&self) -> Element<AppMessage> {
//         let now = std::time::Instant::now();
//         Column::new()
//             .align_items(iced::Alignment::Center)
//             .push(vertical_space())
//             .push(
//                 Button::new(
//                     Row::new()
//                         .push(horizontal_space())
//                         .push(Text::new("Animate!"))
//                         .push(horizontal_space()),
//                 )
//                 .on_press(AppMessage::Animate)
//                 .style(iced::theme::Button::custom(self.animated_toggle.animated(
//                     ButtonStyle::new(Color::from_rgb8(255, 0, 0)),
//                     ButtonStyle::new(Color::from_rgb8(0, 0, 255)),
//                     now,
//                 )))
//                 .width(self.animated_toggle.animated(100., 500., now)),
//             )
//             .push(vertical_space())
//             .width(Length::Fill)
//             .height(Length::Fill)
//             .into()
//     }
// }

// struct ButtonStyle {
//     background: IColor,
// }

// impl ButtonStyle {
//     fn new(background: Color) -> Self {
//         Self {
//             background: IColor::new(background),
//         }
//     }
// }

// impl iced::widget::button::StyleSheet for ButtonStyle {
//     type Style = Theme;
//     fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
//         let mut a = iced::widget::button::Appearance::default();
//         a.background = Some(iced::Background::Color(self.background.color));
//         a.text_color = Color::WHITE;
//         a.border = iced::Border::with_radius(10.);
//         return a;
//     }
// }
// impl Interpolable for ButtonStyle {
//     fn interpolated(self, other: Self, ratio: f32) -> Self {
//         Self {
//             background: self.background.interpolated(other.background, ratio),
//         }
//     }
// }

// struct IColor {
//     color: Color,
// }

// impl IColor {
//     fn new(color: Color) -> Self {
//         Self { color }
//     }
// }

// impl Interpolable for IColor {
//     fn interpolated(&self, other: &Self, ratio: f32) -> &Self {
//         if ratio >= 1.0 {
//             return other;
//         } else if ratio > 0.0 {
//             return &Self {
//                 color: Color::new(
//                     self.color.r.interpolated(&other.color.r, ratio),
//                     self.color.g.interpolated(&other.color.g, ratio),
//                     self.color.b.interpolated(&other.color.b, ratio),
//                     self.color.a.interpolated(&other.color.a, ratio),
//                 ),
//             };
//         } else {
//             return self;
//         }
//     }
// }
