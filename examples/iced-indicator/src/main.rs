use iced::border::Radius;
use iced::font::Weight;
use iced::widget::canvas::path::lyon_path::geom::euclid::Transform2D;
use iced::widget::canvas::path::lyon_path::geom::Angle;
use iced::widget::canvas::path::lyon_path::math::vector;
use iced::widget::canvas::path::Arc;
use iced::widget::canvas::{self, Frame, Geometry, Path, Program, Stroke};
use iced::widget::{center, container, svg, text, vertical_space, Container, Row, Stack};
use iced::window::frames;
use iced::{
    mouse, Background, Border, Color, Font, Point, Rectangle, Renderer, Subscription, Task,
};
use iced::{Element, Length, Theme};
use lilt::{Animated, FloatRepresentable};
use lilt::{Easing, Interpolable};
use std::default::Default;
use std::f32::consts::PI;
use std::time::{Duration, Instant};

pub fn main() -> iced::Result {
    iced::application("Iced Demo", Example::update, Example::view)
        .subscription(Example::subscription)
        .run()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IndicatorState {
    Analyzing,
    Safe,
    Warning,
}

impl FloatRepresentable for IndicatorState {
    fn float_value(&self) -> f32 {
        match self {
            IndicatorState::Analyzing => 0.,
            IndicatorState::Safe => 1.,
            IndicatorState::Warning => 2.,
        }
    }
}

struct Example {
    spinner_rotation: Animated<bool, Instant>,
    spinner_rotation_speed: Animated<bool, Instant>,
    indicator_state: Animated<IndicatorState, Instant>,
}

#[derive(Debug, Clone, Copy)]
enum AppMessage {
    Tick,
    UpdateStatus,
}

impl Default for Example {
    fn default() -> Self {
        Self::new()
    }
}

impl Example {
    fn new() -> Self {
        let time = std::time::Instant::now();
        Self {
            spinner_rotation: Animated::new(false)
                .easing(Easing::Linear)
                .duration(300.)
                .repeat_forever()
                .auto_start(true, time),
            spinner_rotation_speed: Animated::new(false)
                .easing(Easing::EaseInOut)
                .duration(500.)
                .repeat_forever()
                .auto_reverse()
                .auto_start(true, time),
            indicator_state: Animated::new(IndicatorState::Analyzing)
                .easing(Easing::Custom(|l| {
                    if l < 0.5 {
                        Easing::EaseInOutCirc.value(l)
                    } else {
                        Easing::EaseInOutElastic.value(l)
                    }
                }))
                .duration(200.),
        }
    }

    fn subscription(&self) -> iced::Subscription<AppMessage> {
        Subscription::batch(vec![
            iced::time::every(std::time::Duration::from_millis(2000))
                .map(|_| AppMessage::UpdateStatus),
            frames().map(|_| AppMessage::Tick),
        ])
    }

    fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        let time = std::time::Instant::now();
        match message {
            AppMessage::Tick => (),
            AppMessage::UpdateStatus => match self.indicator_state.value {
                IndicatorState::Analyzing => {
                    self.indicator_state.transition(IndicatorState::Safe, time)
                }
                IndicatorState::Safe => self
                    .indicator_state
                    .transition(IndicatorState::Warning, time),
                IndicatorState::Warning => self
                    .indicator_state
                    .transition(IndicatorState::Analyzing, time),
            },
        }
        Task::none()
    }

    fn view(&self) -> Element<AppMessage> {
        let time = std::time::Instant::now();
        let backing_color_analyzing = Color::from_rgb8(187, 218, 252);
        let fg_color_analyzing = Color::from_rgb8(96, 162, 241);
        let capsule_color_analyzing = Color::from_rgb8(230, 242, 254);

        let backing_color_warning = Color::from_rgb8(187, 218, 252);
        let fg_color_warning = Color::from_rgb8(235, 75, 67);
        let capsule_color_warning = Color::from_rgb8(250, 226, 227);

        let backing_color_safe = Color::from_rgb8(187, 218, 252);
        let fg_color_safe = Color::from_rgb8(96, 189, 93);
        let capsule_color_safe = Color::from_rgb8(220, 242, 220);

        let backing_color = self
            .indicator_state
            .animate::<InterpolableColor>(
                |a| match a {
                    IndicatorState::Safe => backing_color_safe.into(),
                    IndicatorState::Warning => backing_color_warning.into(),
                    IndicatorState::Analyzing => backing_color_analyzing.into(),
                },
                time,
            )
            .color;
        let fg_color = self
            .indicator_state
            .animate::<InterpolableColor>(
                |a| match a {
                    IndicatorState::Safe => fg_color_safe.into(),
                    IndicatorState::Warning => fg_color_warning.into(),
                    IndicatorState::Analyzing => fg_color_analyzing.into(),
                },
                time,
            )
            .color;
        let capsule_color = self
            .indicator_state
            .animate::<InterpolableColor>(
                |a| match a {
                    IndicatorState::Safe => capsule_color_safe.into(),
                    IndicatorState::Warning => capsule_color_warning.into(),
                    IndicatorState::Analyzing => capsule_color_analyzing.into(),
                },
                time,
            )
            .color;

        let height = 150.;
        let font = Font {
            weight: Weight::Bold,
            ..Default::default()
        };
        let warn_icon = svg(svg::Handle::from_memory(include_bytes!(
            "../resources/warn.svg"
        )));
        let check_icon = svg(svg::Handle::from_memory(include_bytes!(
            "../resources/check.svg"
        )));

        let icon_small = 0.2;
        let icon_big = 0.4;

        Container::new(center(
            Row::new()
                .push(
                    Stack::new()
                        .push(
                            container(
                                Row::new()
                                    .height(Length::Fixed(height))
                                    .push(vertical_space().width(Length::Fixed(height * 0.3)))
                                    .push(
                                        Stack::new()
                                            .width(Length::Fixed(height * 0.3))
                                            .height(Length::Fixed(height * 0.3))
                                            .push(
                                                iced::widget::canvas(Spinner {
                                                    backing_color,
                                                    spinning_color: fg_color,
                                                    stroke: height * 0.06,
                                                    rotation: self.spinner_rotation.animate(
                                                        |v| if v { 0. } else { 1. },
                                                        time.checked_add(Duration::from_millis(
                                                            self.spinner_rotation_speed
                                                                .animate_bool(0., 150., time)
                                                                as u64,
                                                        ))
                                                        .unwrap_or(time),
                                                    ),
                                                    opacity: self.indicator_state.animate_if_eq(
                                                        IndicatorState::Analyzing,
                                                        1.,
                                                        0.,
                                                        time,
                                                    ),
                                                })
                                                .width(Length::Fixed(
                                                    height
                                                        * self.indicator_state.animate_if_eq(
                                                            IndicatorState::Analyzing,
                                                            icon_big,
                                                            icon_small,
                                                            time,
                                                        ),
                                                ))
                                                .height(Length::Fixed(
                                                    height
                                                        * self.indicator_state.animate_if_eq(
                                                            IndicatorState::Analyzing,
                                                            icon_big,
                                                            icon_small,
                                                            time,
                                                        ),
                                                )),
                                            )
                                            .push(center(
                                                warn_icon
                                                    .width(Length::Fixed(
                                                        height
                                                            * self.indicator_state.animate_if_eq(
                                                                IndicatorState::Warning,
                                                                icon_big,
                                                                icon_small,
                                                                time,
                                                            ),
                                                    ))
                                                    .height(Length::Fixed(
                                                        height
                                                            * self.indicator_state.animate_if_eq(
                                                                IndicatorState::Warning,
                                                                icon_big,
                                                                icon_small,
                                                                time,
                                                            ),
                                                    ))
                                                    .style(move |_, _| iced::widget::svg::Style {
                                                        color: Some(fg_color_warning),
                                                    })
                                                    .opacity(self.indicator_state.animate_if_eq(
                                                        IndicatorState::Warning,
                                                        1.,
                                                        0.,
                                                        time,
                                                    )),
                                            ))
                                            .push(center(
                                                check_icon
                                                    .width(Length::Fixed(
                                                        height
                                                            * self.indicator_state.animate_if_eq(
                                                                IndicatorState::Safe,
                                                                icon_big,
                                                                icon_small,
                                                                time,
                                                            ),
                                                    ))
                                                    .height(Length::Fixed(
                                                        height
                                                            * self.indicator_state.animate_if_eq(
                                                                IndicatorState::Safe,
                                                                icon_big,
                                                                icon_small,
                                                                time,
                                                            ),
                                                    ))
                                                    .style(move |_, _| iced::widget::svg::Style {
                                                        color: Some(fg_color_safe),
                                                    })
                                                    .opacity(self.indicator_state.animate_if_eq(
                                                        IndicatorState::Safe,
                                                        1.,
                                                        0.,
                                                        time,
                                                    )),
                                            )),
                                    )
                                    .push(
                                        Stack::new()
                                            .width(Length::Fixed(
                                                self.indicator_state.animate::<f32>(
                                                    |a| match a {
                                                        IndicatorState::Safe => 150.,
                                                        IndicatorState::Warning => 225.,
                                                        IndicatorState::Analyzing => 700.,
                                                    },
                                                    time,
                                                ),
                                            ))
                                            .push(
                                                text("safe").font(font).size(height * 0.35).color(
                                                    fg_color.scale_alpha(
                                                        self.indicator_state.animate_if_eq(
                                                            IndicatorState::Safe,
                                                            1.,
                                                            0.,
                                                            time,
                                                        ),
                                                    ),
                                                ),
                                            )
                                            .push(
                                                text("warning")
                                                    .font(font)
                                                    .size(height * 0.35)
                                                    .color(fg_color.scale_alpha(
                                                        self.indicator_state.animate_if_eq(
                                                            IndicatorState::Warning,
                                                            1.,
                                                            0.,
                                                            time,
                                                        ),
                                                    )),
                                            )
                                            .push(
                                                text("analyzing transaction")
                                                    .font(font)
                                                    .size(height * 0.35)
                                                    .color(fg_color.scale_alpha(
                                                        self.indicator_state.animate_if_eq(
                                                            IndicatorState::Analyzing,
                                                            1.,
                                                            0.,
                                                            time,
                                                        ),
                                                    )),
                                            ),
                                    )
                                    .push(vertical_space().width(Length::Fixed(height * 0.3)))
                                    .align_y(iced::Alignment::Center)
                                    .spacing(height * 0.2),
                            )
                            .style(move |_| iced::widget::container::Style {
                                border: Border {
                                    color: Color::BLACK,
                                    width: 0.,
                                    radius: Radius::new(height * 0.5),
                                },
                                background: Some(Background::Color(capsule_color)),
                                ..Default::default()
                            })
                            .height(Length::Fixed(height))
                            .width(Length::Shrink),
                        )
                        .width(Length::Shrink)
                        .height(Length::Shrink),
                )
                .padding(30.),
        ))
        .style(move |_| iced::widget::container::Style {
            background: Some(Background::Color(Color::WHITE)),
            ..Default::default()
        })
        .into()
    }
}

