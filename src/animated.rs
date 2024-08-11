use crate::traits::{AnimationTime, FloatRepresentable, Interpolable};
/// Wraps state to enable interpolated transitions
///
/// # Example
///
/// ```rust
/// use lilt::Animated;
/// use iced::time::Instant;
///
/// struct MyViewState {
///     animated_toggle: Animated<bool, Instant>,
/// }
/// // Initialize
/// let mut state = MyViewState {
///     animated_toggle: Animated::new(false),
/// };
/// // Update
/// let now = std::time::Instant::now();
/// state
///     .animated_toggle
///     .transition(!state.animated_toggle.value, now);
/// // Animate
/// let animated_width = state.animated_toggle.animate_bool(0., 100., now);
/// let animated_width = state.animated_toggle.animate(
///    |on| if on { 0. } else { 100. },
///    now,
/// );
/// ```
///
/// An `Animated` struct represents a single animation axis. Multiple axes require multiple `Animated` structs.
/// For example - to animate an x and a y position on the screen with different durations you'd need to
/// wrap multiple float values independently.
///
/// ```rust
/// use std::time::Instant;
/// use lilt::Animated;
///
/// struct MyState {
///     animated_x: Animated<f32, Instant>,
///     animated_y: Animated<f32, Instant>,
/// }
/// ```
#[derive(Clone, Debug, Default)]
pub struct Animated<T, Time>
where
    T: FloatRepresentable + Clone + PartialEq,
    Time: AnimationTime,
{
    animation: Animation<Time>,
    pub value: T,
    last_value: T,
}

impl<T, Time> Animated<T, Time>
where
    T: FloatRepresentable + Clone + Copy + PartialEq,
    Time: AnimationTime,
{
    /// Creates an animated value with specified animation settings
    pub fn new_with_settings(value: T, duration_ms: f32, easing: Easing) -> Self {
        let mut animation = Animation::default(value.float_value());
        animation.settings.duration_ms = duration_ms;
        animation.settings.easing = easing;
        Animated {
            value,
            last_value: value,
            animation,
        }
    }
    /// Creates an animated value with a default animation
    pub fn new(value: T) -> Self {
        Self {
            value,
            last_value: value,
            animation: Animation::default(value.float_value()),
        }
    }
    /// Specifies the duration of the animation in milliseconds
    pub fn duration(mut self, duration_ms: f32) -> Self {
        self.animation.settings.duration_ms = duration_ms;
        self
    }
    /// Specifies the easing with which to animate transitions
    pub fn easing(mut self, easing: Easing) -> Self {
        self.animation.settings.easing = easing;
        self
    }
    /// Delays the animation by the given number of milliseconds
    pub fn delay(mut self, delay_ms: f32) -> Self {
        self.animation.delay_ms = delay_ms;
        self
    }
    /// Repeats animations the specified number of times
    /// Passing a repetition count of 1 plays the animation twice in total
    pub fn repeat(mut self, count: u32) -> Self {
        self.animation.repetitions = count;
        self
    }
    /// Repeats transitions forever
    pub fn repeat_forever(mut self) -> Self {
        self.animation.repeat_forever = true;
        self
    }
    /// Automatically play repetitions in reverse after they complete
    pub fn auto_reverse(mut self) -> Self {
        self.animation.auto_reverse_repetitions = true;
        self
    }
    /// Begins a transition as soon as the animation is created
    pub fn auto_start(mut self, new_value: T, at: Time) -> Self {
        self.transition(new_value, at);
        self
    }
    /// Applies an alternative duration while animating backwards
    pub fn asymmetric_duration(mut self, duration_ms: f32) -> Self {
        self.animation.asymmetric_settings = Some(AnimationSettings {
            duration_ms,
            easing: self
                .animation
                .asymmetric_settings
                .map(|a| a.easing)
                .unwrap_or(self.animation.settings.easing),
        });
        self
    }
    /// Applies an alternative easing while animating backwards
    pub fn asymmetric_easing(mut self, easing: Easing) -> Self {
        self.animation.asymmetric_settings = Some(AnimationSettings {
            duration_ms: self
                .animation
                .asymmetric_settings
                .map(|a| a.duration_ms)
                .unwrap_or(self.animation.settings.duration_ms),
            easing,
        });
        self
    }
    /// Updates the wrapped state & begins an animation
    pub fn transition(&mut self, new_value: T, at: Time) {
        if self.value != new_value {
            self.last_value = self.value;
            self.value = new_value;
            self.animation
                .transition(new_value.float_value(), at, false)
        }
    }
    /// Updates the wrapped state & instantaneously completes an animation.
    /// Ignores animation settings such as delay & duration.
    pub fn transition_instantaneous(&mut self, new_value: T, at: Time) {
        if self.value != new_value {
            self.last_value = self.value;
            self.value = new_value;
            self.animation.transition(new_value.float_value(), at, true);
        }
    }
    /// Returns whether the animation is complete, given the current time
    pub fn in_progress(&self, time: Time) -> bool {
        self.animation.in_progress(time)
    }
    /// Interpolates between states of any value that implements `Interpolable`, given the current time
    pub fn animate<I>(&self, map: impl Fn(T) -> I, time: Time) -> I
    where
        I: Interpolable,
    {
        // The generic T values are arbitrary targets that may not be continuous,
        // so we can't store an interrupted T in the case that it's something like
        // an int or enum - therefore we store the interrupted float representation.
        //
        // Given ONLY a function which maps T values to interpolable values,
        // we need some way to go from an interrupt float & a unit progress value
        // to the final interpolable value.
        //
        // The only way to do so without storing interpolable values is to represent
        // the interrupt float (origin) as an interpolable value and interpolate between
        // that and the current destination.
        let interrupted_range = self.value.float_value() - self.last_value.float_value();
        let unit_interrupt_value = if interrupted_range == 0. {
            0.
        } else {
            (self.animation.origin - self.last_value.float_value()) / interrupted_range
        };
        let interrupt_interpolable =
            map(self.last_value).interpolated(map(self.value), unit_interrupt_value);
        interrupt_interpolable
            .interpolated(map(self.value), self.animation.eased_unit_progress(time))
    }
    // Just for nicer testing
    #[allow(dead_code)]
    fn linear_progress(&self, time: Time) -> f32 {
        self.animation.linear_progress(time)
    }
    #[allow(dead_code)]
    fn eased_progress(&self, time: Time) -> f32 {
        self.animation.eased_progress(time)
    }
}

