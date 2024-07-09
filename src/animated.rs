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
///     .transition(!state.animated_toggle.value(), now);
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
    T: FloatRepresentable + Clone,
    Time: AnimationTime,
{
    animation: Animation<T, Time>,
}

impl<T, Time> Animated<T, Time>
where
    T: FloatRepresentable + Clone,
    Time: AnimationTime,
{
    pub fn value(&self) -> T {
        self.animation
            .transition
            .clone()
            .map_or(self.animation.origin_value.clone(), |t| t.destination_value)
    }
    /// Creates an animated value with specified animation settings
    pub fn new_with_settings(value: T, duration_ms: f32, easing: Easing) -> Self {
        let float = value.float_value();
        let mut animation = Animation::default(value, float);
        animation.settings.duration_ms = duration_ms;
        animation.settings.easing = easing;
        Animated { animation }
    }
    /// Creates an animated value with a default animation
    pub fn new(value: T) -> Self {
        let float = value.float_value();
        Self {
            animation: Animation::default(value, float),
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
        let f = new_value.float_value();
        self.animation.transition(new_value, f, at, false);
    }
    /// Updates the wrapped state & instantaneously completes an animation.
    /// Ignores animation settings such as delay & duration.
    pub fn transition_instantaneous(&mut self, new_value: T, at: Time) {
        let f = new_value.float_value();
        self.animation.transition(new_value, f, at, true);
    }
    /// Returns whether the animation is complete, given the current time
    pub fn in_progress(&self, time: Time) -> bool {
        self.animation.in_progress(time)
    }
    /// Interpolates between states of any value that implements `Interpolable`, given the current time
    pub fn animate<I>(&self, v: impl Fn(T) -> I, time: Time) -> I
    where
        I: Interpolable,
    {
        let from = self.animation.origin_value.clone();
        let to = self
            .animation
            .transition
            .clone()
            .map_or(self.animation.origin_value.clone(), |t| t.destination_value);
        v(from).interpolated(v(to), self.animation.eased_unit_progress(time))
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

impl<Time> Animated<bool, Time>
where
    Time: AnimationTime,
{
    /// Interpolates any value that implements `Interpolable`, given the current time
    pub fn animate_bool<I: Clone>(&self, false_value: I, true_value: I, time: Time) -> I
    where
        I: Interpolable,
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
struct Animation<T, Time>
where
    T: FloatRepresentable + Clone,
{
    origin_value: T,
    origin: f32,
    delay_ms: f32,
    settings: AnimationSettings,
    asymmetric_settings: Option<AnimationSettings>,
    repetitions: u32,
    auto_reverse_repetitions: bool,
    repeat_forever: bool,
    transition: Option<TransitionState<T, Time>>,
}

#[derive(Clone, Copy, Debug, Default)]
struct AnimationSettings {
    duration_ms: f32,
    easing: Easing,
}

#[derive(Clone, Copy, Debug, Default)]
struct TransitionState<T, Time>
where
    T: FloatRepresentable + Clone,
{
    destination_value: T,
    destination: f32,
    start_time: Time,
}

impl<T, Time> Animation<T, Time>
where
    T: FloatRepresentable + Clone,
    Time: AnimationTime,
{
    fn default(origin_value: T, origin: f32) -> Self {
        Animation {
            origin_value,
            origin,
            settings: AnimationSettings {
                duration_ms: 100.,
                easing: Easing::EaseInOut,
            },
            asymmetric_settings: None,
            delay_ms: 0.,
            transition: None,
            repetitions: 1,
            auto_reverse_repetitions: false,
            repeat_forever: false,
        }
    }

    fn transition(
        &mut self,
        destination_value: T,
        destination: f32,
        time: Time,
        instantaneous: bool,
    ) {
        if instantaneous {
            self.origin = destination;
            self.transition = None;
            return;
        }
        self.origin = self.eased_progress(time);
        self.transition = Some(TransitionState {
            destination_value,
            start_time: time,
            destination,
        });
    }

    fn current_settings(&self, time: Time) -> (AnimationSettings, Option<f32>, bool) {
        let Some(transition) = &self.transition else {
            return (self.settings, None, false);
        };
        let asymmetric_settings = self.asymmetric_settings.unwrap_or(self.settings);
        if self.auto_reverse_repetitions {
            let elapsed = f32::max(
                0.,
                time.elapsed_since(transition.start_time) - self.delay_ms,
            );
            let first_duration = if transition.destination > self.origin {
                self.settings.duration_ms
            } else {
                asymmetric_settings.duration_ms
            };
            let total_duration = self.settings.duration_ms + asymmetric_settings.duration_ms;
            let first_animation = elapsed % total_duration - first_duration < 0.;
            let current_elapsed = if first_animation {
                elapsed % total_duration
            } else {
                elapsed % total_duration - first_duration
            };
            if first_animation {
                (self.settings, Some(current_elapsed), false)
            } else {
                (asymmetric_settings, Some(current_elapsed), true)
            }
        } else if transition.destination > self.origin {
            (self.settings, None, false)
        } else {
            (asymmetric_settings, None, true)
        }
    }

    fn linear_unit_progress(&self, time: Time) -> f32 {
        if let Some(transition) = &self.transition {
            let (settings, elapsed, reversing) = self.current_settings(time);
            let elapsed = elapsed.unwrap_or(f32::max(
                0.,
                time.elapsed_since(transition.start_time) - self.delay_ms,
            ));
            let true_repetitions = if self.auto_reverse_repetitions {
                (self.repetitions * 2) + 1
            } else {
                self.repetitions
            };
            let total_duration = settings.duration_ms * true_repetitions as f32;
            if total_duration == 0. {
                return 1.;
            }
            let limited_elapsed = f32::min(elapsed, total_duration);
            let progress_ms: f32;
            if self.repeat_forever {
                progress_ms = elapsed % settings.duration_ms;
            } else if elapsed >= total_duration {
                progress_ms = settings.duration_ms;
            } else {
                progress_ms = limited_elapsed % settings.duration_ms;
            }
            let absolute_unit_progress = progress_ms / settings.duration_ms;
            let unit_progress = if reversing && self.auto_reverse_repetitions {
                //Reversal must be represented in the context of the forward animation in this case
                1. - absolute_unit_progress
            } else {
                absolute_unit_progress
            };
            unit_progress
        } else {
            0.
        }
    }

    fn linear_progress(&self, time: Time) -> f32 {
        self.origin
            + (self.linear_unit_progress(time)
                * (self
                    .transition
                    .clone()
                    .map_or(self.origin, |t| t.destination)
                    - self.origin))
    }

    fn eased_unit_progress(&self, time: Time) -> f32 {
        match &self.transition {
            Some(transition) if transition.destination != self.origin => {
                let (settings, _, _) = self.current_settings(time);
                settings.easing.value(self.linear_unit_progress(time))
            }
            Some(animation) => animation.destination,
            None => self.origin,
        }
    }

    fn eased_progress(&self, time: Time) -> f32 {
        self.origin
            + (self.eased_unit_progress(time)
                * (self
                    .transition
                    .clone()
                    .map_or(self.origin, |t| t.destination)
                    - self.origin))
    }

    fn in_progress(&self, time: Time) -> bool {
        let linear_progress = self.linear_progress(time);
        matches!(&self.transition, Some(animation) if linear_progress != animation.destination)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Hash, Default)]
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
    fn value(self, x: f32) -> f32 {
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
    fn test_transition() {
        let mut anim = Animated::new(0.).duration(1000.).easing(Easing::Linear);
        anim.transition(10.0, 0.0);
        assert!(anim.animation.transition.is_some());
        assert_eq!(anim.animation.transition.unwrap().destination, 10.0);
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
        assert_eq!(anim.linear_progress(1.0), 5.0);
        assert_eq!(anim.linear_progress(1.5), 10.0);

        // Progression backward
        anim.transition(0.0, 1.5);
        assert_eq!(anim.linear_progress(2.5), 0.0);

        // Progression forward in fractions
        anim.transition(10.0, 3.);
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
        assert_eq!(anim.linear_progress(1000.0), 10.); // 100% forward

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
        assert_eq!(anim.linear_progress(1000.0), 10.0); // 100% forward

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
        assert_eq!(anim.linear_progress(0.), 10.);
        assert_eq!(anim.linear_progress(1.), 10.);
        anim.transition_instantaneous(0., 1.);
        assert_eq!(anim.linear_progress(1.), 0.);
        assert_eq!(anim.linear_progress(2.), 0.);
        anim.transition(10., 10.);
        assert_eq!(anim.linear_progress(10.), 0.);
        assert_eq!(anim.linear_progress(1010.), 10.);
        anim.transition_instantaneous(0., 1010.);
        assert_eq!(anim.linear_progress(1010.), 0.);
        assert_eq!(anim.linear_progress(1011.), 0.);
        assert_eq!(anim.linear_progress(1020.), 0.);
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