impl From<Color> for InterpolableColor {
    fn from(value: Color) -> Self {
        Self { color: value }
    }
}

struct InterpolableColor {
    color: Color,
}

impl Interpolable for InterpolableColor {
    fn interpolated(&self, other: Self, ratio: f32) -> Self {
        let a = self.color;
        let b = other.color;
        let mix = Color::from_rgb(
            a.r + ((b.r - a.r) * ratio),
            a.g + ((b.g - a.g) * ratio),
            a.b + ((b.b - a.b) * ratio),
        );
        Self { color: mix }
    }
}

#[derive(Debug)]
struct Spinner {
    backing_color: Color,
    spinning_color: Color,
    stroke: f32,
    rotation: f32,
    opacity: f32,
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
        let stroke = self.stroke;
        let radius = (f32::min(bounds.width, bounds.height) * 0.5) - (stroke * 0.5);
        let backing = Path::new(|p| {
            p.arc(Arc {
                center: Point::new(bounds.center_x() - bounds.x, bounds.center_y() - bounds.y),
                radius,
                start_angle: iced::Radians(0.),
                end_angle: iced::Radians(2. * PI),
            });
        });
        let circle = Path::new(|p| {
            p.arc(Arc {
                center: Point::new(bounds.center_x() - bounds.x, bounds.center_y() - bounds.y),
                radius,
                start_angle: iced::Radians(0.),
                end_angle: iced::Radians(0.5 * PI),
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
            &backing,
            Stroke::default()
                .with_width(stroke)
                .with_line_cap(canvas::LineCap::Round)
                .with_color(self.backing_color.scale_alpha(self.opacity)),
        );
        frame.stroke(
            &circle,
            Stroke::default()
                .with_width(stroke)
                .with_line_cap(canvas::LineCap::Round)
                .with_color(self.spinning_color.scale_alpha(self.opacity)),
        );
        vec![frame.into_geometry()]
    }
}
