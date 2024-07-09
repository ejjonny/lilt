use iced::font::Weight;
use iced::widget::canvas::path::lyon_path::geom::euclid::Transform2D;
use iced::widget::canvas::path::lyon_path::geom::Angle;
use iced::widget::canvas::path::lyon_path::math::vector;
use iced::widget::canvas::path::Arc;
use iced::widget::canvas::{self, Frame, Geometry, Path, Program, Stroke};
use iced::widget::{horizontal_space, text, vertical_space, Column, Container, Row, Space, Stack};
use iced::window::frames;
use iced::{mouse, Color, Font, Point, Rectangle, Renderer, Task};
use iced::{Element, Length, Theme};
use lilt::Animated;
use lilt::Easing;
use std::f32::consts::PI;
use std::time::Instant;

pub fn main() -> iced::Result {
    iced::application("Iced Demo", Example::update, Example::view)
        .font(include_bytes!("OverusedGrotesk-Black.ttf"))
        .subscription(Example::subscription)
        .run()
}

struct Example {
    spinner_trim: Animated<bool, Instant>,
    spinner_rotation: Animated<bool, Instant>,
    bars: Vec<Animated<bool, Instant>>,
}

#[derive(Debug, Clone, Copy)]
enum AppMessage {
    Tick,
}

impl Default for Example {
    fn default() -> Self {
        Self::new()
    }
}

impl Example {
    fn new() -> Self {
        let time = std::time::Instant::now();
        let left: Vec<Animated<bool, Instant>> = (0..50)
            .map(|i| {
                Animated::new(false)
                    .duration(800.)
                    .easing(Easing::EaseInOutBounce)
                    .delay(i as f32 * 30.)
                    .repeat_forever()
                    .auto_start(true, time)
            })
            .rev()
            .collect();
        let right: Vec<Animated<bool, Instant>> = (0..50)
            .map(|i| {
                Animated::new(false)
                    .duration(800.)
                    .easing(Easing::EaseInOutBounce)
                    .delay(i as f32 * 30.)
                    .repeat_forever()
                    .auto_start(true, time)
            })
            .collect();
        Self {
            spinner_trim: Animated::new(false)
                .duration(900.)
                .repeat_forever()
                .auto_reverse()
                .auto_start(true, time),
            spinner_rotation: Animated::new(false)
                .easing(Easing::Linear)
                .duration(900.)
                .repeat_forever()
                .auto_start(true, time),
            bars: [left, right].concat(),
        }
    }

    fn subscription(&self) -> iced::Subscription<AppMessage> {
        frames().map(|_| AppMessage::Tick)
    }

    fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        match message {
            AppMessage::Tick => (),
        }
        Task::none()
    }

    fn view(&self) -> Element<AppMessage> {
        let time = std::time::Instant::now();
        let mut overused_font: Font = Font::with_name("Overused Grotesk Roman");
        overused_font.weight = Weight::Black;
        Stack::new()
            .push(
                Row::new()
                    .extend(
                        self.bars
                            .iter()
                            .map(|b| {
                                Container::new(Space::new(
                                    Length::Fill,
                                    b.animate_bool(10., 300., time),
                                ))
                                .style(move |_| {
                                    iced::widget::container::Style::default().with_background(
                                        Color::from_rgb8(
                                            b.animate_bool(0., 108., time) as u8,
                                            b.animate_bool(0., 74., time) as u8,
                                            b.animate_bool(0., 181., time) as u8,
                                        ),
                                    )
                                })
                                .into()
                            })
                            .collect::<Vec<_>>(),
                    )
                    .height(Length::Fill)
                    .align_items(iced::Alignment::Center),
            )
            .push(
                Column::new()
                    .push(vertical_space())
                    .push(
                        Row::new()
                            .push(horizontal_space())
                            .push(text("lilt").font(overused_font).size(100.))
                            .push(horizontal_space()),
                    )
                    .push(vertical_space()),
            )
            .push(
                Column::new()
                    .push(vertical_space())
                    .push(
                        Row::new()
                            .push(horizontal_space())
                            .push(
                                iced::widget::canvas(Spinner {
                                    trim: self.spinner_trim.animate_bool(0.5, 0., time),
                                    rotation: self.spinner_rotation.animate_bool(0., 1., time),
                                })
                                .height(Length::Fixed(250.))
                                .width(Length::Fixed(250.)),
                            )
                            .push(horizontal_space()),
                    )
                    .push(vertical_space()),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

#[derive(Debug)]
struct Spinner {
    trim: f32,
    rotation: f32,
}

impl Program<AppMessage> for Spinner {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let stroke = 30.;
        let radius = (f32::min(bounds.width, bounds.height) * 0.5) - (stroke * 0.5);
        let circle = Path::new(|p| {
            p.arc(Arc {
                center: Point::new(bounds.center_x() - bounds.x, bounds.center_y() - bounds.y),
                radius,
                start_angle: iced::Radians(0.),
                end_angle: iced::Radians(self.trim * 2. * PI),
            });
        })
        .transform(
            &Transform2D::identity()
                .then_translate(vector(-bounds.width * 0.5, -bounds.height * 0.5))
                .then_rotate(Angle::radians(self.rotation * 2. * PI))
                .then_translate(vector(bounds.width * 0.5, bounds.height * 0.5)),
        );
        // let _debug_frame = Path::rectangle(Point::ORIGIN, bounds.size());
        // frame.fill(&_debug_frame, Color::from_rgb8(255, 0, 0));
        frame.stroke(
            &circle,
            Stroke::default()
                .with_width(stroke)
                .with_line_cap(canvas::LineCap::Round)
                .with_color(Color::WHITE),
        );
        vec![frame.into_geometry()]
    }
}
