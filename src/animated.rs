use crate::traits::{AnimationTime, FloatRepresentable, Interpolable};
/// Wraps state to enable interpolated transitions
///
/// # Example
/// // Define
/// struct MyViewState {
///    animated_toggle: Animated<bool, Instant>,
/// }
/// // Initialize
/// MyViewState {
///    animated_toggle: Animated::new(false, 300., Easing::EaseOut),
/// }
/// // Update
/// let now = std::time::Instant::now();
/// self
///    .animated_toggle
///    .transition(!self.animated_toggle.value, now)
/// // Interpolate
/// let interpolated_width = self.animated_toggle.interpolate(100., 500., now)
#[derive(Clone, Debug, Default)]
pub struct Animated<T, Time>
where
    T: FloatRepresentable,
    Time: AnimationTime,
{
    /// The wrapped state - updates to this value can be interpolated
    pub value: T,
    animation: InterpolatedInterruptionAnimation<Time>,
}

impl<T, Time> Animated<T, Time>
where
    T: FloatRepresentable,
    Time: AnimationTime,
{
    /// Creates an animated value
    pub fn new(value: T, duration_ms: f32, timing: Easing, delay_ms: f32) -> Self {
        let float = value.float_value();
        Animated {
            value,
            animation: InterpolatedInterruptionAnimation::new(Animation::new(
                float,
                duration_ms,
                timing,
                delay_ms,
            )),
        }
    }
    /// Updates the wrapped state & begins an animation
    pub fn transition(&mut self, new_value: T, at: Time) {
        self.animation.transition(new_value.float_value(), at);
        self.value = new_value
    }
    /// Returns whether the animation is complete, given the current time
    // pub fn in_progress(self, time: Time) -> bool {
    //     self.animation.in_progress(time)
    // }
    /// Interpolates any value that implements `Interpolable`, given the current time.
    pub fn animate<I>(&self, from: I, to: I, time: Time) -> I
    where
        I: Interpolable,
    {
        from.interpolated(to, self.animation.timed_progress(time))
    }
}

#[derive(Clone, Debug, Default)]
struct InterpolatedInterruptionAnimation<Time> {
    animation: Animation<Time>,
    interrupted: Vec<Interruption<Time>>,
}

#[derive(Clone, Copy, Debug, Default)]
struct Interruption<Time> {
    animation: Animation<Time>,
    time: Time,
}

