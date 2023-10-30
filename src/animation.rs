use iced_core::Color;

pub trait AnimatableValue<T = Self>
where
    Self: Clone + std::fmt::Debug + PartialEq + Sized,
{
    fn distance(&self, other: &Self) -> f32;
    fn diff(&self, other: &Self) -> Self;
    fn sum(&self, other: &Self) -> Self;
    fn scale(&self, amount: f32) -> Self;
    fn magnitude(&self) -> f32;
    fn normalized(&self) -> Self;
}

impl AnimatableValue for (f32, f32) {
    fn distance(&self, other: &Self) -> f32 {
        self.diff(other).magnitude()
    }
    fn diff(&self, other: &Self) -> Self {
        (self.0 - other.0, self.1 - other.1)
    }
    fn sum(&self, other: &Self) -> Self {
        (self.0 + other.0, self.1 + other.1)
    }
    fn scale(&self, amount: f32) -> Self {
        (self.0 * amount, self.1 * amount)
    }
    fn magnitude(&self) -> f32 {
        f32::sqrt(
            vec![self.0, self.1]
                .iter()
                .map(|v| f32::powf(*v, 2.0))
                .sum(),
        )
    }
    fn normalized(&self) -> Self {
        let magnitude = self.magnitude();
        self.scale(1.0 / magnitude)
    }
}