impl<T, Time> Animated<T, Time>
where
    T: FloatRepresentable + Clone + Copy + PartialEq,
    Time: AnimationTime,
{
    /// Interpolates to `equal` when the wrapped value matches the provided `value`
    /// Otherwise interpolatea towards `default`
    pub fn animate_if_eq<I>(&self, value: T, equal: I, default: I, time: Time) -> I
    where
        I: Interpolable + Clone,
    {
        self.animate(
            |v| {
                if v == value {
                    equal.clone()
                } else {
                    default.clone()
                }
            },
            time,
        )
    }
}

impl<Time> Animated<bool, Time>
where
    Time: AnimationTime,
{
    /// Interpolates any value that implements `Interpolable`, given the current time
    pub fn animate_bool<I>(&self, false_value: I, true_value: I, time: Time) -> I
    where
        I: Interpolable + Clone,
    {
        self.animate(
            move |b| {
                if b {
                    true_value.clone()
                } else {
                    false_value.clone()
                }
            },
            time,
        )
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Animation<Time>
where
    Time: AnimationTime,
{
    origin: f32,
    destination: f32,
    delay_ms: f32,
    settings: AnimationSettings,
    asymmetric_settings: Option<AnimationSettings>,
    repetitions: u32,
    auto_reverse_repetitions: bool,
    repeat_forever: bool,
    transition_time: Option<Time>,
}

#[derive(Clone, Copy, Debug, Default)]
struct AnimationSettings {
    duration_ms: f32,
    easing: Easing,
}

impl<Time> Animation<Time>
where
    Time: AnimationTime,
{
    fn default(origin: f32) -> Self {
        Animation {
            origin,
            destination: origin,
            settings: AnimationSettings {
                duration_ms: 100.,
                easing: Easing::EaseInOut,
            },
            asymmetric_settings: None,
            delay_ms: 0.,
            repetitions: 1,
            auto_reverse_repetitions: false,
            repeat_forever: false,
            transition_time: None,
        }
    }

    fn transition(&mut self, destination: f32, time: Time, instantaneous: bool) {
        if self.destination != destination {
            if instantaneous {
                self.origin = destination;
                self.destination = destination;
                return;
            }
            if self.in_progress(time) {
                let eased_progress = self.eased_progress(time);
                self.origin = eased_progress;
            } else {
                self.origin = self.destination;
            }
            self.transition_time = Some(time);
            self.destination = destination;
        }
    }

    fn current_progress(&self, time: Time) -> Progress {
        let Some(transition_time) = self.transition_time else {
            return Progress {
                linear_unit_progress: 0.,
                eased_unit_progress: 0.,
                complete: true,
            };
        };
        let elapsed = f32::max(0., time.elapsed_since(transition_time) - self.delay_ms);

        let settings;
        let elapsed_current;
        let auto_reversing;

        if self.auto_reverse_repetitions {
            let asymmetry = self.asymmetric_settings.unwrap_or(self.settings);
            let combined_durations = self.settings.duration_ms + asymmetry.duration_ms;
            let first_animation = elapsed % combined_durations - self.settings.duration_ms <= 0.;
            if first_animation {
                elapsed_current = elapsed % combined_durations;
                settings = self.settings;
                auto_reversing = false;
            } else {
                settings = asymmetry;
                elapsed_current = elapsed % combined_durations - self.settings.duration_ms;
                auto_reversing = true;
            }
        } else if self.destination.float_value() < self.origin.float_value() {
            settings = self.asymmetric_settings.unwrap_or(self.settings);
            elapsed_current = elapsed;
            auto_reversing = false;
        } else {
            settings = self.settings;
            elapsed_current = elapsed;
            auto_reversing = false;
        }

        let total_duration = self.total_duration();
        if total_duration == 0. {
            return Progress {
                linear_unit_progress: 1.,
                eased_unit_progress: settings.easing.value(1.),
                complete: true,
            };
        }

        let complete = !self.repeat_forever && elapsed >= total_duration;
        let repeat = elapsed_current / settings.duration_ms;
        let progress = if complete { 1. } else { repeat % 1. };
        if auto_reversing && !complete {
            Progress {
                linear_unit_progress: 1. - progress,
                eased_unit_progress: settings.easing.value(1. - progress),
                complete,
            }
        } else {
            Progress {
                linear_unit_progress: progress,
                eased_unit_progress: settings.easing.value(progress),
                complete,
            }
        }
    }

    fn linear_unit_progress(&self, time: Time) -> f32 {
        self.current_progress(time).linear_unit_progress
    }

    fn eased_unit_progress(&self, time: Time) -> f32 {
        self.current_progress(time).eased_unit_progress
    }

    fn total_duration(&self) -> f32 {
        let true_repetitions = if self.auto_reverse_repetitions {
            (self.repetitions * 2) + 1
        } else {
            self.repetitions
        } as f32;
        if true_repetitions > 1. {
            if true_repetitions % 2. == 0. {
                self.settings.duration_ms * (true_repetitions * 0.5)
                    + self
                        .asymmetric_settings
                        .unwrap_or(self.settings)
                        .duration_ms
                        * (true_repetitions * 0.5)
            } else {
                self.settings.duration_ms * ((true_repetitions - true_repetitions % 2.) * 0.5)
                    + self
                        .asymmetric_settings
                        .unwrap_or(self.settings)
                        .duration_ms
                        * ((true_repetitions - true_repetitions % 2.) * 0.5)
                    + self.settings.duration_ms
            }
        } else if self.destination.float_value() < self.origin.float_value() {
            self.asymmetric_settings
                .unwrap_or(self.settings)
                .duration_ms
                * true_repetitions
        } else {
            self.settings.duration_ms * true_repetitions
        }
    }

    fn linear_progress(&self, time: Time) -> f32 {
        self.origin.float_value() + (self.linear_unit_progress(time) * self.progress_range())
    }

    fn eased_progress(&self, time: Time) -> f32 {
        self.origin.float_value() + (self.eased_unit_progress(time) * self.progress_range())
    }

    fn progress_range(&self) -> f32 {
        self.destination.float_value() - self.origin.float_value()
    }

    fn in_progress(&self, time: Time) -> bool {
        !self.current_progress(time).complete
    }
}

struct Progress {
    linear_unit_progress: f32,
    eased_unit_progress: f32,
    complete: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Easing {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,
    Custom(fn(f32) -> f32),
}

impl Easing {
    pub fn value(self, x: f32) -> f32 {
        let pi = std::f32::consts::PI;
        match self {
            Easing::Linear => x,
            Easing::EaseIn => 1.0 - f32::cos((x * pi) / 2.0),
            Easing::EaseOut => f32::sin((x * pi) / 2.0),
            Easing::EaseInOut => -(f32::cos(pi * x) - 1.0) / 2.0,
            Easing::EaseInQuad => x * x,
            Easing::EaseOutQuad => 1.0 - (1.0 - x) * (1.0 - x),
            Easing::EaseInOutQuad => {
                if x < 0.5 {
                    2.0 * x * x
                } else {
                    1.0 - (-2.0 * x + 2.0).powi(2) / 2.0
                }
            }
            Easing::EaseInCubic => x * x * x,
            Easing::EaseOutCubic => 1.0 - (1.0 - x).powi(3),
            Easing::EaseInOutCubic => {
                if x < 0.5 {
                    4.0 * x * x * x
                } else {
                    1.0 - (-2.0 * x + 2.0).powi(3) / 2.0
                }
            }
            Easing::EaseInQuart => x.powi(4),
            Easing::EaseOutQuart => 1.0 - (1.0 - x).powi(4),
            Easing::EaseInOutQuart => {
                if x < 0.5 {
                    8.0 * x * x * x * x
                } else {
                    1.0 - (-2.0 * x + 2.0).powi(4) / 2.0
                }
            }
            Easing::EaseInQuint => x * x * x * x * x,
            Easing::EaseOutQuint => 1.0 - (1.0 - x).powi(5),
            Easing::EaseInOutQuint => {
                if x < 0.5 {
                    16.0 * x * x * x * x * x
                } else {
                    1.0 - (-2.0 * x + 2.0).powi(5) / 2.0
                }
            }
            Easing::EaseInExpo => {
                if x == 0.0 {
                    0.0
                } else {
                    (2.0_f32).powf(10.0 * x - 10.0)
                }
            }
            Easing::EaseOutExpo => {
                if x == 1.0 {
                    1.0
                } else {
                    1.0 - (2.0_f32).powf(-10.0 * x)
                }
            }
            Easing::EaseInOutExpo => match x {
                0.0 => 0.0,
                1.0 => 1.0,
                x if x < 0.5 => (2.0_f32).powf(20.0 * x - 10.0) / 2.0,
                _ => (2.0 - (2.0_f32).powf(-20.0 * x + 10.0)) / 2.0,
            },
            Easing::EaseInCirc => 1.0 - (1.0 - x * x).sqrt(),
            Easing::EaseOutCirc => (1.0 - (x - 1.0).powi(2)).sqrt(),
            Easing::EaseInOutCirc => {
                if x < 0.5 {
                    (1.0 - (1.0 - (2.0 * x).powi(2)).sqrt()) / 2.0
                } else {
                    (1.0 + (1.0 - (-2.0 * x + 2.0).powi(2)).sqrt()) / 2.0
                }
            }
            Easing::EaseInBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                c3 * x * x * x - c1 * x * x
            }
            Easing::EaseOutBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                1.0 + c3 * (x - 1.0).powi(3) + c1 * (x - 1.0).powi(2)
            }
            Easing::EaseInOutBack => {
                let c1 = 1.70158;
                let c2 = c1 * 1.525;
                if x < 0.5 {
                    ((2.0 * x).powi(2) * ((c2 + 1.0) * 2.0 * x - c2)) / 2.0
                } else {
                    ((2.0 * x - 2.0).powi(2) * ((c2 + 1.0) * (x * 2.0 - 2.0) + c2) + 2.0) / 2.0
                }
            }
            Easing::EaseInElastic => {
                let c4 = (2.0 * pi) / 3.0;
                if x == 0.0 {
                    0.0
                } else if x == 1.0 {
                    1.0
                } else {
                    -(2.0_f32.powf(10.0 * x - 10.0)) * f32::sin((x * 10.0 - 10.75) * c4)
                }
            }
            Easing::EaseOutElastic => {
                let c4 = (2.0 * pi) / 3.0;
                if x == 0.0 {
                    0.0
                } else if x == 1.0 {
                    1.0
                } else {
                    2.0_f32.powf(-10.0 * x) * f32::sin((x * 10.0 - 0.75) * c4) + 1.0
                }
            }
            Easing::EaseInOutElastic => {
                let c5 = (2.0 * pi) / 4.5;
                if x == 0.0 {
                    0.0
                } else if x == 1.0 {
                    1.0
                } else if x < 0.5 {
                    -(2.0_f32.powf(20.0 * x - 10.0) * f32::sin((20.0 * x - 11.125) * c5)) / 2.0
                } else {
                    (2.0_f32.powf(-20.0 * x + 10.0) * f32::sin((20.0 * x - 11.125) * c5)) / 2.0
                        + 1.0
                }
            }
            Easing::EaseInBounce => 1.0 - Self::EaseOutBounce.value(1.0 - x),
            Easing::EaseOutBounce => {
                let n1 = 7.5625;
                let d1 = 2.75;
                if x < 1.0 / d1 {
                    n1 * x * x
                } else if x < 2.0 / d1 {
                    n1 * (x - 1.5 / d1).powi(2) + 0.75
                } else if x < 2.5 / d1 {
                    n1 * (x - 2.25 / d1).powi(2) + 0.9375
                } else {
                    n1 * (x - 2.625 / d1).powi(2) + 0.984375
                }
            }
            Easing::EaseInOutBounce => {
                if x < 0.5 {
                    (1.0 - Self::EaseOutBounce.value(1.0 - 2.0 * x)) / 2.0
                } else {
                    (1.0 + Self::EaseOutBounce.value(2.0 * x - 1.0)) / 2.0
                }
            }
            Easing::Custom(f) => f(x),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repeat_forever() {
        let mut anim = Animated::new(0.)
            .duration(1000.)
            .easing(Easing::Linear)
            .repeat_forever();

        anim.transition(10.0, 0.0);

        // Test progression over multiple cycles
        assert_eq!(anim.eased_progress(0.0), 0.0);
        assert_eq!(anim.eased_progress(500.0), 5.0);
        assert_eq!(anim.eased_progress(1000.0), 0.0);
        assert_eq!(anim.eased_progress(1500.0), 5.0);
        assert_eq!(anim.eased_progress(2000.0), 0.0);
        assert_eq!(anim.eased_progress(2500.0), 5.0);

        // Ensure animation is still in progress after multiple cycles
        assert!(anim.in_progress(10000.0));
    }

    fn plot_easing(easing: Easing) {
        const WIDTH: usize = 80;
        const HEIGHT: usize = 40;
        let mut plot = vec![vec![' '; WIDTH]; HEIGHT];

        for x in 0..WIDTH {
            let t = x as f32 / (WIDTH - 1) as f32;
            let y = easing.value(t);
            let y_scaled = ((1.0 - y) * (HEIGHT - 20) as f32).round() as usize + 10;
            let y_scaled = y_scaled.min(HEIGHT - 1);
            plot[y_scaled][x] = '*';
        }

        println!("\nPlot for {:?}:", easing);
        for row in plot {
            println!("{}", row.iter().collect::<String>());
        }
        println!();
    }

    #[test]
    fn visualize_all_easings() {
        let easings = [
            Easing::Linear,
            Easing::EaseIn,
            Easing::EaseOut,
            Easing::EaseInOut,
            Easing::EaseInQuad,
            Easing::EaseOutQuad,
            Easing::EaseInOutQuad,
            Easing::EaseInCubic,
            Easing::EaseOutCubic,
            Easing::EaseInOutCubic,
            Easing::EaseInQuart,
            Easing::EaseOutQuart,
            Easing::EaseInOutQuart,
            Easing::EaseInQuint,
            Easing::EaseOutQuint,
            Easing::EaseInOutQuint,
            Easing::EaseInExpo,
            Easing::EaseOutExpo,
            Easing::EaseInOutExpo,
            Easing::EaseInCirc,
            Easing::EaseOutCirc,
            Easing::EaseInOutCirc,
            Easing::EaseInBack,
            Easing::EaseOutBack,
            Easing::EaseInOutBack,
            Easing::EaseInElastic,
            Easing::EaseOutElastic,
            Easing::EaseInOutElastic,
            Easing::EaseInBounce,
            Easing::EaseOutBounce,
            Easing::EaseInOutBounce,
        ];

        for easing in &easings {
            plot_easing(*easing);
        }
    }

    #[test]
    fn test_custom_easing() {
        let custom_ease = Easing::Custom(|x| x * x); // Quadratic ease-in
        assert_eq!(custom_ease.value(0.0), 0.0);
        assert_eq!(custom_ease.value(0.5), 0.25);
        assert_eq!(custom_ease.value(1.0), 1.0);
    }

    #[test]
    fn test_linear_progress() {
        let mut anim = Animated::new(0.).duration(1000.).easing(Easing::Linear);
        anim.transition(10.0, 0.0);

        assert_eq!(anim.linear_progress(0.0), 0.0);
        assert_eq!(anim.linear_progress(500.0), 5.0);
        assert_eq!(anim.linear_progress(1000.0), 10.0);
        assert_eq!(anim.linear_progress(1500.0), 10.0); // Stays at destination after completion
    }

    #[test]
    fn test_eased_progress_with_easing() {
        let mut anim = Animated::new(0.).duration(1000.).easing(Easing::EaseIn);
        anim.transition(10.0, 0.0);

        assert_eq!(anim.eased_progress(0.0), 0.0);
        assert!(anim.eased_progress(500.0) < 5.0); // Should be less than linear due to ease-in
        assert_eq!(anim.eased_progress(1000.0), 10.0);
    }

    #[test]
    fn test_in_progress() {
        let mut anim = Animated::new(0.).duration(1000.).easing(Easing::EaseIn);
        assert!(!anim.in_progress(0.0));

        anim.transition(10.0, 0.0);
        assert!(anim.in_progress(0.0));
        assert!(anim.in_progress(500.0));
        assert!(!anim.in_progress(1000.0));
    }

    #[test]
    fn test_repetitions() {
        let mut anim = Animated::new(0.)
            .duration(1000.)
            .easing(Easing::Linear)
            .repeat(3);
        anim.transition(10.0, 0.0);

        assert_eq!(anim.linear_progress(1500.0), 5.0); // Middle of second repetition
        assert_eq!(anim.linear_progress(3000.0), 10.0); // End of third repetition
        assert!(!anim.in_progress(5001.));
        assert_eq!(anim.linear_progress(3500.0), 10.0); // Stays at destination after all repetitions
    }

    #[test]
    fn test_auto_reverse_repetitions() {
        let mut anim = Animated::new(0.)
            .duration(1000.)
            .easing(Easing::Linear)
            .auto_reverse()
            .repeat(2);
        anim.transition(10.0, 0.0);

        assert_eq!(anim.linear_progress(500.0), 5.0); // Middle of first forward
        assert_eq!(anim.linear_progress(1500.0), 5.0); // Middle of first reverse
        assert_eq!(anim.linear_progress(2500.0), 5.0); // Middle of second forward
        assert_eq!(anim.linear_progress(3500.0), 5.0); // Middle of second reverse
        assert_eq!(anim.linear_progress(4000.0), 0.0); // End at start position
        assert!(!anim.in_progress(5000.));
    }

    #[test]
    fn test_auto_reverse_repetitions_bool() {
        let anim = Animated::new(false)
            .duration(1000.)
            .auto_start(true, 0.)
            .repeat(2)
            .auto_reverse();
        assert_eq!(anim.linear_progress(500.0), 0.5); // Middle of first forward
        assert_eq!(anim.linear_progress(1500.0), 0.5); // Middle of first reverse
        assert_eq!(anim.linear_progress(2500.0), 0.5); // Middle of second forward
        assert_eq!(anim.linear_progress(3500.0), 0.5); // Middle of second reverse
        assert!(anim.in_progress(3500.));
        assert_eq!(anim.linear_progress(4000.0), 0.0); // End at start position
        dbg!(anim.linear_progress(5000.01));
        assert!(!anim.in_progress(5000.01));
    }

    #[test]
    fn test_delay() {
        let mut anim = Animated::new(0.)
            .duration(1000.)
            .easing(Easing::Linear)
            .delay(500.);
        anim.transition(10.0, 0.0);

        assert_eq!(anim.linear_progress(250.0), 0.0); // Still in delay
        assert_eq!(anim.linear_progress(750.0), 2.5); // 25% progress after delay
        assert_eq!(anim.linear_progress(1500.0), 10.0); // Completed
    }

    #[test]
    fn test_interruption() {
        let mut anim = Animated::new(0.).duration(1000.).easing(Easing::Linear);
        anim.transition(10.0, 0.0);

        assert_eq!(anim.linear_progress(500.0), 5.0);

        anim.transition(20.0, 500.0); // Interrupt halfway
        assert_eq!(anim.animation.origin, 5.0); // New origin should be the current progress
        assert_eq!(anim.linear_progress(1000.0), 12.5); // Halfway to new destination
        assert_eq!(anim.linear_progress(1500.0), 20.0); // Completed to new destination
    }

    #[test]
    fn test_instant_animation() {
        let mut anim = Animated::new(0.).duration(0.).easing(Easing::Linear);
        assert_eq!(anim.linear_progress(0.0), 0.0);
        // If animation duration is 0.0 the transition should happen instantly
        // & require a redraw without any time passing
        anim.transition(10.0, 0.0);
        assert_eq!(anim.linear_progress(0.0), 10.0);
    }

    #[test]
    fn test_progression() {
        let mut anim = Animated::new(0.).duration(1.).easing(Easing::Linear);
        // With a duration of 1.0 & linear timing we should be halfway to our
        // destination at 0.5
        anim.transition(10.0, 0.5);
        assert_eq!(anim.value, 10.);
        assert_eq!(anim.linear_progress(1.0), 5.0);
        assert_eq!(anim.linear_progress(1.5), 10.0);

        // Progression backward
        anim.transition(0.0, 1.5);
        assert_eq!(anim.value, 0.);
        assert_eq!(anim.linear_progress(2.5), 0.0);

        // Progression forward in fractions
        anim.transition(10.0, 3.);
        assert_eq!(anim.value, 10.);
        assert!(approximately_equal(anim.linear_progress(3.), 0.0));
        assert!(approximately_equal(anim.linear_progress(3.2), 2.0));
        assert!(approximately_equal(anim.linear_progress(3.8), 8.0));
        assert!(approximately_equal(anim.linear_progress(4.0), 10.0));
    }

    #[test]
    fn test_progression_negative() {
        let mut anim = Animated::new(0.).duration(1.).easing(Easing::EaseInOut);

        anim.transition(-10.0, 0.0);
        assert_eq!(anim.linear_progress(0.5), -5.0);
        assert_eq!(anim.linear_progress(1.0), -10.0);

        assert!(anim.eased_progress(0.25) > anim.linear_progress(0.25));
        assert!(anim.eased_progress(0.5) == anim.linear_progress(0.5));
        assert!(anim.eased_progress(0.75) < anim.linear_progress(0.75));

        anim.transition(0.0, 1.0);
        assert_eq!(anim.linear_progress(1.5), -5.0);
        assert_eq!(anim.linear_progress(2.0), 0.0);
    }

    #[test]
    fn test_multiple_interrupts_start_forward() {
        let mut anim = Animated::new(0.).duration(1.).easing(Easing::EaseInOut);
        anim.transition(1.0, 0.);
        assert!(anim.in_progress(0.5));
        let progress_at_interrupt = anim.eased_progress(0.5);
        assert_eq!(progress_at_interrupt, Easing::EaseInOut.value(0.5));
        anim.transition(0.0, 0.5);
        assert_eq!(anim.eased_progress(0.5), progress_at_interrupt);
        assert!(anim.in_progress(0.7));
        anim.transition(1.0, 0.7);
        assert!(anim.in_progress(0.9));
    }

    #[test]
    fn test_asymmetric() {
        let mut anim = Animated::new(0.)
            .duration(1000.)
            .easing(Easing::Linear)
            .asymmetric_duration(2000.)
            .asymmetric_easing(Easing::EaseInOut);

        anim.transition(10.0, 0.0);
        assert_eq!(anim.linear_progress(500.0), 5.0); // 50% forward
        assert_eq!(anim.linear_progress(1000.0), 10.); // 100% forward

        anim.transition(0.0, 1000.0);
        assert_eq!(anim.linear_progress(1500.0), 7.5); // 25% backwards
        assert_eq!(anim.linear_progress(2000.0), 5.0); // 50% backwards
        assert_eq!(anim.linear_progress(2500.0), 2.5); // 75% backwards
        assert_eq!(anim.linear_progress(3000.0), 0.0); // 100% backwards

        anim.transition(10.0, 3000.0);
        assert_eq!(anim.linear_progress(3250.0), 2.5); // 25% second forward
        assert_eq!(anim.linear_progress(3500.0), 5.0); // 50% second forward
        assert_eq!(anim.linear_progress(3750.0), 7.5); // 75% second forward
        assert_eq!(anim.linear_progress(4000.0), 10.0); // 100% second forward
    }

    #[test]
    fn test_asymmetric_auto_reversal() {
        let mut anim = Animated::new(0.)
            .duration(1000.)
            .easing(Easing::Linear)
            .asymmetric_duration(2000.)
            .asymmetric_easing(Easing::EaseInOut)
            .auto_reverse()
            .repeat(1);

        anim.transition(10.0, 0.0);

        // ->
        assert_eq!(anim.linear_progress(500.0), 5.0); // 50% forward
        assert_eq!(anim.linear_progress(1000.0), 0.); // 100% forward

        // <-
        assert_eq!(anim.linear_progress(1500.0), 7.5); // 25% backwards
        assert_eq!(anim.linear_progress(2000.0), 5.0); // 50% backwards
        assert_eq!(anim.linear_progress(2500.0), 2.5); // 75% backwards
        assert_eq!(anim.linear_progress(3000.0), 0.0); // 100% backwards

        assert!(anim.eased_progress(1500.0) > anim.linear_progress(1500.0)); // 25% backwards
        assert!(anim.eased_progress(2000.0) == anim.linear_progress(2000.0)); // 50% backwards
        assert!(anim.eased_progress(2500.0) < anim.linear_progress(2500.0)); // 75% backwards

        // ->
        assert_eq!(anim.linear_progress(3250.0), 2.5); // 25% second forward
        assert_eq!(anim.linear_progress(3500.0), 5.0); // 50% second forward
        assert_eq!(anim.linear_progress(3750.0), 7.5); // 75% second forward
        assert_eq!(anim.linear_progress(4000.0), 10.0); // 100% second forward

        assert!(anim.eased_progress(3250.0) == anim.linear_progress(3250.0)); // 25% forward
        assert!(anim.eased_progress(3500.0) == anim.linear_progress(3500.0)); // 50% forward
        assert!(anim.eased_progress(3750.0) == anim.linear_progress(3750.0)); // 75% forward
    }

    #[test]
    fn test_auto_reversal() {
        let mut anim = Animated::new(0.)
            .duration(1000.)
            .easing(Easing::EaseInOut)
            .auto_reverse()
            .repeat(1);

        anim.transition(10.0, 0.0);

        assert_eq!(anim.linear_progress(0.0), 0.0);

        // ->
        assert_eq!(anim.linear_progress(250.0), 2.5); // 25% forward
        assert_eq!(anim.linear_progress(500.0), 5.0); // 50% forward
        assert_eq!(anim.linear_progress(750.0), 7.5); // 75% forward
        assert_eq!(anim.linear_progress(1000.0), 0.0); // 100% forward

        assert!(anim.eased_progress(250.0) < anim.linear_progress(250.0));
        assert!(anim.eased_progress(500.0) == anim.linear_progress(500.0));
        assert!(anim.eased_progress(750.0) > anim.linear_progress(750.0));

        // <-
        assert_eq!(anim.linear_progress(1250.0), 7.5); // 25% backwards
        assert_eq!(anim.linear_progress(1500.0), 5.0); // 50% backwards
        assert_eq!(anim.linear_progress(1750.0), 2.5); // 75% backwards
        assert_eq!(anim.linear_progress(2000.0), 0.0); // 100% backwards

        assert!(anim.eased_progress(1250.0) > anim.linear_progress(1250.0));
        assert!(anim.eased_progress(1500.0) == anim.linear_progress(1500.0));
        assert!(anim.eased_progress(1750.0) < anim.linear_progress(1750.0));

        // ->
        assert_eq!(anim.linear_progress(2250.0), 2.5); // 25% forward
        assert_eq!(anim.linear_progress(2500.0), 5.0); // 50% forward
        assert_eq!(anim.linear_progress(2750.0), 7.5); // 75% forward
        assert_eq!(anim.linear_progress(3000.0), 10.0); // 100% forward
    }

    #[test]
    fn test_transition_instantaneous() {
        let mut anim = Animated::new(0.).duration(1000.).easing(Easing::Linear);
        anim.transition_instantaneous(10., 0.);
        assert_eq!(anim.value, 10.);
        assert_eq!(anim.linear_progress(0.), 10.);
        assert_eq!(anim.linear_progress(1.), 10.);
        anim.transition_instantaneous(0., 1.);
        assert_eq!(anim.value, 0.);
        assert_eq!(anim.linear_progress(1.), 0.);
        assert_eq!(anim.linear_progress(2.), 0.);
        anim.transition(10., 10.);
        assert_eq!(anim.value, 10.);
        assert_eq!(anim.linear_progress(10.), 0.);
        assert_eq!(anim.linear_progress(1010.), 10.);
        anim.transition_instantaneous(0., 1010.);
        assert_eq!(anim.value, 0.);
        assert_eq!(anim.linear_progress(1010.), 0.);
        assert_eq!(anim.linear_progress(1011.), 0.);
        assert_eq!(anim.linear_progress(1020.), 0.);
    }

    #[test]
    fn test_negative_values() {
        let mut anim = Animated::new(0.).duration(1000.).easing(Easing::Linear);
        anim.transition(-10.0, 0.0);
        assert_eq!(anim.linear_progress(0.0), 0.0);
        assert_eq!(anim.linear_progress(500.0), -5.0);
        assert_eq!(anim.linear_progress(1000.0), -10.0);
        anim.transition(-5.0, 1000.0);
        assert_eq!(anim.linear_progress(1000.0), -10.0);
        assert_eq!(anim.linear_progress(1500.0), -7.5);
        assert_eq!(anim.linear_progress(2000.0), -5.0);
    }

    #[test]
    fn test_negative_to_positive_transition() {
        let mut anim = Animated::new(-5.).duration(1000.).easing(Easing::Linear);
        anim.transition(5.0, 0.0);
        assert_eq!(anim.linear_progress(0.0), -5.0);
        assert_eq!(anim.linear_progress(500.0), 0.0);
        assert_eq!(anim.linear_progress(1000.0), 5.0);
    }

    #[test]
    fn test_interruption_with_negative_values() {
        let mut anim = Animated::new(0.).duration(1000.).easing(Easing::Linear);
        anim.transition(-10.0, 0.0);
        assert_eq!(anim.linear_progress(250.0), -2.5);
        anim.transition(5.0, 250.0); // Interrupt at 25%
        assert_eq!(anim.animation.origin, -2.5); // New origin should be the current progress
        assert_eq!(anim.linear_progress(750.0), 1.25); // Halfway to new destination
        assert_eq!(anim.linear_progress(1250.0), 5.0); // Completed to new destination
    }

    #[test]
    fn test_multiple_interruptions() {
        let mut anim = Animated::new(0.).duration(1000.).easing(Easing::Linear);
        anim.transition(10.0, 0.0);
        assert_eq!(anim.linear_progress(500.0), 5.);
        anim.transition(15.0, 500.0); // First interruption
        assert_eq!(anim.linear_progress(1000.0), 10.); // 50% to new destination
        anim.transition(0.0, 1000.0); // Second interruption
        assert_eq!(anim.animation.origin, 10.); // New origin after second interruption
        assert_eq!(anim.linear_progress(1500.0), 5.); // Halfway to final destination
        assert_eq!(anim.linear_progress(2000.0), 0.0); // Completed to final destination
    }

    #[test]
    fn test_interrupt_unchanged_destination() {
        // Interrupts that don't change the destination shouldn't change duration.
        // Despite the majority of other cases where we have to.
        let mut anim_a = Animated::new(0.).duration(1000.).easing(Easing::Linear);
        let mut anim_b = Animated::new(0.).duration(1000.).easing(Easing::Linear);
        anim_a.transition(10., 0.);
        anim_b.transition(10., 0.);

        anim_a.transition(10., 250.);
        assert_eq!(anim_a.linear_progress(250.), anim_b.linear_progress(250.));
        assert_eq!(anim_a.linear_progress(500.), anim_b.linear_progress(500.));
        assert_eq!(anim_a.linear_progress(750.), anim_b.linear_progress(750.));
        assert_eq!(anim_a.linear_progress(1000.), anim_b.linear_progress(1000.));
    }

    #[test]
    fn test_interruption_with_direction_change() {
        let mut anim = Animated::new(0.).duration(1000.).easing(Easing::Linear);
        anim.transition(10.0, 0.0);
        assert_eq!(anim.linear_progress(500.0), 5.0);
        anim.transition(-5.0, 500.0); // Interrupt and change direction
        assert_eq!(anim.animation.origin, 5.0); // New origin should be the current progress
        assert_eq!(anim.linear_progress(1000.0), 0.0); // Halfway back to new destination
        assert_eq!(anim.linear_progress(1500.0), -5.0); // Completed to new destination
    }

    #[test]
    fn test_zero_duration_transition() {
        let mut anim = Animated::new(0.).duration(0.).easing(Easing::Linear);
        anim.transition(10.0, 0.0);
        assert_eq!(anim.linear_progress(0.0), 10.0); // Should immediately reach the destination
        assert!(!anim.in_progress(0.0)); // Should not be in progress
    }

    #[test]
    fn test_interruption_at_completion() {
        let mut anim = Animated::new(0.).duration(1000.).easing(Easing::Linear);
        anim.transition(10.0, 0.0);
        assert_eq!(anim.linear_progress(1000.0), 10.0); // Completed
        anim.transition(20.0, 1000.0); // Interrupt right at completion
        assert_eq!(anim.animation.origin, 10.0); // New origin should be the completed value
        assert_eq!(anim.linear_progress(1500.0), 15.0); // Halfway to new destination
        assert_eq!(anim.linear_progress(2000.0), 20.0); // Completed to new destination
    }

    #[test]
    fn test_animate() {
        let mut anim = Animated::new(0.0f32)
            .duration(1000.0)
            .easing(Easing::Linear);
        anim.transition(10.0, 0.0);

        let result = anim.animate(|v| v, 500.0);
        assert_eq!(result, 5.0);

        let result = anim.animate(|v| v * 2.0, 750.0);
        assert_eq!(result, 15.0);
    }

    #[test]
    fn test_animate_if_eq() {
        let mut anim = Animated::new(0.0f32)
            .duration(1000.0)
            .easing(Easing::Linear);
        anim.transition(10.0, 0.0);

        let result = anim.animate_if_eq(10.0, 100.0, 0.0, 500.0);
        assert_eq!(result, 50.0);

        let result = anim.animate_if_eq(5.0, 100.0, 0.0, 500.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_animate_bool() {
        let mut anim = Animated::new(false).duration(1000.0).easing(Easing::Linear);
        anim.transition(true, 0.0);

        let result = anim.animate_bool(0.0, 10.0, 500.0);
        assert_eq!(result, 5.0);

        let result = anim.animate_bool(0.0, 10.0, 1000.0);
        assert_eq!(result, 10.0);
    }

    #[test]
    fn test_animate_with_interruption() {
        let mut anim = Animated::new(0.0f32)
            .duration(1000.0)
            .easing(Easing::Linear);
        anim.transition(10.0, 0.0);

        let result = anim.animate(|v| v, 500.0);
        assert_eq!(result, 5.0);

        anim.transition(20.0, 500.0);
        let result = anim.animate(|v| v, 1000.0);
        assert_eq!(result, 12.5);

        let result = anim.animate(|v| v, 1500.0);
        assert_eq!(result, 20.0);
    }

    #[test]
    fn test_animate_with_custom_easing() {
        let custom_ease = Easing::Custom(|x| x * x); // Quadratic ease-in
        let mut anim = Animated::new(0.0f32).duration(1000.0).easing(custom_ease);
        anim.transition(10.0, 0.0);

        let result = anim.animate(|v| v, 500.0);
        assert_eq!(result, 2.5); // (0.5^2 * 10)

        let result = anim.animate(|v| v, 750.0);
        assert_eq!(result, 5.625); // (0.75^2 * 10)
    }

    #[test]
    fn test_no_change_after_completion() {
        let anim = Animated::new(false)
            .duration(400.)
            .auto_start(true, 0.)
            .repeat(2)
            .auto_reverse();
        // Begin
        assert_eq!(anim.animate_bool(0., 10., 800.), 0.);
        assert_eq!(anim.animate_bool(0., 10., 1000.), 5.);
        assert_eq!(anim.animate_bool(0., 10., 1200.), 0.);
        assert_eq!(anim.animate_bool(0., 10., 1400.), 5.);
        assert_eq!(anim.animate_bool(0., 10., 1600.), 0.);
        assert_eq!(anim.animate_bool(0., 10., 1800.), 5.);

        // Completion
        assert_eq!(anim.animate_bool(0., 10., 2000.), 10.);

        // No changes after completion
        assert_eq!(anim.animate_bool(0., 10., 2200.), 10.);
        assert_eq!(anim.animate_bool(0., 10., 2400.), 10.);
        assert_eq!(anim.animate_bool(0., 10., 2600.), 10.);
        assert_eq!(anim.animate_bool(0., 10., 2800.), 10.);
        assert_eq!(anim.animate_bool(0., 10., 3000.), 10.);
    }

    impl AnimationTime for f32 {
        fn elapsed_since(self, time: Self) -> f32 {
            self - time
        }
    }

    fn approximately_equal(a: f32, b: f32) -> bool {
        let close = f32::abs(a - b) < 1e-5;
        if !close {
            dbg!(a, b);
        }
        close
    }
}