impl<Time> InterpolatedInterruptionAnimation<Time>
where
    Time: AnimationTime,
{
    fn new(animation: Animation<Time>) -> Self {
        Self {
            animation,
            interrupted: Vec::new(),
        }
    }
    fn transition(&mut self, destination: f32, time: Time) {
        if let Some(interrupted) = self.animation.transition(destination, time) {
            // Clean up interruptions
            self.interrupted.retain(|i| {
                time.elapsed_since(i.time) < self.animation.interrupt_lerp_duration_ms()
            });
            if self.interrupted.len() >= MAX_INTERRUPTIONS {
                self.interrupted.remove(0);
            }
            self.interrupted.push(interrupted);
        }
    }
    fn timed_progress(&self, time: Time) -> f32 {
        if !self.interrupted.is_empty() {
            let mut interrupted_sum = 0.;
            let interrupted_weights: Vec<f32> = self
                .interrupted
                .iter()
                .map(|i| {
                    1.0 - f32::max(
                        0.0,
                        f32::min(
                            1.0,
                            time.elapsed_since(i.time)
                                / self.animation.interrupt_lerp_duration_ms(),
                        ),
                    )
                })
                .collect();
            let total_interrupted_weight: f32 = interrupted_weights.iter().sum();

            if total_interrupted_weight > 0.0 {
                for (i, interruption) in self.interrupted.iter().enumerate() {
                    let weight = interrupted_weights[i] / total_interrupted_weight;
                    interrupted_sum += interruption.animation.timed_progress(time) * weight;
                }

                let new_weight = Easing::EaseInOut
                    .value(1. - total_interrupted_weight / self.interrupted.len() as f32);
                let interrupted_weight = 1.0 - new_weight;

                let new_progress = self.animation.timed_progress(time) * new_weight;

                return new_progress + (interrupted_sum * interrupted_weight);
            }
        }
        self.animation.timed_progress(time)
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Animation<Time> {
    origin: f32,
    duration_ms: f32,
    timing: Easing,
    delay_ms: f32,
    animation_state: Option<AnimationState<Time>>,
}

const MAX_INTERRUPTIONS: usize = 1;
const INTERRUPT_LERP_DURATION_RATIO: f32 = 0.25;

#[derive(Clone, Copy, Debug, Default)]
struct AnimationState<Time> {
    destination: f32,
    start_time: Time,
}

impl<Time> Animation<Time>
where
    Time: AnimationTime,
{
    fn new(origin: f32, duration_ms: f32, timing: Easing, delay_ms: f32) -> Self {
        Animation {
            origin,
            duration_ms,
            timing,
            delay_ms,
            animation_state: None,
        }
    }

    fn interrupt_lerp_duration_ms(&self) -> f32 {
        f32::min(
            (self.duration_ms * INTERRUPT_LERP_DURATION_RATIO) + self.delay_ms,
            200.,
        )
    }

    fn transition(&mut self, destination: f32, time: Time) -> Option<Interruption<Time>> {
        let linear_progress = self.linear_progress(time);
        let interrupted = self.clone();
        let interrupt_lerp_duration =
            if self.linear_progress(time.advanced_by(self.interrupt_lerp_duration_ms())) >= 1. {
                0.
            } else {
                self.interrupt_lerp_duration_ms()
            };
        match &mut self.animation_state {
            Some(animation) if linear_progress != animation.destination => {
                // Snapshot current state as the new animation origin
                self.origin = interrupted.timed_progress(time.advanced_by(interrupt_lerp_duration));
                animation.destination = destination;
                animation.start_time = time.advanced_by(interrupt_lerp_duration);
                return Some(Interruption {
                    animation: interrupted,
                    time,
                });
            }

            Some(_) | None => {
                self.origin = linear_progress;
                self.animation_state = Some(AnimationState {
                    start_time: time,
                    destination,
                });
                return None;
            }
        }
    }

    fn linear_progress(&self, time: Time) -> f32 {
        if let Some(animation) = &self.animation_state {
            let elapsed = f32::max(0., time.elapsed_since(animation.start_time) - self.delay_ms);
            assert!(elapsed.is_sign_positive());
            let position_delta: f32;
            let duration = self.duration_ms;
            let delta = f32::max(0., f32::min(1., elapsed / duration));
            let direction = animation.destination - self.origin;
            position_delta = direction * delta;
            if self.duration_ms == 0.0
                || position_delta >= f32::abs(self.origin + animation.destination)
            {
                return animation.destination.clone();
            } else {
                return self.origin + position_delta;
            }
        };
        self.origin.clone()
    }

    fn timed_progress(&self, time: Time) -> f32 {
        match &self.animation_state {
            Some(animation) if animation.destination != self.origin => {
                let position = self.linear_progress(time);
                let progress_in_animation = f32::abs(position - self.origin);
                let range_of_animation = f32::abs(animation.destination - self.origin);
                let completion = progress_in_animation / range_of_animation;
                let animation_range = animation.destination - self.origin;
                let result = self.origin + (animation_range * self.timing.value(completion));
                return result;
            }
            Some(animation) => animation.destination.clone(),
            None => self.origin.clone(),
        }
    }

    fn in_progress(&self, time: Time) -> bool {
        let linear_progress = self.linear_progress(time);
        match &self.animation_state {
            Some(animation) if linear_progress != animation.destination => true,
            _ => false,
        }
    }
}

/// Animation easing curves - defined as a function from 0 to 1
#[derive(Clone, Copy, Debug, Default, PartialEq, Hash)]
pub enum Easing {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,
    Custom,
}

impl Easing {
    fn value(self, linear_progress: f32) -> f32 {
        let x = linear_progress;
        let pi = std::f32::consts::PI;
        match self {
            Easing::Linear => linear_progress,
            Easing::EaseIn => 1.0 - f32::cos((x * pi) / 2.0),
            Easing::EaseOut => f32::sin((x * pi) / 2.0),
            Easing::EaseInOut => -(f32::cos(pi * x) - 1.0) / 2.0,
            Easing::EaseInQuint => x * x * x * x * x,
            Easing::EaseOutQuint => 1.0 - f32::powf(1.0 - x, 5.0),
            Easing::EaseInOutQuint => {
                if x < 0.5 {
                    16.0 * x * x * x * x * x
                } else {
                    1.0 - f32::powf(-2.0 * x + 2.0, 5.0) / 2.0
                }
            }
            _ => linear_progress,
        }
    }
}

#[cfg(test)]
mod animatedvalue_tests {
    use super::*;

    #[test]
    fn test_instant_animation() {
        let mut anim = Animation::<f32>::new(0.0, 1.0, Easing::Linear, 0.);
        let clock = 0.0;
        assert_eq!(anim.linear_progress(clock), 0.0);
        // If animation duration is 0.0 the transition should happen instantly
        // & require a redraw without any time passing
        anim.transition(10.0, clock);
        assert_eq!(anim.linear_progress(clock), 0.0);
    }

    #[test]
    fn test_progression() {
        let mut anim = Animation::<f32>::new(0.0, 1.0, Easing::Linear, 0.);
        let mut clock = 0.0;
        // With a duration of 1.0 & linear timing we should be halfway to our
        // destination at 0.5
        anim.transition(10.0, clock);
        clock += 0.5;
        assert_eq!(anim.linear_progress(clock), 5.0);
        clock += 0.5;
        assert_eq!(anim.linear_progress(clock), 10.0);

        // Progression backward
        anim.transition(0.0, clock);
        clock += 1.0;
        assert_eq!(anim.linear_progress(clock), 0.0);

        // Progression forward in thirds
        anim.transition(10.0, clock);
        clock += 0.2;
        assert!(approximately_equal(anim.linear_progress(clock), 2.0));
        clock += 0.2;
        assert!(approximately_equal(anim.linear_progress(clock), 4.0));
        clock += 0.4;
        assert!(approximately_equal(anim.linear_progress(clock), 8.0));
        clock += 0.2;
        assert!(approximately_equal(anim.linear_progress(clock), 10.0));
    }

    #[test]
    fn test_interrupt() {
        let mut anim = Animation::<f32>::new(0.0, 1.0, Easing::Linear, 0.);
        let mut clock = 0.0;
        // Interruptions should continue at the same speed the interrupted
        // animation was progressing at.
        anim.transition(10.0, clock);
        clock += 0.5;
        assert_eq!(anim.linear_progress(clock), 5.0);
        // If we interrupt exactly halfway through distance & duration we
        // should arrive back at the start with another half of the duration
        anim.transition(0.0, clock);
        clock += 0.5;
        assert_eq!(anim.linear_progress(clock), 0.0);

        // Begin an animation
        anim.transition(10.0, clock);
        clock += 0.2;
        assert!(approximately_equal(anim.linear_progress(clock), 2.0));
        // Interrupt one fifth of the way through
        // The animation is playing at 10 units per time unit
        // The target is only 1.0 away
        // We should arrive at the target after 0.1 time units
        anim.transition(1.0, clock);
        clock += 0.1;
        assert!(approximately_equal(anim.linear_progress(clock), 1.0));
    }

    #[test]
    fn test_interrupt_nonlinear() {
        let mut anim = Animation::<f32>::new(1.0, 10.0, Easing::EaseIn, 0.);
        let mut clock = 0.0;

        // Interrupt halfway through with asymmetrical timing
        anim.transition(0.0, clock);
        assert_eq!(anim.linear_progress(clock), 1.0);
        clock += 1.0;
        let progress_at_interrupt = anim.timed_progress(clock);
        assert_eq!(progress_at_interrupt, 1.0 - Easing::EaseIn.value(0.1));

        // Interrupted animation should begin from wherever the timed function
        // was interrupted, which is different from the linear progress.
        anim.transition(1.0, clock);
        assert_eq!(anim.animation_state.unwrap().destination, 1.0);
        assert_eq!(anim.timed_progress(clock), progress_at_interrupt);
        // Since we've interrupted at some in-between, non-linear point in
        // the animation, the time it takes to finish won't be as clean.
        // It should take a bit less time to return home because it's an
        // EaseIn timing curve. The animation we interrupted was easing in
        // & therefore closer to where it started.
        clock += 3.0;
        assert_eq!(anim.linear_progress(clock), 1.0);
    }

    #[test]
    fn test_multiple_interrupts_start_forward() {
        let mut anim = Animation::<f32>::new(0.0, 1.0, Easing::EaseInOut, 0.);
        let mut clock = 0.0;
        anim.transition(1.0, clock);
        clock += 0.5;
        assert!(anim.in_progress(clock));
        let progress_at_interrupt = anim.timed_progress(clock);
        assert_eq!(progress_at_interrupt, Easing::EaseInOut.value(0.5));
        anim.transition(0.0, clock);
        assert_eq!(anim.timed_progress(clock), progress_at_interrupt);
        clock += 0.2;
        assert!(anim.in_progress(clock));
        anim.transition(1.0, clock);
        clock += 0.2;
        assert!(anim.in_progress(clock));
    }

    impl AnimationTime for f32 {
        type Duration = f32;
        fn elapsed_since(self, time: Self) -> f32 {
            self - time
        }
        fn advanced_by(self, duration_ms: f32) -> Self {
            self + duration_ms
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