impl AnimatableValue for f32 {
    fn distance(&self, other: &Self) -> f32 {
        self.diff(other).magnitude()
    }
    fn diff(&self, other: &Self) -> Self {
        self - other
    }
    fn sum(&self, other: &Self) -> Self {
        self + other
    }
    fn scale(&self, amount: f32) -> Self {
        self * amount
    }
    fn magnitude(&self) -> f32 {
        f32::sqrt(f32::powf(*self, 2.0))
    }
    fn normalized(&self) -> Self {
        self / self.magnitude()
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Animation<Time, Value>
where
    Value: AnimatableValue,
{
    pub origin: Value,
    pub duration_ms: f32,
    pub timing: Timing,
    pub animation_state: Option<AnimationState<Time, Value>>,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct AnimationState<Time, Value> {
    pub destination: Value,
    pub started_time: Time,
    pub speed_at_interrupt: Option<f32>,
}

pub trait AnimationTime: Copy {
    fn elapsed_since(self, time: Self) -> f32;
}

impl AnimationTime for std::time::Instant {
    fn elapsed_since(self, time: Self) -> f32 {
        (self - time).as_millis() as f32
    }
}

impl<Time, Value> Animation<Time, Value>
where
    Time: AnimationTime + std::fmt::Debug,
    Value: AnimatableValue,
{
    pub fn new(origin: Value, duration: f32, timing: Timing) -> Self {
        Animation {
            origin,
            duration_ms: duration,
            timing,
            animation_state: None,
        }
    }

    pub fn transition(&mut self, destination: Value, time: Time) {
        let timed_progress = self.timed_progress(time);
        let linear_progress = self.linear_progress(time);
        match &mut self.animation_state {
            Some(animation) if linear_progress != animation.destination => {
                // Snapshot current state as the new animation origin
                if animation.speed_at_interrupt.is_none() {
                    animation.speed_at_interrupt =
                        Some(animation.destination.distance(&self.origin) / self.duration_ms);
                }
                self.origin = timed_progress;
                animation.destination = destination;
                animation.started_time = time;
            }

            Some(_) | None => {
                self.origin = linear_progress;
                self.animation_state = Some(AnimationState {
                    started_time: time,
                    destination,
                    speed_at_interrupt: None,
                })
            }
        }
    }

    pub fn linear_progress(&self, time: Time) -> Value {
        if let Some(animation) = &self.animation_state {
            let elapsed = time.elapsed_since(animation.started_time);
            let position_delta: Value;
            if let Some(speed) = animation.speed_at_interrupt {
                let direction = animation.destination.diff(&self.origin).normalized();
                position_delta = direction.scale(elapsed * speed);
            } else {
                let duration = self.duration_ms;
                let delta = elapsed / duration;
                let direction = animation.destination.diff(&self.origin);
                position_delta = direction.scale(delta);
            }
            if self.duration_ms == 0.0
                || position_delta.magnitude() >= self.origin.distance(&animation.destination)
            {
                return animation.destination.clone();
            } else {
                return self.origin.sum(&position_delta);
            }
        };
        self.origin.clone()
    }

    pub fn timed_progress(&self, time: Time) -> Value {
        match &self.animation_state {
            Some(animation) if animation.destination != self.origin => {
                let position = self.linear_progress(time);
                let progress_in_animation = position.distance(&self.origin);
                let range_of_animation = animation.destination.distance(&self.origin);
                let completion = progress_in_animation / range_of_animation;
                let animation_range = animation.destination.diff(&self.origin);
                self.origin
                    .sum(&animation_range.scale(self.timing.timing(completion)))
            }
            _ => return self.origin.clone(),
        }
    }

    pub fn in_progress(&self, time: Time) -> bool {
        let linear_progress = self.linear_progress(time);
        match &self.animation_state {
            Some(animation) if linear_progress != animation.destination => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum Timing {
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

impl Timing {
    fn timing(self, linear_progress: f32) -> f32 {
        let x = linear_progress;
        let pi = std::f32::consts::PI;
        match self {
            Timing::Linear => linear_progress,
            Timing::EaseIn => 1.0 - f32::cos((x * pi) / 2.0),
            Timing::EaseOut => f32::sin((x * pi) / 2.0),
            Timing::EaseInOut => -(f32::cos(pi * x) - 1.0) / 2.0,
            Timing::EaseInQuint => x * x * x * x * x,
            Timing::EaseOutQuint => 1.0 - f32::powf(1.0 - x, 5.0),
            Timing::EaseInOutQuint => {
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

pub trait Interpolable {
    fn interpolated(self, other: Self, ratio: f32) -> Self;
}

impl Interpolable for Color {
    fn interpolated(self, other: Self, ratio: f32) -> Self {
        if ratio >= 1.0 {
            return other;
        } else if ratio > 0.0 {
            return Color::new(
                self.r.interpolated(other.r, ratio),
                self.g.interpolated(other.g, ratio),
                self.b.interpolated(other.b, ratio),
                self.a.interpolated(other.a, ratio),
            );
        } else {
            return self;
        }
    }
}

impl Interpolable for f32 {
    fn interpolated(self, other: Self, ratio: f32) -> Self {
        self * (1.0 - ratio) + other * ratio
    }
}

impl<T> Interpolable for Option<T>
where
    T: Interpolable + Copy,
{
    fn interpolated(self, other: Self, ratio: f32) -> Self {
        match (self, other) {
            (Some(a), Some(b)) => Some(a.interpolated(b, ratio)),
            _ => other,
        }
    }
}

#[cfg(test)]
mod animatedvalue_tests {
    use super::*;

    #[test]
    fn test_instant_animation() {
        let mut anim = Animation::<f32, f32>::new(0.0, 1.0, Timing::Linear);
        let clock = 0.0;
        assert_eq!(anim.linear_progress(clock), 0.0);
        // If animation duration is 0.0 the transition should happen instantly
        // & require a redraw without any time passing
        anim.transition(10.0, clock);
        assert_eq!(anim.linear_progress(clock), 0.0);
    }

    #[test]
    fn test_progression() {
        let mut anim = Animation::<f32, f32>::new(0.0, 1.0, Timing::Linear);
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
        let mut anim = Animation::<f32, f32>::new(0.0, 1.0, Timing::Linear);
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
        let mut anim = Animation::<f32, f32>::new(1.0, 10.0, Timing::EaseIn);
        let mut clock = 0.0;

        // Interrupt halfway through with asymmetrical timing
        anim.transition(0.0, clock);
        assert_eq!(anim.linear_progress(clock), 1.0);
        clock += 1.0;
        let progress_at_interrupt = anim.timed_progress(clock);
        assert_eq!(progress_at_interrupt, 1.0 - Timing::EaseIn.timing(0.1));

        // Interrupted animation should begin from wherever the timed function
        // was interrupted, which is different from the linear progress.
        anim.transition(1.0, clock);
        assert_eq!(anim.animation_state.unwrap().destination, 1.0);
        assert_eq!(anim.timed_progress(clock), progress_at_interrupt);
        assert!(anim.animation_state.unwrap().speed_at_interrupt.is_some());
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
        let mut anim = Animation::<f32, f32>::new(0.0, 1.0, Timing::EaseInOut);
        let mut clock = 0.0;
        anim.transition(1.0, clock);
        clock += 0.5;
        assert!(anim.in_progress(clock));
        let progress_at_interrupt = anim.timed_progress(clock);
        assert_eq!(progress_at_interrupt, Timing::EaseInOut.timing(0.5));
        anim.transition(0.0, clock);
        assert_eq!(anim.timed_progress(clock), progress_at_interrupt);
        clock += 0.2;
        assert!(anim.in_progress(clock));
        anim.transition(1.0, clock);
        clock += 0.2;
        assert!(anim.in_progress(clock));
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
